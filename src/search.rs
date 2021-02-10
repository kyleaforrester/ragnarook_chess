use crate::board;
use crate::eval;
use crate::misc;
use crate::move_gen;
use crate::UciGo::{Depth, Infinite, Movetime, Nodes, Time};
use crate::{UciGo, UciOption, UciValue};
use std::cmp;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SEED_XOR: u64 = 0x77de55f9d2fe1e0d;
const AVG_CHILD_COUNT: f32 = 50.0;
const MAX_GAME_LENGTH: u32 = 60;
const TIME_EXTENSION_MULT_MAX: f32 = 3.0;
const BYTES_PER_NODE: u32 = 1000;

pub struct Node {
    pub board: board::Board,
    visits: RwLock<u32>,
    depth: RwLock<u32>,
    eval: RwLock<f32>,
    ending: RwLock<Option<Ending>>,
    pub children: RwLock<Vec<Arc<Node>>>,
    parent: Weak<Node>,
    last_move: Option<String>,
    //proc_threads is number of threads searching in this node's children
    //helps spread out mcts search to prevent thread clumping
    proc_threads: RwLock<u32>,
}

#[derive(Copy, Clone)]
enum Ending {
    Draw,
    //Win tracks how many moves from mate we are
    WhiteWin(u32),
    //Loss tracks how many moves from mate we are
    BlackWin(u32),
}

impl Node {
    pub fn new(pos: board::Board) -> Node {
        Node {
            board: pos,
            visits: RwLock::new(1),
            depth: RwLock::new(0),
            eval: RwLock::new(0.5),
            ending: RwLock::new(None),
            children: RwLock::new(Vec::new()),
            parent: Weak::new(),
            last_move: None,
            proc_threads: RwLock::new(0),
        }
    }

    pub fn spawn(leaf: &Arc<Node>, mut board: board::Board, last_move: String) -> Node {
        if board.is_w_move {
            board.is_w_move = false;
        } else {
            board.is_w_move = true;
            board.fullmove_clock += 1;
        }
        let eval = eval::evaluate(&board);
        Node {
            board: board,
            visits: RwLock::new(1),
            depth: RwLock::new(0),
            eval: RwLock::new(eval),
            ending: RwLock::new(None),
            children: RwLock::new(Vec::new()),
            parent: Arc::downgrade(leaf),
            last_move: Some(last_move),
            proc_threads: RwLock::new(0),
        }
    }
}

