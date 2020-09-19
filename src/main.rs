mod board;
mod search;
mod misc;
mod move_gen;

use board::Board;
use search::Node;
use std::thread;
use std::sync::{RwLock, Mutex, Arc, RwLockWriteGuard};

struct UciOption {
    value: UciValue,
    name: String,
}

enum UciValue {
    Button,
    Check{value: bool, default: bool},
    Spin{value: i32, default: i32, min: i32, max: i32},
}

enum UciGo {
    Time{wtime: Option<u32>, btime: Option<u32>, winc: Option<u32>, binc: Option<u32>, movestogo: Option<u32>},
    Depth{plies: u32},
    Nodes{count: u32},
    Movetime{mseconds: u32},
    Infinite,
}

enum PositionState {
    Initial,
    StartPos,
    Fen,
    Moves,
}

enum GoState {
    WTime,
    BTime,
    WInc,
    BInc,
    Movestogo,
    Initial,
}

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {

    let (mut options, mut root) = initialize();
    let mut searching = Arc::new(Mutex::new(false));

    println!("Ragnarook 0.1 by Kyle Forrester");

    loop {
        let input = tokenize_stdin();
        match input[0].as_str() {
            "uci" => uci_uci(&options),
            "isready" => uci_isready(),
            "setoption" => uci_setoption(&mut options, input),
            "ucinewgame" => root = uci_newgame(),
            "position" => root = uci_position(root, input),
            "go" => uci_go(&root, &options, &searching, input),
            "stop" => uci_stop(&searching),
            "quit" => uci_quit(),
            _ => println!("Invalid command: {}", input[0]),
        }
    }
}

fn initialize() -> (Arc<Vec<UciOption>>, Arc<Node>) {
    let mut options = Arc::new(Vec::new());
    options.push(UciOption {
        name: String::from("Threads"),
        value: UciValue::Spin {
            value: 1,
            default: 1,
            min: 1,
            max: 2048,
        }
    });
    options.push(UciOption {
        name: String::from("MultiPV"),
        value: UciValue::Spin {
            value: 1,
            default: 1,
            min: 1,
            max: 256,
        }
    });
    options.push(UciOption {
        name: String::from("Move_Overhead"),
        value: UciValue::Spin {
            value: 100,
            default: 100,
            min: 10,
            max: 5000,
        }
    });
    options.push(UciOption {
        name: String::from("Move_Speed"),
        value: UciValue::Spin {
            value: 50,
            default: 50,
            min: 0,
            max: 100,
        }
    });
    options.push(UciOption {
        name: String::from("MCTS_Explore"),
        value: UciValue::Spin {
            value: 50,
            default: 50,
            min: 0,
            max: 100,
        }
    });
    options.push(UciOption {
        name: String::from("MCTS_Hash"),
        value: UciValue::Spin {
            value: 256,
            default: 256,
            min: 16,
            max: 32768,
        }
    });
    options.push(UciOption {
        name: String::from("Skill"),
        value: UciValue::Spin {
            value: 25,
            default: 25,
            min: 0,
            max: 25,
        }
    });
    options.push(UciOption {
        name: String::from("Contempt"),
        value: UciValue::Spin {
            value: 0,
            default: 0,
            min: -100,
            max: 100,
        }
    });
    options.push(UciOption {
        name: String::from("Dynamism"),
        value: UciValue::Spin {
            value: 50,
            default: 50,
            min: 0,
            max: 100,
        }
    });

    let mut root = Arc::new(Node::new(Board::new(STARTPOS)));

    (options, root)
}

fn uci_uci(options: &Vec<UciOption>) {
    println!("id name Rust_Chess 0.1.0");
    println!("id author Kyle Forrester");

    for option in options {
        match option.value{
            UciValue::Button => println!("option name {} type button", option.name),
            UciValue::Check{value, default} => println!("option name {} type check default {}", option.name, default),
            UciValue::Spin{value, default, min, max} => println!("option name {} type spin default {} min {} max{}", option.name, default, min, max),
        }
    }

    println!("uciok");
}

fn uci_isready() {
    println!("readyok");
}

fn uci_setoption(mut options: &Vec<UciOption>, input: Vec<String>) {
    if input[1] != "name" || input[3] != "value" {
        println!("Unrecognized UCI setoption command");
        return;
    }

    //Find the option the user is trying to modify
    let mut option = match options.iter_mut().find(|&x| x.name == input[2]) {
        Some(o) => o,
        None => {
            println!("Unrecognized UCI setoption command");
            return;
        }
    };

    //Set the option's value to the user input
    match option.value {
        UciValue::Check{value, default} => match input[4].as_str() {
            "true" => value = true,
            "false" => value = false,
            _ => println!("Unrecognized UCI setoption command"),
        },
        UciValue::Spin{value, default, min, max} => match input[4].trim().parse() {
            Ok(v) => value = v,
            Err(_) => println!("Unrecognized UCI setoption command"),
        },
        _ => println!("Internal Error. UCI property not initialized appropriately."),
    }
}

fn uci_newgame() -> Arc<Node> {
    return Arc::new(Node::new(Board::new(STARTPOS)));
}

