mod board;
mod eval;
mod magic;
mod misc;
mod move_gen;
mod search;

use board::Board;
use search::Node;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct UciOption {
    value: UciValue,
    name: String,
}

#[derive(Clone)]
pub enum UciValue {
    Button,
    Check {
        value: bool,
        default: bool,
    },
    Spin {
        value: i32,
        default: i32,
        min: i32,
        max: i32,
    },
}

#[derive(Clone)]
pub enum UciGo {
    Time {
        wtime: Option<u32>,
        btime: Option<u32>,
        winc: Option<u32>,
        binc: Option<u32>,
        movestogo: Option<u32>,
    },
    Depth {
        plies: u32,
    },
    Nodes {
        count: u32,
    },
    Movetime {
        mseconds: u32,
    },
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
    let searching = Arc::new(Mutex::new(false));

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
            "fen" => print_fen(&root),
            _ => println!("Invalid command: {}", input[0]),
        }
    }
}

fn initialize() -> (Vec<UciOption>, Arc<Node>) {
    let mut options = Vec::new();
    options.push(UciOption {
        name: String::from("Threads"),
        value: UciValue::Spin {
            value: 1,
            default: 1,
            min: 1,
            max: 2048,
        },
    });
    options.push(UciOption {
        name: String::from("MultiPV"),
        value: UciValue::Spin {
            value: 1,
            default: 1,
            min: 1,
            max: 256,
        },
    });
    options.push(UciOption {
        name: String::from("Move_Overhead"),
        value: UciValue::Spin {
            value: 100,
            default: 100,
            min: 10,
            max: 5000,
        },
    });
    options.push(UciOption {
        name: String::from("Move_Speed"),
        value: UciValue::Spin {
            value: 50,
            default: 50,
            min: 1,
            max: 100,
        },
    });
    options.push(UciOption {
        name: String::from("MCTS_Explore"),
        value: UciValue::Spin {
            value: 50,
            default: 50,
            min: 1,
            max: 100,
        },
    });
    options.push(UciOption {
        name: String::from("MCTS_Hash"),
        value: UciValue::Spin {
            value: 4096,
            default: 4096,
            min: 16,
            max: 32768,
        },
    });
    options.push(UciOption {
        name: String::from("Skill"),
        value: UciValue::Spin {
            value: 100,
            default: 100,
            min: 1,
            max: 100,
        },
    });
    options.push(UciOption {
        name: String::from("Contempt"),
        value: UciValue::Spin {
            value: 0,
            default: 0,
            min: -100,
            max: 100,
        },
    });
    options.push(UciOption {
        name: String::from("Dynamism"),
        value: UciValue::Spin {
            value: 50,
            default: 50,
            min: 1,
            max: 100,
        },
    });

    let root = Arc::new(Node::new(Board::new(STARTPOS)));

    (options, root)
}

fn uci_uci(options: &Vec<UciOption>) {
    println!("id name Rust_Chess 0.1.0");
    println!("id author Kyle Forrester");

    for option in options.iter() {
        match option.value {
            UciValue::Button => println!("option name {} type button", option.name),
            UciValue::Check { value: _, default } => {
                println!("option name {} type check default {}", option.name, default)
            }
            UciValue::Spin {
                value: _,
                default,
                min,
                max,
            } => println!(
                "option name {} type spin default {} min {} max {}",
                option.name, default, min, max
            ),
        }
    }

    println!("uciok");
}

fn uci_isready() {
    println!("readyok");
}