pub fn search(
    root: Arc<Node>,
    options: Vec<UciOption>,
    searching: Arc<Mutex<bool>>,
    go_parms: UciGo,
    main: bool,
) {
    let start_time = Instant::now();
    let mut last_info = Instant::now();
    let mut rng_state: u64 = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64)
        ^ SEED_XOR;

    // Unpack UCI options
    let multi_pv = match options.iter().find(|&x| x.name == "MultiPV").unwrap().value {
        UciValue::Spin {
            value,
            default: _,
            min: _,
            max: _,
        } => value,
        _ => panic!("MultiPV UCI Option should be a UciValue::Spin option!"),
    };
    let move_overhead = match options
        .iter()
        .find(|&x| x.name == "Move_Overhead")
        .unwrap()
        .value
    {
        UciValue::Spin {
            value,
            default: _,
            min: _,
            max: _,
        } => value,
        _ => panic!("Move_Overhead UCI Option should be a UciValue::Spin option!"),
    };
    let move_speed = match options
        .iter()
        .find(|&x| x.name == "Move_Speed")
        .unwrap()
        .value
    {
        UciValue::Spin {
            value,
            default: _,
            min: _,
            max: _,
        } => value,
        _ => panic!("Move_Speed UCI Option should be a UciValue::Spin option!"),
    };
    let mcts_explore = match options
        .iter()
        .find(|&x| x.name == "MCTS_Explore")
        .unwrap()
        .value
    {
        UciValue::Spin {
            value,
            default: _,
            min: _,
            max: _,
        } => value,
        _ => panic!("MCTS_Explore UCI Option should be a UciValue::Spin option!"),
    };
    let mcts_hash = match options
        .iter()
        .find(|&x| x.name == "MCTS_Hash")
        .unwrap()
        .value
    {
        UciValue::Spin {
            value,
            default: _,
            min: _,
            max: _,
        } => value,
        _ => panic!("MCTS_Hash UCI Option should be a UciValue::Spin option!"),
    };
    let skill = match options.iter().find(|&x| x.name == "Skill").unwrap().value {
        UciValue::Spin {
            value,
            default: _,
            min: _,
            max: _,
        } => value,
        _ => panic!("Skill UCI Option should be a UciValue::Spin option!"),
    };
    let _contempt = match options
        .iter()
        .find(|&x| x.name == "Contempt")
        .unwrap()
        .value
    {
        UciValue::Spin {
            value,
            default: _,
            min: _,
            max: _,
        } => value,
        _ => panic!("Contempt UCI Option should be a UciValue::Spin option!"),
    };
    let _dynamism = match options
        .iter()
        .find(|&x| x.name == "Dynamism")
        .unwrap()
        .value
    {
        UciValue::Spin {
            value,
            default: _,
            min: _,
            max: _,
        } => value,
        _ => panic!("Dynamism UCI Option should be a UciValue::Spin option!"),
    };

    while *searching.lock().unwrap() {
        // MutexGuard is already dropped due to not being assigned a variable
        // navigate through the tree to identify leaf node
        let leaf = match find_and_bloom_leaf_node(&root, mcts_explore) {
            Ok(n) => n,
            Err(_) => break,
        };
        // propogate values back up the tree
        propogate_values(&leaf);

        if main {
            if last_info.elapsed() >= Duration::from_secs(2) {
                print_info(&root, multi_pv, &start_time);
                last_info = Instant::now();
            }
            if stop_searching(
                &root,
                &start_time,
                &go_parms,
                move_overhead,
                move_speed,
                mcts_hash,
            ) {
                let mut s = searching.lock().unwrap();
                *s = false;
            }
        }
    }

    if main {
        // print info
        print_info(&root, multi_pv, &start_time);
        // print bestmove
        print_bestmove(&root, skill, &mut rng_state);
    }
}

fn print_info(root: &Arc<Node>, multi_pv: i32, start_time: &Instant) {
    let mut children_sorted: Vec<(usize, u32)> = Vec::new();
    let children = root.children.read().unwrap();
    for child in children.iter().enumerate() {
        let node = Arc::clone(child.1);
        children_sorted.push((child.0, *node.visits.read().unwrap()));
    }

    children_sorted.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let depth = *root.depth.read().unwrap();
    let nodes = *root.visits.read().unwrap();
    let time = start_time.elapsed();
    let nps = (nodes as f32) / time.as_secs_f32();

    for i in 0..cmp::min(multi_pv as usize, children_sorted.len()) {
        let child = Arc::clone(&children[children_sorted[i].0]);
        let pv = get_pv(&child);
        println!("info multipv {} depth {} seldepth {} time {} nodes {} pv_nodes {} nps {} score cp {} tbhits 0 pv {}", i + 1, depth, depth, time.as_millis(), nodes, child.visits.read().unwrap(), nps, misc::eval_to_cp(*child.eval.read().unwrap()), pv.trim());
    }
}

fn get_pv(node: &Arc<Node>) -> String {
    let mut pv = String::new();
    pv.push_str(node.last_move.as_ref().unwrap());

    let mut children_sorted: Vec<(usize, u32)> = Vec::new();

    let children = node.children.read().unwrap();
    for child in children.iter().enumerate() {
        let node = Arc::clone(child.1);
        children_sorted.push((child.0, *node.visits.read().unwrap()));
    }

    children_sorted.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    if children_sorted.len() > 0 {
        pv.push(' ');
        pv.push_str(&get_pv(&children[children_sorted[0].0]));
    }

    pv
}