fn uci_position(root: Arc<Node>, input: Vec<String>) -> Arc<Node> {
    if input.len() <= 1 {
        println!("Unrecognized UCI position command");
        return root;
    }
    //Build out what the board should look like
    let mut pos_state = PositionState::Initial;
    let mut fen = String::from(STARTPOS);
    let mut fen_accumulator: Vec<String> = Vec::new();
    let mut moves_accumulator: Vec<String> = Vec::new();
    for token in input.iter().skip(1) {
        match token.as_str() {
            "startpos" => {
                pos_state = PositionState::StartPos;
                fen = String::from(STARTPOS);
            },
            "fen" => {
                pos_state = PositionState::Fen;
            },
            "moves" => {
                pos_state = PositionState::Moves;
            },
            _ => {
                match pos_state {
                    PositionState::Initial => {
                        println!("Unexpected parameter {}!", token);
                        return root;
                    },
                    PositionState::StartPos => {
                        println!("Unexpected parameter {}!", token);
                        return root;
                    },
                    PositionState::Fen => {
                        fen_accumulator.push(token.to_string());
                    },
                    PositionState::Moves => {
                        moves_accumulator.push(token.to_string());
                    }
                }
            }
        }
    }
    
    if fen_accumulator.len() > 0 {
        fen = fen_accumulator.join(" ");
    }
    
    //Create the board position
    let board = Board::new(&fen);
    for mov in moves_accumulator.iter() {
        board.do_move(mov);
    }
    
    //Set the root node to the current node, child node, or grandchild with matching board
    //If no one matches the board, start a new root node with the correct board
    if root.board.eq(&board) {
        return root;
    }
    let children = root.children.read().unwrap();
    for child in children.iter() {
        if child.board.eq(&board) {
            return Arc::clone(child);
        }
        let grandchildren = child.children.read().unwrap();
        for grandchild in grandchildren.iter() {
            if grandchild.board.eq(&board) {
                return Arc::clone(grandchild);
            }
        }
    }

    //None were found, start a new root node
    Arc::new(Node::new(board))
}

fn uci_go(root: &Arc<Node>, options: &Arc<Vec<UciOption>>, searching: &Arc<Mutex<bool>>, input: Vec<String>) {
    let mut s = searching.lock().unwrap();
    *s = true;

    let go_cmd = Arc::new(parse_go_command(input));
    let new_root = Arc::clone(root);
    let new_options = Arc::clone(options);
    let new_searching = Arc::clone(searching);
    let new_go_cmd = Arc::clone(&go_cmd);

    thread::spawn(move || {
        search::search(new_root, new_options, new_searching, new_go_cmd, true);
    });


    let threads = match options.iter().find(|&&x| x.name == "Threads").unwrap().value {
        UciValue::Spin{value, default, min, max} => value,
        _ => panic!("Threads UCI Option should be a Spin option!"),
    };

    for i in 0..threads-1 {
        let new_root = Arc::clone(root);
        let new_options = Arc::clone(options);
        let new_searching = Arc::clone(searching);
        let new_go_cmd = Arc::clone(&go_cmd);
        thread::spawn(move || {
            search::search(new_root, new_options, new_searching, new_go_cmd, false);
        });
    }
}

fn parse_go_command(input: Vec<String>) -> UciGo {
    match input[1].as_str() {
        "wtime" | "btime" | "winc" | "binc" | "movestogo" => parse_go_time(input),
        "depth" => parse_go_depth(input),
        "nodes" => parse_go_nodes(input),
        "movetime" => parse_go_movetime(input),
        "infinite" => parse_go_infinite(input),
    }
}

fn parse_go_time(input: Vec<String>) -> UciGo {
    let mut wtime = Option::None;
    let mut btime = Option::None;
    let mut winc = Option::None;
    let mut binc = Option::None;
    let mut movestogo = Option::None;
    let mut state = GoState::Initial;
    for word in &input[1..] {
        match word.as_str() {
            "wtime" => state = GoState::WTime,
            "btime" => state = GoState::BTime,
            "winc" => state = GoState::WInc,
            "binc" => state = GoState::BInc,
            "movestogo" => state = GoState::Movestogo,
            _ => {
                match state {
                    GoState::WTime => wtime = Some(word.parse().unwrap()),
                    GoState::BTime => btime = Some(word.parse().unwrap()),
                    GoState::WInc => winc = Some(word.parse().unwrap()),
                    GoState::BInc => binc = Some(word.parse().unwrap()),
                    GoState::Movestogo => movestogo = Some(word.parse().unwrap()),
                    GoState::Initial => panic!("Go command requires arguments!"),
                }
            }
        }
    }

    let go_enum = UciGo::Time {
        wtime: wtime,
        btime: btime,
        winc: winc,
        binc: binc,
        movestogo: movestogo,
    };
    if go_enum.wtime.is_none() && go_enum.btime.is_none() {
        panic!("Go command with times must implement either wtime or btime!");
    }
    go_enum
}

fn parse_go_depth(input: Vec<String>) -> UciGo {
    UciGo::Depth {
        plies: input[2].parse().unwrap(),
    }
}

fn parse_go_nodes(input: Vec<String>) -> UciGo {
    UciGo::Nodes{
        count: input[2].parse().unwrap(),
    }
}

fn parse_go_movetime(input: Vec<String>) -> UciGo {
    UciGo::Movetime{
        mseconds: input[2].parse().unwrap(),
    }
}

fn parse_go_infinite(input: Vec<String>) -> UciGo {
    UciGo::Infinite
}

fn uci_stop(searching: &Mutex<bool>) {
    let mut s = searching.lock().unwrap();
    *s = false;
}

fn uci_quit() {
    std::process::exit(0);
}

fn tokenize_stdin() -> Vec<String> {
    let mut input = String::new();

    io::stdin().read_line(&mut input).expect("Error reading from stdin");
    input.split_ascii_whitespace().collect();



    String::from("Yo")
}