fn uci_setoption(options: &mut Vec<UciOption>, input: Vec<String>) {
    if input[1] != "name" || input[3] != "value" {
        println!("Unrecognized UCI setoption command");
        return;
    }

    //Find the option the user is trying to modify
    let option = match options.iter_mut().find(|x| x.name == input[2]) {
        Some(o) => o,
        None => {
            println!("Unrecognized UCI setoption command");
            return;
        }
    };

    //Set the option's value to the user input
    match option.value {
        UciValue::Check {
            ref mut value,
            default: _,
        } => match input[4].as_str() {
            "true" => *value = true,
            "false" => *value = false,
            _ => println!("Unrecognized UCI setoption command"),
        },
        UciValue::Spin {
            ref mut value,
            default: _,
            min: _,
            max: _,
        } => match input[4].trim().parse() {
            Ok(v) => *value = v,
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
            }
            "fen" => {
                pos_state = PositionState::Fen;
            }
            "moves" => {
                pos_state = PositionState::Moves;
            }
            _ => match pos_state {
                PositionState::Initial => {
                    println!("Unexpected parameter {}!", token);
                    return root;
                }
                PositionState::StartPos => {
                    println!("Unexpected parameter {}!", token);
                    return root;
                }
                PositionState::Fen => {
                    fen_accumulator.push(token.to_string());
                }
                PositionState::Moves => {
                    moves_accumulator.push(token.to_string());
                }
            },
        }
    }

    if fen_accumulator.len() > 0 {
        fen = fen_accumulator.join(" ");
    }

    //Create the board position
    let mut board = Board::new(&fen);
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

fn uci_go(
    root: &Arc<Node>,
    options: &Vec<UciOption>,
    searching: &Arc<Mutex<bool>>,
    input: Vec<String>,
) {
    let mut s = searching.lock().unwrap();
    *s = true;

    let go_cmd = parse_go_command(input);
    let new_root = Arc::clone(root);
    let new_options = options.clone();
    let new_searching = Arc::clone(searching);
    let new_go_cmd = go_cmd.clone();

    thread::spawn(move || {
        search::search(new_root, new_options, new_searching, new_go_cmd, true);
    });

    let threads = match options.iter().find(|&x| x.name == "Threads").unwrap().value {
        UciValue::Spin {
            value,
            default: _,
            min: _,
            max: _,
        } => value,
        _ => panic!("Threads UCI Option should be a Spin option!"),
    };

    for _i in 0..threads - 1 {
        let new_root = Arc::clone(root);
        let new_options = options.clone();
        let new_searching = Arc::clone(searching);
        let new_go_cmd = go_cmd.clone();
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
        _ => panic!("UCI go command {} not recognized", input[1].as_str()),
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
            _ => match state {
                GoState::WTime => wtime = Some(word.parse().unwrap()),
                GoState::BTime => btime = Some(word.parse().unwrap()),
                GoState::WInc => winc = Some(word.parse().unwrap()),
                GoState::BInc => binc = Some(word.parse().unwrap()),
                GoState::Movestogo => movestogo = Some(word.parse().unwrap()),
                GoState::Initial => panic!("Go command requires arguments!"),
            },
        }
    }

    let go_enum = UciGo::Time {
        wtime: wtime,
        btime: btime,
        winc: winc,
        binc: binc,
        movestogo: movestogo,
    };
    if wtime.is_none() && btime.is_none() {
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
    UciGo::Nodes {
        count: input[2].parse().unwrap(),
    }
}

fn parse_go_movetime(input: Vec<String>) -> UciGo {
    UciGo::Movetime {
        mseconds: input[2].parse().unwrap(),
    }
}

fn parse_go_infinite(_input: Vec<String>) -> UciGo {
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
    io::stdin()
        .read_line(&mut input)
        .expect("Error reading from stdin");
    input
        .split_ascii_whitespace()
        .map(|x| String::from(x))
        .collect()
}

