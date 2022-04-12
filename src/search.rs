use crate::board;
use crate::eval;
use crate::misc;
use crate::move_gen;
use crate::UciGo::{Depth, Infinite, Movetime, Nodes, Time};
use crate::{UciGo, UciOption, UciValue};
use std::cmp::{self, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SEED_XOR: u64 = 0x77de55f9d2fe1e0d;
const AVG_CHILD_COUNT: f32 = 50.0;
const MAX_GAME_LENGTH: u32 = 100;
const TIME_EXTENSION_MULT_MAX: f32 = 3.0;
const BYTES_PER_NODE: u64 = 880;

#[derive(Debug)]
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

#[derive(Copy, Clone, Debug)]
pub enum Ending {
    Draw,
    //Win tracks how many moves from mate we are
    WhiteWin(u32),
    //Loss tracks how many moves from mate we are
    BlackWin(u32),
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if !self.board.is_w_move {
            // Root board is white to move.  This child board is black to move.
            // Evaluated for white's perspective.
            if other.board.is_w_move {
                return None;
            }
            match *self.ending.read().unwrap() {
                Some(le) => match *other.ending.read().unwrap() {
                    Some(re) => match le {
                        Ending::Draw => match re {
                            Ending::Draw => Some(Ordering::Equal),
                            Ending::WhiteWin(rm) => Some(Ordering::Less),
                            Ending::BlackWin(rm) => Some(Ordering::Greater),
                        },
                        Ending::WhiteWin(lm) => match re {
                            Ending::Draw => Some(Ordering::Greater),
                            Ending::WhiteWin(rm) => Some(rm.cmp(&lm)),
                            Ending::BlackWin(rm) => Some(Ordering::Greater),
                        },
                        Ending::BlackWin(lm) => match re {
                            Ending::Draw => Some(Ordering::Less),
                            Ending::WhiteWin(rm) => Some(Ordering::Less),
                            Ending::BlackWin(rm) => Some(lm.cmp(&rm)),
                        },
                    },
                    None => match le {
                        Ending::Draw => {
                            if *other.eval.read().unwrap() < 0.5 {
                                Some(Ordering::Greater)
                            } else if *other.eval.read().unwrap() > 0.5 {
                                Some(Ordering::Less)
                            } else {
                                Some(Ordering::Equal)
                            }
                        }
                        Ending::WhiteWin(lm) => Some(Ordering::Greater),
                        Ending::BlackWin(lm) => Some(Ordering::Less),
                    },
                },
                None => {
                    match *other.ending.read().unwrap() {
                        Some(re) => match re {
                            Ending::Draw => {
                                if *self.eval.read().unwrap() > 0.5 {
                                    Some(Ordering::Greater)
                                } else if *self.eval.read().unwrap() < 0.5 {
                                    Some(Ordering::Less)
                                } else {
                                    Some(Ordering::Equal)
                                }
                            }
                            Ending::WhiteWin(rm) => Some(Ordering::Less),
                            Ending::BlackWin(rm) => Some(Ordering::Greater),
                        },
                        None => {
                            match self
                                .visits
                                .read()
                                .unwrap()
                                .partial_cmp(&other.visits.read().unwrap())
                                .unwrap()
                            {
                                Ordering::Less => Some(Ordering::Less),
                                Ordering::Greater => Some(Ordering::Greater),
                                Ordering::Equal => self
                                    .eval
                                    .read()
                                    .unwrap()
                                    .partial_cmp(&other.eval.read().unwrap()),
                            }
                        } //None => self.eval.read().unwrap().partial_cmp(&other.eval.read().unwrap()),
                    }
                }
            }
        } else {
            // Root board is black to move.  This child board is white to move.
            // Evaluated for black's perspective.
            if !other.board.is_w_move {
                return None;
            }
            match *self.ending.read().unwrap() {
                Some(le) => match *other.ending.read().unwrap() {
                    Some(re) => match le {
                        Ending::Draw => match re {
                            Ending::Draw => Some(Ordering::Equal),
                            Ending::WhiteWin(rm) => Some(Ordering::Greater),
                            Ending::BlackWin(rm) => Some(Ordering::Less),
                        },
                        Ending::WhiteWin(lm) => match re {
                            Ending::Draw => Some(Ordering::Less),
                            Ending::WhiteWin(rm) => Some(lm.cmp(&rm)),
                            Ending::BlackWin(rm) => Some(Ordering::Less),
                        },
                        Ending::BlackWin(lm) => match re {
                            Ending::Draw => Some(Ordering::Greater),
                            Ending::WhiteWin(rm) => Some(Ordering::Greater),
                            Ending::BlackWin(rm) => Some(rm.cmp(&lm)),
                        },
                    },
                    None => match le {
                        Ending::Draw => {
                            if *other.eval.read().unwrap() > 0.5 {
                                Some(Ordering::Greater)
                            } else if *other.eval.read().unwrap() < 0.5 {
                                Some(Ordering::Less)
                            } else {
                                Some(Ordering::Equal)
                            }
                        }
                        Ending::WhiteWin(lm) => Some(Ordering::Less),
                        Ending::BlackWin(lm) => Some(Ordering::Greater),
                    },
                },
                None => {
                    match *other.ending.read().unwrap() {
                        Some(re) => match re {
                            Ending::Draw => {
                                if *self.eval.read().unwrap() < 0.5 {
                                    Some(Ordering::Greater)
                                } else if *self.eval.read().unwrap() > 0.5 {
                                    Some(Ordering::Less)
                                } else {
                                    Some(Ordering::Equal)
                                }
                            }
                            Ending::WhiteWin(rm) => Some(Ordering::Greater),
                            Ending::BlackWin(rm) => Some(Ordering::Less),
                        },
                        None => {
                            match self
                                .visits
                                .read()
                                .unwrap()
                                .partial_cmp(&other.visits.read().unwrap())
                                .unwrap()
                            {
                                Ordering::Less => Some(Ordering::Less),
                                Ordering::Greater => Some(Ordering::Greater),
                                Ordering::Equal => other
                                    .eval
                                    .read()
                                    .unwrap()
                                    .partial_cmp(&self.eval.read().unwrap()),
                            }
                        } //None => other.eval.read().unwrap().partial_cmp(&self.eval.read().unwrap()),
                    }
                }
            }
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match self.partial_cmp(other) {
            Some(o) => match o {
                Ordering::Equal => true,
                _ => false,
            },
            None => false,
        }
    }
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
        let (end, eval) = eval::evaluate(&board);
        Node {
            board: board,
            visits: RwLock::new(1),
            depth: RwLock::new(0),
            eval: RwLock::new(eval),
            ending: RwLock::new(end),
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
                print_info(&root, multi_pv, &start_time, &mut rng_state);
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
        print_info(&root, multi_pv, &start_time, &mut rng_state);
        // print bestmove
        let best_node = get_bestmove(&root, skill, &mut rng_state).unwrap();
        println!("bestmove {}", best_node.last_move.as_ref().unwrap());
    }
}

fn print_info(root: &Arc<Node>, multi_pv: i32, start_time: &Instant, rng_state: &mut u64) {
    let mut children = root.children.write().unwrap();
    children.sort_unstable_by(|a, b| b.partial_cmp(&a).unwrap());
    drop(children);

    let time = start_time.elapsed();
    let nodes = *root.visits.read().unwrap();
    let nps = (nodes as f32) / time.as_secs_f32();

    let children = root.children.read().unwrap();
    for i in 0..cmp::min(multi_pv as usize, children.len()) {
        let child = Arc::clone(&children[i]);
        let pv = get_pv(&child);
        let eval = match *child.ending.read().unwrap() {
            Some(e) => match e {
                Ending::Draw => "cp 0".to_string(),
                Ending::WhiteWin(m) => format!("mate {}", m),
                Ending::BlackWin(m) => format!("mate -{}", m),
            },
            None => format!("cp {}", misc::eval_to_cp(*child.eval.read().unwrap())),
        };
        let depth = *child.depth.read().unwrap();
        println!("info multipv {} depth {} seldepth {} time {} nodes {} pv_nodes {} nps {} score {} tbhits 0 pv {}", i + 1, depth, depth, time.as_millis(), nodes, child.visits.read().unwrap(), nps, eval, pv.trim());
    }
}

fn get_pv(node: &Arc<Node>) -> String {
    let mut pv = String::new();
    let mut next_node = Arc::clone(node);
    pv.push_str(node.last_move.as_ref().unwrap());

    loop {
        match get_bestmove(&next_node, 100, &mut 0) {
            Some(n) => {
                pv.push_str(" ");
                pv.push_str(n.last_move.as_ref().unwrap());
                next_node = n;
            }
            None => break,
        }
    }

    pv
}

fn get_bestmove(root: &Arc<Node>, skill: i32, rng_state: &mut u64) -> Option<Arc<Node>> {
    let mut children = root.children.write().unwrap();
    children.sort_unstable_by(|a, b| b.partial_cmp(&a).unwrap());
    if children.len() > 0 {
        Some(Arc::clone(&children[0]))
    } else {
        None
    }

    /*
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
    */
}

fn best_move_adjudication(root: &Arc<Node>) -> Option<Arc<Node>> {
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
                    let draw_child = children
                        .iter()
                        .filter(|x| match *x.ending.read().unwrap() {
                            Some(e) => match e {
                                Ending::Draw => true,
                                Ending::WhiteWin(m) => false,
                                Ending::BlackWin(m) => false,
                            },
                            None => false,
                        })
                        .nth(0)
                        .unwrap();
                    Some(Arc::clone(draw_child))
                }
                Ending::WhiteWin(m) => {
                    if root.board.is_w_move {
                        Some(Arc::clone(&children[fast_w_node]))
                    } else {
                        Some(Arc::clone(&children[slow_w_node]))
                    }
                }
                Ending::BlackWin(m) => {
                    if root.board.is_w_move {
                        Some(Arc::clone(&children[slow_b_node]))
                    } else {
                        Some(Arc::clone(&children[fast_b_node]))
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
                    return Some(Arc::clone(draw_children[0]));
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
                    decr_proc_threads(&node);
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
                *node.depth.write().unwrap() = 1;
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
    let scale = 1.02_f32.powf((mcts_explore as f32) - 100.0);

    if is_w_move {
        eval + scale * explore
    } else {
        (1.0 - eval) + scale * explore
    }
}

fn decr_proc_threads(node: &Arc<Node>) {
    let mut new_node = Arc::clone(node);
    loop {
        *new_node.proc_threads.write().unwrap() -= 1;
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

            *node.proc_threads.write().unwrap() -= 1;

            let mut new_visits = 1;
            let length = children.len();
            let mut w_wins = 0;
            let mut b_wins = 0;
            let mut draws = 0;
            let mut fast_w_win = u32::MAX;
            let mut slow_w_win = 0;
            let mut fast_b_win = u32::MAX;
            let mut slow_b_win = 0;
            let mut new_eval = if length > 0 {
                children[0].eval.read().unwrap().deref().clone()
            } else {
                0.5
            };
            for child in children.iter() {
                // Update new eval
                let c_eval = child.eval.read().unwrap();
                let eval = node.eval.read().unwrap();
                if node.board.is_w_move {
                    if *c_eval > new_eval {
                        new_eval = *c_eval;
                        //println!("Replacing eval {} with {} for {} from {}", *eval, new_eval, node.board.to_string(), child.board.to_string());
                    }
                } else {
                    if *c_eval < new_eval {
                        new_eval = *c_eval;
                        //println!("Replacing eval {} with {} for {} from {}", *eval, new_eval, node.board.to_string(), child.board.to_string());
                    }
                }
                drop(eval);
                drop(c_eval);

                // Update parent visits
                new_visits += *child.visits.read().unwrap();

                // Update parent depth
                let c_depth = child.depth.read().unwrap();
                let mut depth = node.depth.write().unwrap();
                if *c_depth + 1 > *depth {
                    *depth = *c_depth + 1;
                }
                drop(depth);
                drop(c_depth);

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
            *node.visits.write().unwrap() = new_visits;
            *node.eval.write().unwrap() = new_eval;

            // Update parent ending
            // Checkmate and Stalemate calculation for leaf nodes with no children
            if length == 0 {
                if node.board.is_w_move
                    && move_gen::is_attacked(&node.board, false, node.board.w_k_bb)
                {
                    *node.ending.write().unwrap() = Some(Ending::BlackWin(0));
                } else if !node.board.is_w_move
                    && move_gen::is_attacked(&node.board, true, node.board.b_k_bb)
                {
                    *node.ending.write().unwrap() = Some(Ending::WhiteWin(0));
                } else {
                    *node.ending.write().unwrap() = Some(Ending::Draw);
                }
            } else {
                // Not a leaf node with no children, propogate values
                if node.board.is_w_move {
                    if w_wins > 0 {
                        *node.ending.write().unwrap() = Some(Ending::WhiteWin(fast_w_win));
                    } else if draws > 0 && draws == length - b_wins {
                        *node.ending.write().unwrap() = Some(Ending::Draw);
                    } else if b_wins == length {
                        *node.ending.write().unwrap() = Some(Ending::BlackWin(slow_b_win));
                    }
                } else {
                    if b_wins > 0 {
                        *node.ending.write().unwrap() = Some(Ending::BlackWin(fast_b_win));
                    } else if draws > 0 && draws == length - w_wins {
                        *node.ending.write().unwrap() = Some(Ending::Draw);
                    } else if w_wins == length {
                        *node.ending.write().unwrap() = Some(Ending::WhiteWin(slow_w_win));
                    }
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
    if root
        .children
        .read()
        .unwrap()
        .iter()
        .all(|x| x.ending.read().unwrap().is_some())
    {
        return true;
    }
    if u64::try_from(*root.visits.read().unwrap()).unwrap() * BYTES_PER_NODE
        > u64::try_from(mcts_hash).unwrap() * 1048576
    {
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
            let time_allowed = (time_left.unwrap() - u32::try_from(move_overhead).unwrap()) as f32;
            let time_ration = time_allowed / (m_to_go as f32 * speed);

            if need_extension {
                return cmp::min(
                    ((time_ration * TIME_EXTENSION_MULT_MAX) as u32 + time_inc) as u128,
                    time_allowed as u128,
                ) < start_time.elapsed().as_millis();
            } else {
                return cmp::min(
                    (time_ration as u32 + time_inc) as u128,
                    time_allowed as u128,
                ) < start_time.elapsed().as_millis();
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
    let mut children = root.children.write().unwrap();
    children.sort_unstable_by(|a, b| b.partial_cmp(&a).unwrap());
    drop(children);
    let children = root.children.read().unwrap();

    if children.len() <= 1 {
        return false;
    }

    match *children[0].ending.read().unwrap() {
        Some(e) => match e {
            Ending::Draw => return true,
            _ => return false,
        },
        None => (),
    }

    if *children[0].eval.read().unwrap() <= *children[1].eval.read().unwrap() {
        return true;
    }
    false
}
