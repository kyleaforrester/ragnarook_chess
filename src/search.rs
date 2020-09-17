use crate::{UciOption, UciValue, UciGo};
use crate::misc;
use crate::board;
use crate::move_gen;
use std::cmp;
use std::time::{SystemTime, Instant, Duration, UNIX_EPOCH};
use std::sync::{Weak, RwLock, Mutex, Arc, RwLockWriteGuard};

const SEED_XOR: u64 = 0x77de55f9d2fe1e0d;
const AVG_CHILD_COUNT: f32 = 50.0;

pub struct Node {
    board: board::Board,
    visits: RwLock<u32>,
    depth: RwLock<u32>,
    height: RwLock<u32>,
    eval: RwLock<f32>,
    ending: RwLock<Option<Ending>>,
    children: RwLock<Vec<Arc<Node>>>,
    parent: Weak<Node>,
    last_move: Option<String>,
    //proc_threads is number of threads searching in this node's children
    //helps spread out mcts search to prevent thread clumping
    proc_threads: RwLock<u32>,
}

enum Ending {
    Draw,
    //Win tracks how many moves from mate we are
    Win(u32),
    //Loss tracks how many moves from mate we are
    Loss(u32),
}

impl Node {
    pub fn new(pos: board::Board) -> Node {
        Node {
            board: pos,
            visits: RwLock::new(1),
            depth: RwLock::new(0),
            height: RwLock::new(0),
            eval: RwLock::new(0.5),
            ending: RwLock::new(None),
            children: RwLock::new(Vec::new()),
            parent: Weak::new(),
            last_move: None,
            proc_threads: RwLock::new(0),
        }
    }
}