fn print_fen(root: &Arc<Node>) {
    println!("{}", root.board);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(string: &str) -> Vec<String> {
        string
            .split_ascii_whitespace()
            .map(|x| String::from(x))
            .collect()
    }

    fn resolve_fen(cmd: &str) -> String {
        let root = Arc::new(Node::new(Board::new(STARTPOS)));
        let command_vec = tokenize(cmd);
        let root = uci_position(root, command_vec);

        root.board.to_string()
    }

    #[test]
    fn position_en_passent() {
        // White moves en_passent
        assert_eq!(resolve_fen("position fen rnbqkbnr/ppppppp1/7p/4P3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2 moves f7f5"), "rnbqkbnr/ppppp1p1/7p/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
        // Black moves en_passent
        assert_eq!(
            resolve_fen("position startpos moves g1f3 c7c5 h2h3 c5c4 b2b4"),
            "rnbqkbnr/pp1ppppp/8/8/1Pp5/5N1P/P1PPPPP1/RNBQKB1R b KQkq b3 0 3"
        );
        // En_passent on file a
        assert_eq!(
            resolve_fen(
                "position fen rnbqkbnr/1pppp1pp/5p2/pP6/8/8/P1PPPPPP/RNBQKBNR w KQkq a6 0 3"
            ),
            "rnbqkbnr/1pppp1pp/5p2/pP6/8/8/P1PPPPPP/RNBQKBNR w KQkq a6 0 3"
        );
        // En_passent on file a
        assert_eq!(resolve_fen("position fen rnbqkbnr/ppppp1pp/5p2/1P6/8/8/P1PPPPPP/RNBQKBNR b KQkq - 0 2 moves a7a5"), "rnbqkbnr/1pppp1pp/5p2/pP6/8/8/P1PPPPPP/RNBQKBNR w KQkq a6 0 3");
        // En_passent on file h
        assert_eq!(resolve_fen("position fen rnbqkbnr/pppppp1p/8/8/6p1/1P1P4/P1P1PPPP/RNBQKBNR w KQkq - 0 3 moves h2h4"), "rnbqkbnr/pppppp1p/8/8/6pP/1P1P4/P1P1PPP1/RNBQKBNR b KQkq h3 0 3");
    }

    #[test]
    fn position_castling() {
        // White kingside castle
        assert_eq!(resolve_fen("position fen rnb1kb1r/ppppqppp/5n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4 moves e1g1"), "rnb1kb1r/ppppqppp/5n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 5 4");
        // White queenside castle
        assert_eq!(resolve_fen("position fen rnbqk2r/ppp2ppp/4pn2/3p4/1b1P4/2NQB3/PPP1PPPP/R3KBNR w KQkq - 2 5 moves e1c1"), "rnbqk2r/ppp2ppp/4pn2/3p4/1b1P4/2NQB3/PPP1PPPP/2KR1BNR b kq - 3 5");
        // Black queenside castle
        assert_eq!(resolve_fen("position fen r3kb1r/pbppqppp/1pn2n2/4p1B1/2B1P3/3P1N1P/PPP2PP1/RN1Q1RK1 b kq - 2 7 moves e8c8"), "2kr1b1r/pbppqppp/1pn2n2/4p1B1/2B1P3/3P1N1P/PPP2PP1/RN1Q1RK1 w - - 3 8");
        // Black kingside castle
        assert_eq!(resolve_fen("position fen rnbqk2r/ppp2ppp/4pn2/3p4/1b1P4/2NQB3/PPP1PPPP/R3KBNR w KQkq - 2 5 moves e1c1 e8g8"), "rnbq1rk1/ppp2ppp/4pn2/3p4/1b1P4/2NQB3/PPP1PPPP/2KR1BNR w - - 4 6");

        // Move white rooks for king and queen castling
        assert_eq!(
            resolve_fen("position startpos moves g1f3 e7e5 h1g1"),
            "rnbqkbnr/pppp1ppp/8/4p3/8/5N2/PPPPPPPP/RNBQKBR1 b Qkq - 1 2"
        );
        assert_eq!(
            resolve_fen("position startpos moves b1c3 e7e5 a1b1"),
            "rnbqkbnr/pppp1ppp/8/4p3/8/2N5/PPPPPPPP/1RBQKBNR b Kkq - 1 2"
        );
        // Capture white rooks for king and queen castling
        assert_eq!(resolve_fen("position fen rn1qkbnr/pbpppppp/1p6/8/8/2NP2P1/PPP1PP1P/R1BQKBNR b KQkq - 0 3 moves b7h1"), "rn1qkbnr/p1pppppp/1p6/8/8/2NP2P1/PPP1PP1P/R1BQKBNb w Qkq - 0 4");
        assert_eq!(resolve_fen("position fen rnbqk1nr/ppppppbp/6p1/8/8/1P4P1/P1PPPPBP/RNBQK1NR b KQkq - 2 3 moves g7a1"), "rnbqk1nr/pppppp1p/6p1/8/8/1P4P1/P1PPPPBP/bNBQK1NR w Kkq - 0 4");
        // Move white king
        assert_eq!(
            resolve_fen("position startpos moves e2e4 e7e5 e1e2"),
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR b kq - 1 2"
        );

        // Move black rooks for king and queen castling
        assert_eq!(
            resolve_fen("position startpos moves e2e4 g8f6 e4e5 h8g8"),
            "rnbqkbr1/pppppppp/5n2/4P3/8/8/PPPP1PPP/RNBQKBNR w KQq - 1 3"
        );
        assert_eq!(
            resolve_fen("position startpos moves e2e4 b8c6 e4e5 a8b8"),
            "1rbqkbnr/pppppppp/2n5/4P3/8/8/PPPP1PPP/RNBQKBNR w KQk - 1 3"
        );
        // Capture black rooks for king and queen castling
        assert_eq!(
            resolve_fen("position startpos moves b2b3 g7g6 c1b2 c7c6 b2h8"),
            "rnbqkbnB/pp1ppp1p/2p3p1/8/8/1P6/P1PPPPPP/RN1QKBNR b KQq - 0 3"
        );
        assert_eq!(
            resolve_fen("position startpos moves g2g3 b7b6 f1g2 e7e5 g2a8"),
            "Bnbqkbnr/p1pp1ppp/1p6/4p3/8/6P1/PPPPPP1P/RNBQK1NR b KQk - 0 3"
        );
        // Move black king
        assert_eq!(
            resolve_fen("position startpos moves e2e4 e7e5 e1e2 e8e7"),
            "rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR w - - 2 3"
        );
    }

    #[test]
    fn position_promotions() {
        // W_N
        assert_eq!(resolve_fen("position fen r1bk1bnr/pppqpPpp/2np4/8/8/8/PPPP1PPP/RNBQKBNR w KQ - 1 5 moves f7g8N"), "r1bk1bNr/pppqp1pp/2np4/8/8/8/PPPP1PPP/RNBQKBNR b KQ - 0 5");
        // W_B
        assert_eq!(resolve_fen("position fen r1bk1bnr/pppqpPpp/2np4/8/8/8/PPPP1PPP/RNBQKBNR w KQ - 1 5 moves f7g8B"), "r1bk1bBr/pppqp1pp/2np4/8/8/8/PPPP1PPP/RNBQKBNR b KQ - 0 5");
        // W_R
        assert_eq!(resolve_fen("position fen r1bk1bnr/pppqpPpp/2np4/8/8/8/PPPP1PPP/RNBQKBNR w KQ - 1 5 moves f7g8R"), "r1bk1bRr/pppqp1pp/2np4/8/8/8/PPPP1PPP/RNBQKBNR b KQ - 0 5");
        // W_Q
        assert_eq!(resolve_fen("position fen r1bk1bnr/pppqpPpp/2np4/8/8/8/PPPP1PPP/RNBQKBNR w KQ - 1 5 moves f7g8Q"), "r1bk1bQr/pppqp1pp/2np4/8/8/8/PPPP1PPP/RNBQKBNR b KQ - 0 5");
        // B_N
        assert_eq!(resolve_fen("position fen rnbqkbnr/pppp1ppp/8/8/8/2NP4/PPPBP1pP/R2QKBNR b KQkq - 1 5 moves g2h1n"), "rnbqkbnr/pppp1ppp/8/8/8/2NP4/PPPBP2P/R2QKBNn w Qkq - 0 6");
        // B_B
        assert_eq!(resolve_fen("position fen rnbqkbnr/pppp1ppp/8/8/8/2NP4/PPPBP1pP/R2QKBNR b KQkq - 1 5 moves g2h1b"), "rnbqkbnr/pppp1ppp/8/8/8/2NP4/PPPBP2P/R2QKBNb w Qkq - 0 6");
        // B_R
        assert_eq!(resolve_fen("position fen rnbqkbnr/pppp1ppp/8/8/8/2NP4/PPPBP1pP/R2QKBNR b KQkq - 1 5 moves g2h1r"), "rnbqkbnr/pppp1ppp/8/8/8/2NP4/PPPBP2P/R2QKBNr w Qkq - 0 6");
        // B_Q
        assert_eq!(resolve_fen("position fen rnbqkbnr/pppp1ppp/8/8/8/2NP4/PPPBP1pP/R2QKBNR b KQkq - 1 5 moves g2h1q"), "rnbqkbnr/pppp1ppp/8/8/8/2NP4/PPPBP2P/R2QKBNq w Qkq - 0 6");
    }

    #[test]
    fn position_captures() {
        //Capture W_P
        assert_eq!(resolve_fen("position fen rnbqkbnr/pppp1ppp/4p3/5P2/8/8/PPPPP1PP/RNBQKBNR b KQkq - 0 2 moves e6f5"), "rnbqkbnr/pppp1ppp/8/5p2/8/8/PPPPP1PP/RNBQKBNR w KQkq - 0 3");
        //Capture W_N
        assert_eq!(
            resolve_fen("position startpos moves e2e4 d7d5 e4d5"),
            "rnbqkbnr/ppp1pppp/8/3P4/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2"
        );
        //Capture W_B
        assert_eq!(
            resolve_fen("position startpos moves g1f3 e7e5 f3g5 d8g5"),
            "rnb1kbnr/pppp1ppp/8/4p1q1/8/8/PPPPPPPP/RNBQKB1R w KQkq - 0 3"
        );
        //Capture W_R
        assert_eq!(
            resolve_fen("position startpos moves a2a4 e7e6 a1a3 f8a3"),
            "rnbqk1nr/pppp1ppp/4p3/8/P7/b7/1PPPPPPP/1NBQKBNR w Kkq - 0 3"
        );
        //Capture W_Q
        assert_eq!(resolve_fen("position fen rnbqkbnr/pppppp1p/6p1/7Q/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 1 2 moves g6h5"), "rnbqkbnr/pppppp1p/8/7p/4P3/8/PPPP1PPP/RNB1KBNR w KQkq - 0 3");
        //Capture B_P
        assert_eq!(resolve_fen("position startpos moves e2e4 e7e5 g1f3 g8f6 d2d3 d7d6 c1g5 c8g4 f1e2 f8e7 e1g1 e8g8 b1c3 b8c6 d1d2 d8d7 a1b1 a8b8 f1e1 f8e8 g1h1 g8h8"), "1r2r2k/pppqbppp/2np1n2/4p1B1/4P1b1/2NP1N2/PPPQBPPP/1R2R2K w - - 16 12");
        //Capture B_N
        assert_eq!(resolve_fen("position fen rnbqkb1r/ppp1pppp/3p1n2/4P3/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3 moves e5f6"), "rnbqkb1r/ppp1pppp/3p1P2/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 3");
        //Capture B_B
        assert_eq!(
            resolve_fen("position startpos moves a2a4 e7e6 a1a3 f8a3 b2a3"),
            "rnbqk1nr/pppp1ppp/4p3/8/P7/P7/2PPPPPP/1NBQKBNR b Kkq - 0 3"
        );
        //Capture B_R
        assert_eq!(resolve_fen("position fen rnbqkb1r/ppp1pppp/3p1n2/4P3/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3 moves e5f6 d8d7 f6g7 e8d8 g7h8Q"), "rnbk1b1Q/pppqpp1p/3p4/8/8/8/PPPP1PPP/RNBQKBNR b KQ - 0 5");
        //Capture B_Q
        assert_eq!(resolve_fen("position fen rnbqkb1r/ppp1pppp/3p1n2/4P3/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3 moves e5f6 d8d7 f6g7 e8d8 g7h8Q d7h3 g1h3"), "rnbk1b1Q/ppp1pp1p/3p4/8/8/7N/PPPP1PPP/RNBQKB1R b KQ - 0 6");
    }

    #[test]
    fn position() {
        // Starting position
        assert_eq!(
            resolve_fen("position startpos"),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }
}