fn print_bestmove(root: &Arc<Node>, skill: i32, rng_state: &mut u64) {
    let mut children_sorted: Vec<(usize, f32)> = Vec::new();
    let children = root.children.read().unwrap();

    // Check for game endings
    match best_move_adjudication(root) {
        Some(m) => {
            println!("bestmove {}", m);
            return;
        }
        None => (),
    }

    // Never move into unforced checkmate or draw
    let valid_children: Vec<&Arc<Node>> = children
        .iter()
        .filter(|x| x.ending.read().unwrap().is_none())
        .collect();

    // Game is not over, select by node count
    if skill < 100 {
        println!("info Skill set to {}", skill);

        let mut rng: u32;
        for child in valid_children.iter().enumerate() {
            let node = Arc::clone(child.1);
            let (tmp_rng, tmp_rng_state) = misc::spcg32(rng_state);
            rng = tmp_rng;
            *rng_state = tmp_rng_state;
            let percent_loss =
                ((rng as f32) / (std::u32::MAX as f32)) * ((100 - skill) as f32 / 100 as f32) * 2.0;
            let actual = *node.visits.read().unwrap() as f32;
            children_sorted.push((child.0, actual - (actual * percent_loss)));
        }
    } else {
        for child in valid_children.iter().enumerate() {
            let node = Arc::clone(child.1);
            children_sorted.push((child.0, *node.visits.read().unwrap() as f32));
        }
    }

    children_sorted.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let node = Arc::clone(&children[children_sorted[0].0]);
    println!("bestmove {}", node.last_move.as_ref().unwrap());
}

fn best_move_adjudication(root: &Arc<Node>) -> Option<String> {
    let children = root.children.read().unwrap();

    // Check for game endings
    match *root.ending.read().unwrap() {
        Some(e) => {
            let mut fast_w_win = u32::MAX;
            let mut slow_w_win = 0;
            let mut fast_b_win = u32::MAX;
            let mut slow_b_win = 0;
            let mut fast_w_node = 0;
            let mut slow_w_node = 0;
            let mut fast_b_node = 0;
            let mut slow_b_node = 0;
            for child in children.iter().enumerate() {
                match *child.1.ending.read().unwrap() {
                    Some(c_e) => match c_e {
                        Ending::Draw => (),
                        Ending::WhiteWin(m) => {
                            if m < fast_w_win {
                                fast_w_win = m;
                                fast_w_node = child.0;
                            }
                            if m > slow_w_win {
                                slow_w_win = m;
                                slow_w_node = child.0;
                            }
                        }
                        Ending::BlackWin(m) => {
                            if m < fast_b_win {
                                fast_b_win = m;
                                fast_b_node = child.0;
                            }
                            if m > slow_b_win {
                                slow_b_win = m;
                                slow_b_node = child.0;
                            }
                        }
                    },
                    None => (),
                }
            }
            match e {
                Ending::Draw => {
                    // Get a random Draw child
                    let draw_children: Vec<&Arc<Node>> = children
                        .iter()
                        .filter(|x| match *x.ending.read().unwrap() {
                            Some(e) => match e {
                                Ending::Draw => true,
                                Ending::WhiteWin(m) => false,
                                Ending::BlackWin(m) => false,
                            },
                            None => false,
                        })
                        .collect();
                    Some(draw_children[0].last_move.as_ref().unwrap().to_string())
                }
                Ending::WhiteWin(m) => {
                    if root.board.is_w_move {
                        Some(
                            children[fast_w_node]
                                .last_move
                                .as_ref()
                                .unwrap()
                                .to_string(),
                        )
                    } else {
                        Some(
                            children[slow_w_node]
                                .last_move
                                .as_ref()
                                .unwrap()
                                .to_string(),
                        )
                    }
                }
                Ending::BlackWin(m) => {
                    if root.board.is_w_move {
                        Some(
                            children[slow_b_node]
                                .last_move
                                .as_ref()
                                .unwrap()
                                .to_string(),
                        )
                    } else {
                        Some(
                            children[fast_b_node]
                                .last_move
                                .as_ref()
                                .unwrap()
                                .to_string(),
                        )
                    }
                }
            }
        }
        None => {
            // Check to see if a draw is better than continuing
            if (root.board.is_w_move && *root.eval.read().unwrap() < 0.5)
                || (!root.board.is_w_move && *root.eval.read().unwrap() > 0.5)
            {
                // Get a random Draw child
                let draw_children: Vec<&Arc<Node>> = children
                    .iter()
                    .filter(|x| match *x.ending.read().unwrap() {
                        Some(e) => match e {
                            Ending::Draw => true,
                            Ending::WhiteWin(m) => false,
                            Ending::BlackWin(m) => false,
                        },
                        None => false,
                    })
                    .collect();
                if draw_children.len() > 0 {
                    return Some(draw_children[0].last_move.as_ref().unwrap().to_string());
                }
            }
            None
        }
    }
}