pub fn search(root: Arc<Node>, options: Arc<Vec<UciOption>>, searching: Arc<Mutex<bool>>, go_parms: Arc<UciGo>, main: bool) {
    let start_time = Instant::now();
    let mut last_info = Instant::now();
    let mut rng_state: u64 = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64) ^ SEED_XOR;

    // Unpack UCI options
    let multi_pv = match options.iter().find(|&&x| x.name == "MultiPV").unwrap().value {
        UciValue::Spin{value, default, min, max} => value,
        _ => panic!("MultiPV UCI Option should be a UciValue::Spin option!"),
    };
    let move_overhead = match options.iter().find(|&&x| x.name == "Move_Overhead").unwrap().value {
        UciValue::Spin{value, default, min, max} => value,
        _ => panic!("Move_Overhead UCI Option should be a UciValue::Spin option!"),
    };
    let move_speed = match options.iter().find(|&&x| x.name == "Move_Speed").unwrap().value {
        UciValue::Spin{value, default, min, max} => value,
        _ => panic!("Move_Speed UCI Option should be a UciValue::Spin option!"),
    };
    let mcts_explore = match options.iter().find(|&&x| x.name == "MCTS_Explore").unwrap().value {
        UciValue::Spin{value, default, min, max} => value,
        _ => panic!("MCTS_Explore UCI Option should be a UciValue::Spin option!"),
    };
    let mcts_hash = match options.iter().find(|&&x| x.name == "MCTS_Hash").unwrap().value {
        UciValue::Spin{value, default, min, max} => value,
        _ => panic!("MCTS_Hash UCI Option should be a UciValue::Spin option!"),
    };
    let skill = match options.iter().find(|&&x| x.name == "Skill").unwrap().value {
        UciValue::Spin{value, default, min, max} => value,
        _ => panic!("Skill UCI Option should be a UciValue::Spin option!"),
    };
    let contempt = match options.iter().find(|&&x| x.name == "Contempt").unwrap().value {
        UciValue::Spin{value, default, min, max} => value,
        _ => panic!("Contempt UCI Option should be a UciValue::Spin option!"),
    };
    let dynamism = match options.iter().find(|&&x| x.name == "Dynamism").unwrap().value {
        UciValue::Spin{value, default, min, max} => value,
        _ => panic!("Dynamism UCI Option should be a UciValue::Spin option!"),
    };

    while searching.lock().unwrap() {
        // MutexGuard is already dropped due to not being assigned a variable
        // navigate through the tree to identify leaf node
        let (leaf, child_lock) = find_leaf_node(&root, mcts_explore, mcts_hash);
        // bloom leaf node
        move_gen::bloom(&leaf, child_lock);
        // propogate values back up the tree
        propogate_values(&leaf);

        if main {
            if last_info.elapsed() >= Duration::from_secs(2) {
                print_info(root, multi_pv, &start_time);
                last_info = Instant::now();
            }
            if stop_searching(root, &start_time, &go_parms, move_overhead, move_speed, mcts_hash) {
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

fn print_info(root: &Arc<Node>, multi_pv: u32, start_time: &Instant) {
    let mut children_sorted: Vec<(u32, u32)> = Vec::new();
    let children = root.children.read().unwrap();
    for child in children.iter().enumerate() {
        let node = Arc::clone(child.1);
        children_sorted.push((child.0, node.visits.read().unwrap()));
    }

    children_sorted.sort_unstable_by(|a, b| a.1 > b.1);

    let depth = root.depth.read().unwrap();
    let nodes = root.visits.read().unwrap();
    let time = start_time.elapsed();
    let nps = (nodes as f32) / time.as_secs_f32();

    for i in 0..cmp::max(multi_pv as usize, children_sorted.len()) {
        let child = Arc::clone(&children[&children_sorted[i].0]);
        let pv = get_pv(&child);
        println!("info multipv {} depth {} seldepth {} time {} nodes {} pv_nodes {} nps {} score cp {} tbhits 0 pv {}", i, depth, depth, time.as_millis(), nodes, child.visits.read().unwrap(), nps, misc::eval_to_cp(child.eval.read().unwrap()), pv.trim());
    }
}

fn get_pv(node: &Arc<Node>) -> String {
    let mut pv = String::new();
    pv.push_str(node.last_move.unwrap());

    let mut children_sorted: Vec<(u32, u32)> = Vec::new();

    let children = node.children.read().unwrap();
    for child in children.iter().enumerate() {
        let node = Arc::clone(child.1);
        children_sorted.push((child.0, node.visits.read().unwrap()));
    }

    children_sorted.sort_unstable_by(|a, b| a.1 > b.1);

    if children_sorted.len() > 0 {
        pv.push(' ');
        pv.push_str(get_pv(Arc::clone(&children[&children_sorted[0].0])));
    }

    pv
}

fn print_bestmove(root: &Arc<Node>, skill: u32, rng_state: &mut u64) {
    let mut children_sorted: Vec<(u32, f32)> = Vec::new();
    let children = root.children.read().unwrap();

    if skill < 25 {
        println!("info Skill set to {}", skill);

        let mut rng: u32;
        for child in children.iter().enumerate() {
            let node = Arc::clone(child.1);
            (rng, rng_state) = misc::spcg32(rng_state);
            let percent_loss = ((rng as f32) / (std::u32::MAX as f32)) * ((25 - skill) as f32 / 25 as f32) * 2;
            let actual = node.visits.read().unwrap() as f32;
            children_sorted.push((child.0, (actual - (actual * percent_loss)) as i32));
        }
    }
    else {
        for child in children.iter().enumerate() {
            let node = Arc::clone(child.1);
            children_sorted.push((child.0, node.visits.read().unwrap() as f32));
        }
    }

    children_sorted.sort_unstable_by(|a, b| a.1 > b.1);

    let node = Arc::clone(&children[&children_sorted[0].0]);
    println!("bestmove {}", node.last_move.unwrap());
}

fn find_leaf_node (root: &Arc<Node>, mcts_explore: u32) -> (Arc<Node>, RwLockWriteGuard<Vec<Node>>) {
    let mut node = root;

    loop {
        let mut children = node.children.read().unwrap();
        while children.len() > 0 {
            // Continue down the path to the next child
            let children_sorted: Vec<(u32, f32)> = Vec::new();

            for child in children.iter().enumerate() {
                children_sorted.push((child.0, mcts_score(child.1, mcts_explore, node.visits.read().unwrap())));
            }
            children_sorted.sort_unstable_by(|a, b| a.1 > b.1);

            node = &children[&children_sorted[0].0];
            let mut threads = node.proc_threads.write().unwrap();
            *threads += 1;

            children = node.children.read().unwrap();
        }

        match node.children.try_write() {
            Ok(g) => return (node, g),
            Err(_) => {
                decr_proc_threads(node);
                node = root;
            }
        }
    }
}

fn mcts_score(node: &Node, mcts_explore: u32, parent_visits: u32) -> f32 {
    let my_move = if node.height.read().unwrap() % 2 == 0 {
            true
        }
        else {
            false
        };

    let eval = node.eval.read().unwrap();
    let visits = node.visits.read().unwrap();
    let threads = node.proc_threads.read().unwrap();
    let explore = ((parent_visits as f32).ln() / ((visits as f32) + AVG_CHILD_COUNT * (threads as f32))).sqrt();
    let scale = ((mcts_explore as f32) / 50.0 - 1.0).exp2();

    if my_move {
        eval + scale * explore
    }
    else {
        (1 - eval) + scale * explore
    }
}

fn decr_proc_threads(node: &Node) {
    
}

fn propogate_values(node: &Node) {

}

fn stop_searching(root: &Node, start_time: &Instant, go_parms: &Arc<UciGo>, move_overhead: u32, move_speed: u32, mcts_hash: u32) -> bool {
    false
}