fn find_and_bloom_leaf_node(root: &Arc<Node>, mcts_explore: i32) -> Result<Arc<Node>, String> {
    'outer: loop {
        if root.ending.read().unwrap().is_some() {
            return Err("Game Over".to_string());
        }
        let mut node = Arc::clone(root);
        let mut returning = false;
        let mut placeholder;
        *node.proc_threads.write().unwrap() += 1;

        'inner: loop {
            {
                let children = node.children.read().unwrap();
                if children.len() == 0 {
                    break;
                }

                let valid_children: Vec<&Arc<Node>> = children
                    .iter()
                    .filter(|x| x.ending.read().unwrap().is_none())
                    .collect();
                if valid_children.len() == 0 {
                    continue 'outer;
                }

                let mut children_sorted: Vec<(usize, f32)> = Vec::new();
                for child in valid_children.iter().enumerate() {
                    children_sorted.push((
                        child.0,
                        mcts_score(
                            *child.1,
                            mcts_explore,
                            *node.visits.read().unwrap(),
                            node.board.is_w_move,
                        ),
                    ));
                }

                children_sorted.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                placeholder = Arc::clone(valid_children[children_sorted[0].0]);
            }

            node = Arc::clone(&placeholder);
            *node.proc_threads.write().unwrap() += 1;
        }

        match node.children.try_write() {
            Ok(g) => {
                move_gen::bloom(&node, g);
                returning = true;
            }
            Err(_) => (),
        }

        if returning {
            return Ok(node);
        }

        // Failed to lock leaf node, start back at beginning
        decr_proc_threads(&node);
    }
}

fn mcts_score(node: &Arc<Node>, mcts_explore: i32, parent_visits: u32, is_w_move: bool) -> f32 {
    let eval = *node.eval.read().unwrap();
    let visits = *node.visits.read().unwrap();
    let threads = *node.proc_threads.read().unwrap();
    let explore = ((parent_visits as f32).ln()
        / ((visits as f32) + AVG_CHILD_COUNT * (threads as f32)))
        .sqrt();
    let scale = 4.0_f32.powf((mcts_explore as f32) / 50.0 - 1.0);

    if is_w_move {
        eval + scale * explore
    } else {
        (1.0 - eval) + scale * explore
    }
}

fn decr_proc_threads(node: &Arc<Node>) {
    loop {
        let mut new_node = Arc::clone(node);
        {
            let mut threads = new_node.proc_threads.write().unwrap();
            *threads -= 1;
        }
        match new_node.parent.upgrade() {
            Some(a) => new_node = Arc::clone(&a),
            None => break,
        }
    }
}

fn propogate_values(leaf: &Arc<Node>) {
    let mut node = Arc::clone(leaf);

    loop {
        {
            let children = node.children.read().unwrap();
            let mut visits = node.visits.write().unwrap();
            let mut depth = node.depth.write().unwrap();
            let mut eval = node.eval.write().unwrap();
            let mut ending = node.ending.write().unwrap();
            let mut threads = node.proc_threads.write().unwrap();

            *visits = 0;
            *threads -= 1;

            let length = children.len();
            let mut w_wins = 0;
            let mut b_wins = 0;
            let mut draws = 0;
            let mut fast_w_win = u32::MAX;
            let mut slow_w_win = 0;
            let mut fast_b_win = u32::MAX;
            let mut slow_b_win = 0;
            for child in children.iter() {
                // Update parent eval
                let c_eval = child.eval.read().unwrap();
                if node.board.is_w_move {
                    if *c_eval > *eval {
                        *eval = *c_eval;
                    }
                } else {
                    if *c_eval < *eval {
                        *eval = *c_eval;
                    }
                }

                // Update parent visits
                *visits += *child.visits.read().unwrap();

                // Update parent depth
                let c_depth = child.depth.read().unwrap();
                if *c_depth > *depth {
                    *depth = *c_depth;
                }

                // Sample child endings
                match *child.ending.read().unwrap() {
                    Some(e) => match e {
                        Ending::Draw => draws += 1,
                        Ending::WhiteWin(m) => {
                            w_wins += 1;
                            if m + 1 < fast_w_win {
                                fast_w_win = m + 1;
                            } else if m + 1 > slow_w_win {
                                slow_w_win = m + 1;
                            }
                        }
                        Ending::BlackWin(m) => {
                            b_wins += 1;
                            if m + 1 < fast_b_win {
                                fast_b_win = m + 1;
                            } else if m + 1 > slow_b_win {
                                slow_b_win = m + 1;
                            }
                        }
                    },
                    None => (),
                }
            }

            // Update parent ending
            if node.board.is_w_move {
                if w_wins > 0 {
                    *ending = Some(Ending::WhiteWin(fast_w_win));
                } else if draws == length - b_wins {
                    *ending = Some(Ending::Draw);
                } else if b_wins == length {
                    *ending = Some(Ending::BlackWin(slow_b_win));
                }
            } else {
                if b_wins > 0 {
                    *ending = Some(Ending::BlackWin(fast_b_win));
                } else if draws == length - w_wins {
                    *ending = Some(Ending::Draw);
                } else if w_wins == length {
                    *ending = Some(Ending::WhiteWin(slow_w_win));
                }
            }
        }

        // Move to the parent
        match node.parent.upgrade() {
            Some(n) => node = Arc::clone(&n),
            None => break,
        }
    }
}

fn stop_searching(
    root: &Arc<Node>,
    start_time: &Instant,
    go_parms: &UciGo,
    move_overhead: i32,
    move_speed: i32,
    mcts_hash: i32,
) -> bool {
    if root.children.read().unwrap().len() < 2 {
        return true;
    }
    if *root.visits.read().unwrap() * BYTES_PER_NODE > u32::try_from(mcts_hash).unwrap() * 1048576 {
        return true;
    }
    match *go_parms {
        Time {
            wtime,
            winc,
            btime,
            binc,
            movestogo,
        } => {
            let (time_left, time_inc) = if root.board.is_w_move {
                (wtime, winc)
            } else {
                (btime, binc)
            };
            let time_inc = match time_inc {
                Some(t) => t,
                None => 0,
            };

            let m_to_go = match movestogo {
                Some(s) => s,
                None => cmp::min(
                    misc::eval_to_movestogo(*root.eval.read().unwrap()),
                    MAX_GAME_LENGTH,
                ),
            };

            let need_extension = needs_extension(root);
            let speed = 4.0_f32.powf((move_speed as f32) / 50.0 - 1.0);
            let _nodes = root.visits.read().unwrap();
            let time_allowed = (m_to_go * time_inc + time_left.unwrap()
                - u32::try_from(move_overhead).unwrap()) as f32
                / (m_to_go as f32 * speed);

            if need_extension {
                return (time_allowed * TIME_EXTENSION_MULT_MAX) as u128
                    > start_time.elapsed().as_millis();
            } else {
                return time_allowed as u128 > start_time.elapsed().as_millis();
            }
        }
        Depth { plies } => {
            let depth = root.depth.read().unwrap();
            *depth > plies
        }
        Nodes { count } => {
            let visits = root.visits.read().unwrap();
            *visits > count
        }
        Movetime { mseconds } => {
            mseconds > u32::try_from(start_time.elapsed().as_millis()).unwrap()
        }
        Infinite => false,
    }
}

fn needs_extension(root: &Arc<Node>) -> bool {
    let mut children_sorted: Vec<(usize, u32)> = Vec::new();
    let children = root.children.read().unwrap();
    for child in children.iter().enumerate() {
        let node = Arc::clone(child.1);
        children_sorted.push((child.0, *node.visits.read().unwrap()));
    }

    children_sorted.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    if *children[children_sorted[0].0].eval.read().unwrap()
        < *children[children_sorted[1].0].eval.read().unwrap()
    {
        true
    } else {
        false
    }
}
