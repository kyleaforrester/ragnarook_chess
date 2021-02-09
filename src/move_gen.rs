use std::num::Wrapping;
use std::sync::{Arc, RwLockWriteGuard};

use crate::board::{self, Board, PieceType};
use crate::magic;
use crate::search::{self, Node};

const a_file_bb: u64 = 0x0101010101010101;
const b_file_bb: u64 = 0x0202020202020202;
const c_file_bb: u64 = 0x0404040404040404;
const d_file_bb: u64 = 0x0808080808080808;
const e_file_bb: u64 = 0x1010101010101010;
const f_file_bb: u64 = 0x2020202020202020;
const g_file_bb: u64 = 0x4040404040404040;
const h_file_bb: u64 = 0x8080808080808080;
const rank_1_bb: u64 = 0x00000000000000ff;
const rank_2_bb: u64 = 0x000000000000ff00;
const rank_3_bb: u64 = 0x0000000000ff0000;
const rank_4_bb: u64 = 0x00000000ff000000;
const rank_5_bb: u64 = 0x000000ff00000000;
const rank_6_bb: u64 = 0x0000ff0000000000;
const rank_7_bb: u64 = 0x00ff000000000000;
const rank_8_bb: u64 = 0xff00000000000000;

pub fn bloom(leaf: &Arc<Node>, mut children: RwLockWriteGuard<Vec<Arc<Node>>>) {
    let w_pieces = leaf.board.w_p_bb
        | leaf.board.w_n_bb
        | leaf.board.w_b_bb
        | leaf.board.w_r_bb
        | leaf.board.w_q_bb
        | leaf.board.w_k_bb;
    let b_pieces = leaf.board.b_p_bb
        | leaf.board.b_n_bb
        | leaf.board.b_b_bb
        | leaf.board.b_r_bb
        | leaf.board.b_q_bb
        | leaf.board.b_k_bb;

    children.extend(gen_pawn_moves(leaf, w_pieces, b_pieces));
    children.extend(gen_knight_moves(leaf, w_pieces, b_pieces));
    children.extend(gen_bishop_moves(leaf, w_pieces, b_pieces));
    children.extend(gen_rook_moves(leaf, w_pieces, b_pieces));
    children.extend(gen_queen_moves(leaf, w_pieces, b_pieces));
    children.extend(gen_king_moves(leaf, w_pieces, b_pieces));
}

fn get_piecetype(board: &Board, bb: u64) -> Option<PieceType> {
    if board.w_p_bb & bb > 0 {
        Some(PieceType::WP)
    } else if board.w_n_bb & bb > 0 {
        Some(PieceType::WN)
    } else if board.w_b_bb & bb > 0 {
        Some(PieceType::WB)
    } else if board.w_r_bb & bb > 0 {
        Some(PieceType::WR)
    } else if board.w_q_bb & bb > 0 {
        Some(PieceType::WQ)
    } else if board.w_k_bb & bb > 0 {
        Some(PieceType::WK)
    } else if board.b_p_bb & bb > 0 {
        Some(PieceType::BP)
    } else if board.b_n_bb & bb > 0 {
        Some(PieceType::BN)
    } else if board.b_b_bb & bb > 0 {
        Some(PieceType::BB)
    } else if board.b_r_bb & bb > 0 {
        Some(PieceType::BR)
    } else if board.b_q_bb & bb > 0 {
        Some(PieceType::BQ)
    } else if board.b_k_bb & bb > 0 {
        Some(PieceType::BK)
    } else {
        None
    }
}

fn notate(from: u64, to: u64, promotion: Option<&PieceType>) -> String {
    let from_pos = u64::trailing_zeros(from);
    let to_pos = u64::trailing_zeros(to);

    let from_row = from_pos / 8 + 1;
    let from_col = match from_pos % 8 {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => panic!("Invalid column."),
    };
    let to_row = to_pos / 8 + 1;
    let to_col = match to_pos % 8 {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => panic!("Invalid column."),
    };

    match promotion {
        Some(p) => {
            let c = match p {
                &PieceType::WN | &PieceType::BN => 'n',
                &PieceType::WB | &PieceType::BB => 'b',
                &PieceType::WR | &PieceType::BR => 'r',
                &PieceType::WQ | &PieceType::BQ => 'q',
                _ => panic!("Not a valid promotion type!"),
            };
            format!("{}{}{}{}{}", from_col, from_row, to_col, to_row, c)
        }
        None => format!("{}{}{}{}", from_col, from_row, to_col, to_row),
    }
}

fn gen_pawn_moves(leaf: &Arc<Node>, w_pieces: u64, b_pieces: u64) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.board.is_w_move {
        let mut p_bb = board.w_p_bb;
        while p_bb.count_ones() > 0 {
            // Gets bitboard with only lsb set
            let lsb_p_bb = p_bb & (!p_bb + 1);

            // Promotion possibilities
            let promotions = if lsb_p_bb & rank_7_bb > 0 {
                vec![PieceType::WN, PieceType::WB, PieceType::WR, PieceType::WQ]
            } else {
                vec![PieceType::WP]
            };

            let one_ahead_bb = lsb_p_bb << 8;
            let capture_bbs = if lsb_p_bb & a_file_bb > 0 {
                vec![lsb_p_bb << 9]
            } else if lsb_p_bb & h_file_bb > 0 {
                vec![lsb_p_bb << 7]
            } else {
                vec![lsb_p_bb << 7, lsb_p_bb << 9]
            };
            for promotion in promotions.iter() {
                // Move ahead one square
                if one_ahead_bb & all_pieces == 0 {
                    board.w_p_bb &= !lsb_p_bb;
                    match promotion {
                        &PieceType::WP => board.w_p_bb |= one_ahead_bb,
                        &PieceType::WN => board.w_n_bb |= one_ahead_bb,
                        &PieceType::WB => board.w_b_bb |= one_ahead_bb,
                        &PieceType::WR => board.w_r_bb |= one_ahead_bb,
                        &PieceType::WQ => board.w_q_bb |= one_ahead_bb,
                        _ => panic!("Invalid promotion type for WP"),
                    }

                    board.halfmove_clock = 0;
                    board.en_passent = None;

                    //King cannot be in check
                    if !is_attacked(&board, false, board.w_k_bb) {
                        let last_move = if lsb_p_bb & rank_7_bb > 0 {
                            notate(lsb_p_bb, one_ahead_bb, Some(promotion))
                        } else {
                            notate(lsb_p_bb, one_ahead_bb, None)
                        };
                        children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                    }
                    board = leaf.board;
                }

                // Move ahead two squares
                if lsb_p_bb & rank_2_bb > 0
                    && one_ahead_bb & all_pieces == 0
                    && (lsb_p_bb << 16) & all_pieces == 0
                {
                    board.w_p_bb &= !lsb_p_bb;
                    board.w_p_bb |= lsb_p_bb << 16;
                    board.halfmove_clock = 0;
                    board.en_passent = None;

                    // Check for en_passent enablement
                    let enemy_ep_pawns = if lsb_p_bb & a_file_bb > 0 {
                        vec![lsb_p_bb << 17]
                    } else if lsb_p_bb & h_file_bb > 0 {
                        vec![lsb_p_bb << 15]
                    } else {
                        vec![lsb_p_bb << 15, lsb_p_bb << 17]
                    };
                    for enemy_ep_pawn in enemy_ep_pawns {
                        if board.b_p_bb & enemy_ep_pawn > 0 {
                            board.en_passent = Some(lsb_p_bb << 8);
                        }
                    }

                    //King cannot be in check
                    if !is_attacked(&board, false, board.w_k_bb) {
                        let last_move = notate(lsb_p_bb, lsb_p_bb << 16, None);
                        children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                    }
                    board = leaf.board;
                }

                // Check for captures
                for capture_bb in capture_bbs.iter() {
                    let captured_piece = get_piecetype(&board, capture_bb & b_pieces);
                    if captured_piece.is_some() {
                        board.w_p_bb &= !lsb_p_bb;
                        match promotion {
                            &PieceType::WP => board.w_p_bb |= capture_bb,
                            &PieceType::WN => board.w_n_bb |= capture_bb,
                            &PieceType::WB => board.w_b_bb |= capture_bb,
                            &PieceType::WR => board.w_r_bb |= capture_bb,
                            &PieceType::WQ => board.w_q_bb |= capture_bb,
                            _ => panic!("Invalid promotion type for WP"),
                        }
                        let captured_pt = captured_piece.unwrap();
                        match captured_pt {
                            PieceType::BP => board.b_p_bb &= !capture_bb,
                            PieceType::BN => board.b_n_bb &= !capture_bb,
                            PieceType::BB => board.b_b_bb &= !capture_bb,
                            PieceType::BR => board.b_r_bb &= !capture_bb,
                            PieceType::BQ => board.b_q_bb &= !capture_bb,
                            _ => panic!("Invalid capture PieceType for WP"),
                        }

                        board.halfmove_clock = 0;
                        board.en_passent = None;

                        //King cannot be in check
                        if !is_attacked(&board, false, board.w_k_bb) {
                            let last_move = if lsb_p_bb & rank_7_bb > 0 {
                                notate(lsb_p_bb, *capture_bb, Some(promotion))
                            } else {
                                notate(lsb_p_bb, *capture_bb, None)
                            };
                            children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                        }
                        board = leaf.board;
                    }
                }

                // Check for en-passent
                match board.en_passent {
                    Some(ep_bb) => {
                        for capture_bb in capture_bbs.iter() {
                            if capture_bb & ep_bb > 0 {
                                board.w_p_bb &= !lsb_p_bb;
                                board.w_p_bb |= ep_bb;
                                board.b_p_bb &= !(ep_bb >> 8);
                                board.en_passent = None;
                                board.halfmove_clock = 0;

                                //King cannot be in check
                                if !is_attacked(&board, false, board.w_k_bb) {
                                    let last_move = notate(lsb_p_bb, ep_bb, None);
                                    children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                                }
                                board = leaf.board;
                            }
                        }
                    }
                    None => (),
                }
            }

            // Removes LSB of bitboard
            p_bb &= p_bb - 1;
        }
    } else {
        // Black to move
        let mut p_bb = board.b_p_bb;
        while p_bb.count_ones() > 0 {
            // Gets bitboard with only lsb set
            let lsb_p_bb = p_bb & (!p_bb + 1);

            // Promotion possibilities
            let promotions = if lsb_p_bb & rank_2_bb > 0 {
                vec![PieceType::BN, PieceType::BB, PieceType::BR, PieceType::BQ]
            } else {
                vec![PieceType::BP]
            };

            let one_ahead_bb = lsb_p_bb >> 8;
            let capture_bbs = if lsb_p_bb & a_file_bb > 0 {
                vec![lsb_p_bb >> 7]
            } else if lsb_p_bb & h_file_bb > 0 {
                vec![lsb_p_bb >> 9]
            } else {
                vec![lsb_p_bb >> 7, lsb_p_bb >> 9]
            };
            for promotion in promotions.iter() {
                // Move ahead one square
                if one_ahead_bb & all_pieces == 0 {
                    board.b_p_bb &= !lsb_p_bb;
                    match promotion {
                        &PieceType::BP => board.b_p_bb |= one_ahead_bb,
                        &PieceType::BN => board.b_n_bb |= one_ahead_bb,
                        &PieceType::BB => board.b_b_bb |= one_ahead_bb,
                        &PieceType::BR => board.b_r_bb |= one_ahead_bb,
                        &PieceType::BQ => board.b_q_bb |= one_ahead_bb,
                        _ => panic!("Invalid promotion type for BP"),
                    }

                    board.halfmove_clock = 0;
                    board.en_passent = None;

                    //King cannot be in check
                    if !is_attacked(&board, true, board.b_k_bb) {
                        let last_move = if lsb_p_bb & rank_2_bb > 0 {
                            notate(lsb_p_bb, one_ahead_bb, Some(promotion))
                        } else {
                            notate(lsb_p_bb, one_ahead_bb, None)
                        };
                        children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                    }
                    board = leaf.board;
                }

                // Move ahead two squares
                if lsb_p_bb & rank_7_bb > 0
                    && one_ahead_bb & all_pieces == 0
                    && (lsb_p_bb >> 16) & all_pieces == 0
                {
                    board.b_p_bb &= !lsb_p_bb;
                    board.b_p_bb |= lsb_p_bb >> 16;
                    board.halfmove_clock = 0;
                    board.en_passent = None;

                    // Check for en_passent enablement
                    let enemy_ep_pawns = if lsb_p_bb & a_file_bb > 0 {
                        vec![lsb_p_bb >> 15]
                    } else if lsb_p_bb & h_file_bb > 0 {
                        vec![lsb_p_bb >> 17]
                    } else {
                        vec![lsb_p_bb >> 15, lsb_p_bb >> 17]
                    };
                    for enemy_ep_pawn in enemy_ep_pawns {
                        if board.w_p_bb & enemy_ep_pawn > 0 {
                            board.en_passent = Some(lsb_p_bb >> 8);
                        }
                    }

                    //King cannot be in check
                    if !is_attacked(&board, true, board.b_k_bb) {
                        let last_move = notate(lsb_p_bb, lsb_p_bb >> 16, None);
                        children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                    }
                    board = leaf.board;
                }

                // Check for captures
                for capture_bb in capture_bbs.iter() {
                    let captured_piece = get_piecetype(&board, capture_bb & w_pieces);
                    if captured_piece.is_some() {
                        board.b_p_bb &= !lsb_p_bb;
                        match promotion {
                            &PieceType::BP => board.b_p_bb |= capture_bb,
                            &PieceType::BN => board.b_n_bb |= capture_bb,
                            &PieceType::BB => board.b_b_bb |= capture_bb,
                            &PieceType::BR => board.b_r_bb |= capture_bb,
                            &PieceType::BQ => board.b_q_bb |= capture_bb,
                            _ => panic!("Invalid promotion type for BP"),
                        }
                        let captured_pt = captured_piece.unwrap();
                        match captured_pt {
                            PieceType::WP => board.w_p_bb &= !capture_bb,
                            PieceType::WN => board.w_n_bb &= !capture_bb,
                            PieceType::WB => board.w_b_bb &= !capture_bb,
                            PieceType::WR => board.w_r_bb &= !capture_bb,
                            PieceType::WQ => board.w_q_bb &= !capture_bb,
                            _ => panic!("Invalid capture PieceType for BP"),
                        }

                        board.halfmove_clock = 0;
                        board.en_passent = None;

                        //King cannot be in check
                        if !is_attacked(&board, true, board.b_k_bb) {
                            let last_move = if lsb_p_bb & rank_2_bb > 0 {
                                notate(lsb_p_bb, *capture_bb, Some(promotion))
                            } else {
                                notate(lsb_p_bb, *capture_bb, None)
                            };
                            children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                        }
                        board = leaf.board;
                    }
                }

                // Check for en-passent
                match board.en_passent {
                    Some(ep_bb) => {
                        for capture_bb in capture_bbs.iter() {
                            if capture_bb & ep_bb > 0 {
                                board.b_p_bb &= !lsb_p_bb;
                                board.b_p_bb |= ep_bb;
                                board.w_p_bb &= !(ep_bb << 8);
                                board.en_passent = None;
                                board.halfmove_clock = 0;

                                //King cannot be in check
                                if !is_attacked(&board, true, board.b_k_bb) {
                                    let last_move = notate(lsb_p_bb, ep_bb, None);
                                    children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                                }
                                board = leaf.board;
                            }
                        }
                    }
                    None => (),
                }
            }

            // Removes LSB of bitboard
            p_bb &= p_bb - 1;
        }
    }

    children
}

fn gen_knight_moves(leaf: &Arc<Node>, w_pieces: u64, b_pieces: u64) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.board.is_w_move {
        let mut n_bb = board.w_n_bb;
        while n_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_n_bb = n_bb & (!n_bb + 1);

            let mut solo_n_moves = solo_knight_moves(lsb_n_bb, w_pieces);

            while solo_n_moves > 0 {
                // Look at next knight move
                let lsb_solo_n_moves = solo_n_moves & (!solo_n_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_n_moves);
                match captured_pt {
                    Some(ref pt) => match pt {
                        PieceType::BP => board.b_p_bb &= !lsb_solo_n_moves,
                        PieceType::BN => board.b_n_bb &= !lsb_solo_n_moves,
                        PieceType::BB => board.b_b_bb &= !lsb_solo_n_moves,
                        PieceType::BR => board.b_r_bb &= !lsb_solo_n_moves,
                        PieceType::BQ => board.b_q_bb &= !lsb_solo_n_moves,
                        _ => panic!(format!(
                            "Internal error: white knight cannot capture {}!",
                            pt
                        )),
                    },
                    None => (),
                }

                // Move the knight
                board.w_n_bb |= lsb_solo_n_moves;
                board.w_n_bb &= !lsb_n_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, false, board.w_k_bb) {
                    let last_move = notate(lsb_n_bb, lsb_solo_n_moves, None);
                    children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                }
                board = leaf.board;

                // Remove LSB on bitboard
                solo_n_moves &= !lsb_solo_n_moves;
            }

            // Remove LSB on bitboard
            n_bb &= !lsb_n_bb;
        }
    } else {
        // We are black
        let mut n_bb = board.b_n_bb;
        while n_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_n_bb = n_bb & (!n_bb + 1);

            let mut solo_n_moves = solo_knight_moves(lsb_n_bb, b_pieces);

            while solo_n_moves > 0 {
                // Look at next knight move
                let lsb_solo_n_moves = solo_n_moves & (!solo_n_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_n_moves);
                match captured_pt {
                    Some(ref pt) => match pt {
                        PieceType::WP => board.w_p_bb &= !lsb_solo_n_moves,
                        PieceType::WN => board.w_n_bb &= !lsb_solo_n_moves,
                        PieceType::WB => board.w_b_bb &= !lsb_solo_n_moves,
                        PieceType::WR => board.w_r_bb &= !lsb_solo_n_moves,
                        PieceType::WQ => board.w_q_bb &= !lsb_solo_n_moves,
                        _ => panic!(format!(
                            "Internal error: black knight cannot capture {}!",
                            pt
                        )),
                    },
                    None => (),
                }

                // Move the knight
                board.b_n_bb |= lsb_solo_n_moves;
                board.b_n_bb &= !lsb_n_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, true, board.b_k_bb) {
                    let last_move = notate(lsb_n_bb, lsb_solo_n_moves, None);
                    children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                }
                board = leaf.board;

                // Remove LSB on bitboard
                solo_n_moves &= !lsb_solo_n_moves;
            }

            // Remove LSB on bitboard
            n_bb &= !lsb_n_bb;
        }
    }

    children
}

fn gen_bishop_moves(leaf: &Arc<Node>, w_pieces: u64, b_pieces: u64) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.board.is_w_move {
        let mut b_bb = board.w_b_bb;
        while b_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_b_bb = b_bb & (!b_bb + 1);

            let mut solo_b_moves = solo_bishop_moves(lsb_b_bb, w_pieces, all_pieces);

            while solo_b_moves > 0 {
                // Look at next bishop move
                let lsb_solo_b_moves = solo_b_moves & (!solo_b_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_b_moves);
                match captured_pt {
                    Some(ref pt) => match pt {
                        PieceType::BP => board.b_p_bb &= !lsb_solo_b_moves,
                        PieceType::BN => board.b_n_bb &= !lsb_solo_b_moves,
                        PieceType::BB => board.b_b_bb &= !lsb_solo_b_moves,
                        PieceType::BR => board.b_r_bb &= !lsb_solo_b_moves,
                        PieceType::BQ => board.b_q_bb &= !lsb_solo_b_moves,
                        _ => panic!(format!(
                            "Internal error: white bishop cannot capture {}!",
                            pt
                        )),
                    },
                    None => (),
                }

                // Move the bishop
                board.w_b_bb |= lsb_solo_b_moves;
                board.w_b_bb &= !lsb_b_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, false, board.w_k_bb) {
                    let last_move = notate(lsb_b_bb, lsb_solo_b_moves, None);
                    children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                }
                board = leaf.board;

                // Remove LSB on bitboard
                solo_b_moves &= !lsb_solo_b_moves;
            }

            // Remove LSB on bitboard
            b_bb &= !lsb_b_bb;
        }
    } else {
        // We are black
        let mut b_bb = board.b_b_bb;
        while b_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_b_bb = b_bb & (!b_bb + 1);

            let mut solo_b_moves = solo_bishop_moves(lsb_b_bb, b_pieces, all_pieces);

            while solo_b_moves > 0 {
                // Look at next knight move
                let lsb_solo_b_moves = solo_b_moves & (!solo_b_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_b_moves);
                match captured_pt {
                    Some(ref pt) => match pt {
                        PieceType::WP => board.w_p_bb &= !lsb_solo_b_moves,
                        PieceType::WN => board.w_n_bb &= !lsb_solo_b_moves,
                        PieceType::WB => board.w_b_bb &= !lsb_solo_b_moves,
                        PieceType::WR => board.w_r_bb &= !lsb_solo_b_moves,
                        PieceType::WQ => board.w_q_bb &= !lsb_solo_b_moves,
                        _ => panic!(format!(
                            "Internal error: black bishop cannot capture {}!",
                            pt
                        )),
                    },
                    None => (),
                }

                // Move the knight
                board.b_b_bb |= lsb_solo_b_moves;
                board.b_b_bb &= !lsb_b_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, true, board.b_k_bb) {
                    let last_move = notate(lsb_b_bb, lsb_solo_b_moves, None);
                    children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                }
                board = leaf.board;

                // Remove LSB on bitboard
                solo_b_moves &= !lsb_solo_b_moves;
            }

            // Remove LSB on bitboard
            b_bb &= !lsb_b_bb;
        }
    }

    children
}

fn gen_rook_moves(leaf: &Arc<Node>, w_pieces: u64, b_pieces: u64) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.board.is_w_move {
        let mut r_bb = board.w_r_bb;
        while r_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_r_bb = r_bb & (!r_bb + 1);

            let mut solo_r_moves = solo_rook_moves(lsb_r_bb, w_pieces, all_pieces);

            while solo_r_moves > 0 {
                // Look at next rook move
                let lsb_solo_r_moves = solo_r_moves & (!solo_r_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_r_moves);
                match captured_pt {
                    Some(ref pt) => match pt {
                        PieceType::BP => board.b_p_bb &= !lsb_solo_r_moves,
                        PieceType::BN => board.b_n_bb &= !lsb_solo_r_moves,
                        PieceType::BB => board.b_b_bb &= !lsb_solo_r_moves,
                        PieceType::BR => board.b_r_bb &= !lsb_solo_r_moves,
                        PieceType::BQ => board.b_q_bb &= !lsb_solo_r_moves,
                        _ => panic!(format!("Internal error: white rook cannot capture {}!", pt)),
                    },
                    None => (),
                }

                // Move the rook
                board.w_r_bb |= lsb_solo_r_moves;
                board.w_r_bb &= !lsb_r_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }
                if lsb_r_bb & 0x1 > 0 {
                    board.is_w_q_castle = false;
                } else if lsb_r_bb & 0x80 > 0 {
                    board.is_w_castle = false;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, false, board.w_k_bb) {
                    let last_move = notate(lsb_r_bb, lsb_solo_r_moves, None);
                    children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                }
                board = leaf.board;

                // Remove LSB on bitboard
                solo_r_moves &= !lsb_solo_r_moves;
            }

            // Remove LSB on bitboard
            r_bb &= !lsb_r_bb;
        }
    } else {
        // We are black
        let mut r_bb = board.b_r_bb;
        while r_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_r_bb = r_bb & (!r_bb + 1);

            let mut solo_r_moves = solo_rook_moves(lsb_r_bb, b_pieces, all_pieces);

            while solo_r_moves > 0 {
                // Look at next rook move
                let lsb_solo_r_moves = solo_r_moves & (!solo_r_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_r_moves);
                match captured_pt {
                    Some(ref pt) => match pt {
                        PieceType::WP => board.w_p_bb &= !lsb_solo_r_moves,
                        PieceType::WN => board.w_n_bb &= !lsb_solo_r_moves,
                        PieceType::WB => board.w_b_bb &= !lsb_solo_r_moves,
                        PieceType::WR => board.w_r_bb &= !lsb_solo_r_moves,
                        PieceType::WQ => board.w_q_bb &= !lsb_solo_r_moves,
                        _ => panic!(format!("Internal error: black rook cannot capture {}!", pt)),
                    },
                    None => (),
                }

                // Move the rook
                board.b_r_bb |= lsb_solo_r_moves;
                board.b_r_bb &= !lsb_r_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }
                if lsb_r_bb & 0x100000000000000 > 0 {
                    board.is_b_q_castle = false;
                } else if lsb_r_bb & 0x8000000000000000 > 0 {
                    board.is_b_castle = false;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, true, board.b_k_bb) {
                    let last_move = notate(lsb_r_bb, lsb_solo_r_moves, None);
                    children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                }
                board = leaf.board;

                // Remove LSB on bitboard
                solo_r_moves &= !lsb_solo_r_moves;
            }

            // Remove LSB on bitboard
            r_bb &= !lsb_r_bb;
        }
    }

    children
}

fn gen_queen_moves(leaf: &Arc<Node>, w_pieces: u64, b_pieces: u64) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.board.is_w_move {
        let mut q_bb = board.w_q_bb;
        while q_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_q_bb = q_bb & (!q_bb + 1);

            let mut solo_q_moves = solo_rook_moves(lsb_q_bb, w_pieces, all_pieces)
                | solo_bishop_moves(lsb_q_bb, w_pieces, all_pieces);

            while solo_q_moves > 0 {
                // Look at next queen move
                let lsb_solo_q_moves = solo_q_moves & (!solo_q_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_q_moves);
                match captured_pt {
                    Some(ref pt) => match pt {
                        PieceType::BP => board.b_p_bb &= !lsb_solo_q_moves,
                        PieceType::BN => board.b_n_bb &= !lsb_solo_q_moves,
                        PieceType::BB => board.b_b_bb &= !lsb_solo_q_moves,
                        PieceType::BR => board.b_r_bb &= !lsb_solo_q_moves,
                        PieceType::BQ => board.b_q_bb &= !lsb_solo_q_moves,
                        _ => panic!(format!(
                            "Internal error: white queen cannot capture {}!",
                            pt
                        )),
                    },
                    None => (),
                }

                // Move the queen
                board.w_q_bb |= lsb_solo_q_moves;
                board.w_q_bb &= !lsb_q_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, false, board.w_k_bb) {
                    let last_move = notate(lsb_q_bb, lsb_solo_q_moves, None);
                    children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                }
                board = leaf.board;

                // Remove LSB on bitboard
                solo_q_moves &= !lsb_solo_q_moves;
            }

            // Remove LSB on bitboard
            q_bb &= !lsb_q_bb;
        }
    } else {
        // We are black
        let mut q_bb = board.b_q_bb;
        while q_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_q_bb = q_bb & (!q_bb + 1);

            let mut solo_q_moves = solo_rook_moves(lsb_q_bb, b_pieces, all_pieces)
                | solo_bishop_moves(lsb_q_bb, b_pieces, all_pieces);

            while solo_q_moves > 0 {
                // Look at next queen move
                let lsb_solo_q_moves = solo_q_moves & (!solo_q_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_q_moves);
                match captured_pt {
                    Some(ref pt) => match pt {
                        PieceType::WP => board.w_p_bb &= !lsb_solo_q_moves,
                        PieceType::WN => board.w_n_bb &= !lsb_solo_q_moves,
                        PieceType::WB => board.w_b_bb &= !lsb_solo_q_moves,
                        PieceType::WR => board.w_r_bb &= !lsb_solo_q_moves,
                        PieceType::WQ => board.w_q_bb &= !lsb_solo_q_moves,
                        _ => panic!(format!(
                            "Internal error: black queen cannot capture {}!",
                            pt
                        )),
                    },
                    None => (),
                }

                // Move the queen
                board.b_q_bb |= lsb_solo_q_moves;
                board.b_q_bb &= !lsb_q_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, true, board.b_k_bb) {
                    let last_move = notate(lsb_q_bb, lsb_solo_q_moves, None);
                    children.push(Arc::new(Node::spawn(leaf, board, last_move)));
                }
                board = leaf.board;

                // Remove LSB on bitboard
                solo_q_moves &= !lsb_solo_q_moves;
            }

            // Remove LSB on bitboard
            q_bb &= !lsb_q_bb;
        }
    }

    children
}

fn gen_king_moves(leaf: &Arc<Node>, w_pieces: u64, b_pieces: u64) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.board.is_w_move {
        // Castling moves
        // Kingside
        if board.is_w_castle
            && w_pieces & 0x60 == 0
            && !is_attacked(&board, false, board.w_k_bb)
            && !is_attacked(&board, false, 0x20)
            && !is_attacked(&board, false, 0x40)
        {
            // Move rook
            board.w_r_bb &= !0x80;
            board.w_r_bb |= 0x20;

            // Move king
            board.w_k_bb = 0x40;

            // Other board changes
            board.is_w_castle = false;
            board.is_w_q_castle = false;
            board.en_passent = None;
            board.halfmove_clock += 1;

            children.push(Arc::new(Node::spawn(leaf, board, "e1g1".to_string())));
            board = leaf.board;
        }
        // Queenside
        if board.is_w_q_castle
            && w_pieces & 0xe == 0
            && !is_attacked(&board, false, board.w_k_bb)
            && !is_attacked(&board, false, 0x4)
            && !is_attacked(&board, false, 0x8)
        {
            // Move rook
            board.w_r_bb &= !0x1;
            board.w_r_bb |= 0x8;

            // Move king
            board.w_k_bb = 0x4;

            // Other board changes
            board.is_w_castle = false;
            board.is_w_q_castle = false;
            board.en_passent = None;
            board.halfmove_clock += 1;

            children.push(Arc::new(Node::spawn(leaf, board, "e1c1".to_string())));
            board = leaf.board;
        }

        // Standard moves
        let k_bb = board.w_k_bb;

        let mut solo_k_moves = solo_king_moves(k_bb, w_pieces);

        while solo_k_moves > 0 {
            // Look at next king move
            let lsb_solo_k_moves = solo_k_moves & (!solo_k_moves + 1);

            // Strip all enemies away from landing square
            let captured_pt = get_piecetype(&board, lsb_solo_k_moves);
            match captured_pt {
                Some(ref pt) => match pt {
                    PieceType::BP => board.b_p_bb &= !lsb_solo_k_moves,
                    PieceType::BN => board.b_n_bb &= !lsb_solo_k_moves,
                    PieceType::BB => board.b_b_bb &= !lsb_solo_k_moves,
                    PieceType::BR => board.b_r_bb &= !lsb_solo_k_moves,
                    PieceType::BQ => board.b_q_bb &= !lsb_solo_k_moves,
                    _ => panic!(format!("Internal error: white king cannot capture {}!", pt)),
                },
                None => (),
            }

            // Move the king
            board.w_k_bb |= lsb_solo_k_moves;
            board.w_k_bb &= !k_bb;

            // Remove castling rights
            board.is_w_castle = false;
            board.is_w_q_castle = false;

            // Update other board fields
            board.en_passent = None;
            if captured_pt.is_some() {
                board.halfmove_clock = 0;
            } else {
                board.halfmove_clock += 1;
            }

            // If not in check then add to next moves
            if !is_attacked(&board, false, board.w_k_bb) {
                let last_move = notate(k_bb, lsb_solo_k_moves, None);
                children.push(Arc::new(Node::spawn(leaf, board, last_move)));
            }
            board = leaf.board;

            // Remove LSB on bitboard
            solo_k_moves &= !lsb_solo_k_moves;
        }
    } else {
        // We are black
        // Castling moves
        // Kingside
        if board.is_b_castle
            && b_pieces & 0x6000000000000000 == 0
            && !is_attacked(&board, true, board.b_k_bb)
            && !is_attacked(&board, true, 0x2000000000000000)
            && !is_attacked(&board, true, 0x4000000000000000)
        {
            // Move rook
            board.b_r_bb &= !0x8000000000000000;
            board.b_r_bb |= 0x2000000000000000;

            // Move king
            board.b_k_bb = 0x4000000000000000;

            // Other board changes
            board.is_b_castle = false;
            board.is_b_q_castle = false;
            board.en_passent = None;
            board.halfmove_clock += 1;

            children.push(Arc::new(Node::spawn(leaf, board, "e8g8".to_string())));
            board = leaf.board;
        }
        // Queenside
        if board.is_b_q_castle
            && b_pieces & 0xe00000000000000 == 0
            && !is_attacked(&board, true, board.b_k_bb)
            && !is_attacked(&board, true, 0x400000000000000)
            && !is_attacked(&board, true, 0x800000000000000)
        {
            // Move rook
            board.b_r_bb &= !0x100000000000000;
            board.b_r_bb |= 0x800000000000000;

            // Move king
            board.b_k_bb = 0x400000000000000;

            // Other board changes
            board.is_b_castle = false;
            board.is_b_q_castle = false;
            board.en_passent = None;
            board.halfmove_clock += 1;

            children.push(Arc::new(Node::spawn(leaf, board, "e8c8".to_string())));
            board = leaf.board;
        }

        // Standard moves
        let k_bb = board.b_k_bb;

        let mut solo_k_moves = solo_king_moves(k_bb, b_pieces);

        while solo_k_moves > 0 {
            // Look at next king move
            let lsb_solo_k_moves = solo_k_moves & (!solo_k_moves + 1);

            // Strip all enemies away from landing square
            let captured_pt = get_piecetype(&board, lsb_solo_k_moves);
            match captured_pt {
                Some(ref pt) => match pt {
                    PieceType::WP => board.w_p_bb &= !lsb_solo_k_moves,
                    PieceType::WN => board.w_n_bb &= !lsb_solo_k_moves,
                    PieceType::WB => board.w_b_bb &= !lsb_solo_k_moves,
                    PieceType::WR => board.w_r_bb &= !lsb_solo_k_moves,
                    PieceType::WQ => board.w_q_bb &= !lsb_solo_k_moves,
                    _ => panic!(format!("Internal error: black king cannot capture {}!", pt)),
                },
                None => (),
            }

            // Move the king
            board.b_k_bb |= lsb_solo_k_moves;
            board.b_k_bb &= !k_bb;

            // Remove castling rights
            board.is_b_castle = false;
            board.is_b_q_castle = false;

            // Update other board fields
            board.en_passent = None;
            if captured_pt.is_some() {
                board.halfmove_clock = 0;
            } else {
                board.halfmove_clock += 1;
            }

            // If not in check then add to next moves
            if !is_attacked(&board, true, board.b_k_bb) {
                let last_move = notate(k_bb, lsb_solo_k_moves, None);
                children.push(Arc::new(Node::spawn(leaf, board, last_move)));
            }
            board = leaf.board;

            // Remove LSB on bitboard
            solo_k_moves &= !lsb_solo_k_moves;
        }
    }

    children
}

fn is_attacked(board: &Board, by_white: bool, bb: u64) -> bool {
    if !by_white {
        // We are white
        let ally_pieces =
            board.w_p_bb | board.w_n_bb | board.w_b_bb | board.w_r_bb | board.w_q_bb | board.w_k_bb;
        let enemy_pieces =
            board.b_p_bb | board.b_n_bb | board.b_b_bb | board.b_r_bb | board.b_q_bb | board.b_k_bb;
        let all_pieces = ally_pieces | enemy_pieces;

        // Check for knight attacks
        if solo_knight_moves(bb, ally_pieces) & board.b_n_bb > 0 {
            return true;
        }
        // Check for bishop attacks
        if solo_bishop_moves(bb, ally_pieces, all_pieces) & (board.b_b_bb | board.b_q_bb) > 0 {
            return true;
        }
        // Check for rook attacks
        if solo_rook_moves(bb, ally_pieces, all_pieces) & (board.b_r_bb | board.b_q_bb) > 0 {
            return true;
        }
        // Check for king attacks
        if solo_king_moves(bb, ally_pieces) & board.b_k_bb > 0 {
            return true;
        }
        // Check for pawn attacks
        if solo_pawn_attacks(bb, board.b_p_bb, true) > 0 {
            return true;
        }
    } else {
        // We are black
        let ally_pieces =
            board.b_p_bb | board.b_n_bb | board.b_b_bb | board.b_r_bb | board.b_q_bb | board.b_k_bb;
        let enemy_pieces =
            board.w_p_bb | board.w_n_bb | board.w_b_bb | board.w_r_bb | board.w_q_bb | board.w_k_bb;
        let all_pieces = ally_pieces | enemy_pieces;

        // Check for knight attacks
        if solo_knight_moves(bb, ally_pieces) & board.w_n_bb > 0 {
            return true;
        }
        // Check for bishop attacks
        if solo_bishop_moves(bb, ally_pieces, all_pieces) & (board.w_b_bb | board.w_q_bb) > 0 {
            return true;
        }
        // Check for rook attacks
        if solo_rook_moves(bb, ally_pieces, all_pieces) & (board.w_r_bb | board.w_q_bb) > 0 {
            return true;
        }
        // Check for king attacks
        if solo_king_moves(bb, ally_pieces) & board.w_k_bb > 0 {
            return true;
        }
        // Check for pawn attacks
        if solo_pawn_attacks(bb, board.w_p_bb, false) > 0 {
            return true;
        }
    }
    false
}

fn solo_knight_moves(bb: u64, ally_pieces: u64) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    magic::knight_collisions[pos] & !ally_pieces
}

fn solo_bishop_moves(bb: u64, ally_pieces: u64, all_pieces: u64) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    let occupied_coll = magic::bishop_collisions[pos] & all_pieces;
    let magic_ind = Wrapping(magic::bishop_magic_numbers[pos]) * Wrapping(occupied_coll) >> 55;
    magic::bishop_magic_move_sets[pos][magic_ind.0 as usize] & !ally_pieces
}

fn solo_rook_moves(bb: u64, ally_pieces: u64, all_pieces: u64) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    let occupied_coll = magic::rook_collisions[pos] & all_pieces;
    let magic_ind = Wrapping(magic::rook_magic_numbers[pos]) * Wrapping(occupied_coll) >> 52;
    magic::rook_magic_move_sets[pos][magic_ind.0 as usize] & !ally_pieces
}

fn solo_king_moves(bb: u64, ally_pieces: u64) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    magic::king_collisions[pos] & !ally_pieces
}

fn solo_pawn_attacks(bb: u64, enemy_pawns: u64, is_white: bool) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    if is_white {
        magic::w_pawn_attack_collisions[pos] & enemy_pawns
    } else {
        magic::b_pawn_attack_collisions[pos] & enemy_pawns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board;
    use crate::search;

    #[test]
    fn test_move_gen() {
        let scenarios = load_scenarios();

        for s in scenarios.iter() {
            validate_scenario(s);
        }
    }

    fn validate_scenario(tup: &(String, Vec<String>)) {
        let node = Arc::new(search::Node::new(board::Board::new(&tup.0)));
        let guard = node.children.try_write().unwrap();
        bloom(&node, guard);

        let children = node.children.read().unwrap();
        let child_fens: Vec<String> = children.iter().map(|x| x.board.to_string()).collect();

        // Make sure we contain all the required moves
        for answer in tup.1.iter() {
            assert!(
                child_fens.contains(answer),
                "Parent Fen:\n{}\nDid not generate child fen:\n{}\nGenerated fens:\n{:?}\n",
                tup.0,
                answer,
                child_fens
            );
        }

        // Make sure we dont have extra moves generated
        assert_eq!(children.len(), tup.1.len());
    }

    fn load_scenarios() -> Vec<(String, Vec<String>)> {
        let mut scenarios = Vec::new();

        scenarios.push((
            "8/8/8/1k5R/p1n5/8/5r2/K1B5 b - - 89 90".to_string(),
            vec![
                "8/8/2k5/7R/p1n5/8/5r2/K1B5 w - - 90 91".to_string(),
                "8/8/1k6/7R/p1n5/8/5r2/K1B5 w - - 90 91".to_string(),
                "8/8/k7/7R/p1n5/8/5r2/K1B5 w - - 90 91".to_string(),
                "8/8/8/7R/pkn5/8/5r2/K1B5 w - - 90 91".to_string(),
                "8/8/8/1k2n2R/p7/8/5r2/K1B5 w - - 90 91".to_string(),
                "8/8/8/1k3r1R/p1n5/8/8/K1B5 w - - 90 91".to_string(),
            ],
        ));
        scenarios.push((
            "4k3/8/4K2P/4N1P1/1P1pN3/8/8/7r w - - 9 65".to_string(),
            vec![
                "4k3/8/5K1P/4N1P1/1P1pN3/8/8/7r b - - 10 65".to_string(),
                "4k3/8/3K3P/4N1P1/1P1pN3/8/8/7r b - - 10 65".to_string(),
                "4k3/8/7P/4NKP1/1P1pN3/8/8/7r b - - 10 65".to_string(),
                "4k3/8/7P/3KN1P1/1P1pN3/8/8/7r b - - 10 65".to_string(),
                "4k3/5N2/4K2P/6P1/1P1pN3/8/8/7r b - - 10 65".to_string(),
                "4k3/3N4/4K2P/6P1/1P1pN3/8/8/7r b - - 10 65".to_string(),
                "4k3/8/4K1NP/6P1/1P1pN3/8/8/7r b - - 10 65".to_string(),
                "4k3/8/2N1K2P/6P1/1P1pN3/8/8/7r b - - 10 65".to_string(),
                "4k3/8/4K2P/6P1/1P1pN1N1/8/8/7r b - - 10 65".to_string(),
                "4k3/8/4K2P/6P1/1PNpN3/8/8/7r b - - 10 65".to_string(),
                "4k3/8/4K2P/6P1/1P1pN3/5N2/8/7r b - - 10 65".to_string(),
                "4k3/8/4K2P/6P1/1P1pN3/3N4/8/7r b - - 10 65".to_string(),
                "4k3/8/4KN1P/4N1P1/1P1p4/8/8/7r b - - 10 65".to_string(),
                "4k3/8/3NK2P/4N1P1/1P1p4/8/8/7r b - - 10 65".to_string(),
                "4k3/8/4K2P/2N1N1P1/1P1p4/8/8/7r b - - 10 65".to_string(),
                "4k3/8/4K2P/4N1P1/1P1p4/6N1/8/7r b - - 10 65".to_string(),
                "4k3/8/4K2P/4N1P1/1P1p4/2N5/8/7r b - - 10 65".to_string(),
                "4k3/8/4K2P/4N1P1/1P1p4/8/5N2/7r b - - 10 65".to_string(),
                "4k3/8/4K2P/4N1P1/1P1p4/8/3N4/7r b - - 10 65".to_string(),
                "4k3/7P/4K3/4N1P1/1P1pN3/8/8/7r b - - 0 65".to_string(),
                "4k3/8/4K1PP/4N3/1P1pN3/8/8/7r b - - 0 65".to_string(),
                "4k3/8/4K2P/1P2N1P1/3pN3/8/8/7r b - - 0 65".to_string(),
            ],
        ));
        scenarios.push((
            "1rbqk1nr/N1pn1ppp/1p1p1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR b KQk - 5 5".to_string(),
            vec![
                "1rbqk2r/N1pnnppp/1p1p1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6".to_string(),
                "1rbqk2r/N1pn1ppp/1p1p1b1n/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6".to_string(),
                "1rbq1knr/N1pn1ppp/1p1p1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQ - 6 6".to_string(),
                "1rbq2nr/N1pnkppp/1p1p1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQ - 6 6".to_string(),
                "1rb1k1nr/N1pnqppp/1p1p1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6".to_string(),
                "1r1qk1nr/Nbpn1ppp/1p1p1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6".to_string(),
                "1r1qk1nr/N1pn1ppp/bp1p1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6".to_string(),
                "r1bqk1nr/N1pn1ppp/1p1p1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6".to_string(),
                "2bqk1nr/Nrpn1ppp/1p1p1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6".to_string(),
                "1rbqknnr/N1p2ppp/1p1p1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6".to_string(),
                "1rbqk1nr/N1p2ppp/1p1p1b2/p1n1p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6"
                    .to_string(),
                "1rbqk1nr/N1pnbppp/1p1p4/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6".to_string(),
                "1rbqk1nr/N1pn1ppp/1p1p4/p3p1b1/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 6 6".to_string(),
                "1rbqk1nr/N1pn1ppp/1p1p4/p3p3/2PP3b/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6".to_string(),
                "1rbqk1nr/N1pn1ppp/1p1p1b2/p7/2Pp3P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6".to_string(),
                "1rbqk1nr/N1pn1pp1/1p1p1b1p/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6"
                    .to_string(),
                "1rbqk1nr/N1pn1p1p/1p1p1bp1/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6"
                    .to_string(),
                "1rbqk1nr/N2n1ppp/1ppp1b2/p3p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6".to_string(),
                "1rbqk1nr/N1pn1ppp/1p3b2/p2pp3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6".to_string(),
                "1rbqk1nr/N1pn1ppp/3p1b2/pp2p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6".to_string(),
                "1rbqk1nr/N1pn1ppp/1p1p1b2/p7/2PPp2P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6".to_string(),
                "1rbqk1nr/N1pn1ppp/1p1p1b2/4p3/p1PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6".to_string(),
                "1rbqk1nr/N1pn1pp1/1p1p1b2/p3p2p/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6"
                    .to_string(),
                "1rbqk1nr/N1pn1p1p/1p1p1b2/p3p1p1/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6"
                    .to_string(),
                "1rbqk1nr/N2n1ppp/1p1p1b2/p1p1p3/2PP3P/4PB2/PP3PP1/R1BQK1NR w KQk - 0 6"
                    .to_string(),
            ],
        ));
        scenarios.push((
            "6B1/1p6/5Q1k/2P3p1/3Pp2p/4P1P1/rq5P/6K1 b - - 0 57".to_string(),
            vec!["6B1/1p6/5Q2/2P3pk/3Pp2p/4P1P1/rq5P/6K1 w - - 1 58".to_string()],
        ));
        scenarios.push((
            "4r3/6kP/2PR2P1/3K2p1/6N1/8/1b3P2/8 b - - 20 70".to_string(),
            vec![
                "7r/6kP/2PR2P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "6r1/6kP/2PR2P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "5r2/6kP/2PR2P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "3r4/6kP/2PR2P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "2r5/6kP/2PR2P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "1r6/6kP/2PR2P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "r7/6kP/2PR2P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "8/4r1kP/2PR2P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "8/6kP/2PRr1P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "8/6kP/2PR2P1/3Kr1p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "8/6kP/2PR2P1/3K2p1/4r1N1/8/1b3P2/8 w - - 21 71".to_string(),
                "8/6kP/2PR2P1/3K2p1/6N1/4r3/1b3P2/8 w - - 21 71".to_string(),
                "8/6kP/2PR2P1/3K2p1/6N1/8/1b2rP2/8 w - - 21 71".to_string(),
                "8/6kP/2PR2P1/3K2p1/6N1/8/1b3P2/4r3 w - - 21 71".to_string(),
                "4r2k/7P/2PR2P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "4rk2/7P/2PR2P1/3K2p1/6N1/8/1b3P2/8 w - - 21 71".to_string(),
                "4r3/6kP/2PR1bP1/3K2p1/6N1/8/5P2/8 w - - 21 71".to_string(),
                "4r3/6kP/2PR2P1/3Kb1p1/6N1/8/5P2/8 w - - 21 71".to_string(),
                "4r3/6kP/2PR2P1/3K2p1/3b2N1/8/5P2/8 w - - 21 71".to_string(),
                "4r3/6kP/2PR2P1/3K2p1/6N1/2b5/5P2/8 w - - 21 71".to_string(),
                "4r3/6kP/2PR2P1/3K2p1/6N1/b7/5P2/8 w - - 21 71".to_string(),
                "4r3/6kP/2PR2P1/3K2p1/6N1/8/5P2/2b5 w - - 21 71".to_string(),
                "4r3/6kP/2PR2P1/3K2p1/6N1/8/5P2/b7 w - - 21 71".to_string(),
            ],
        ));
        scenarios.push((
            "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N5P/1b4P1/R3K3 w - - 0 30".to_string(),
            vec![
                "r4Bk1/1p1n3p/2p3p1/2Pp4/P3pN2/1N5P/1b4P1/R3K3 b - - 1 30".to_string(),
                "rB4k1/1p1n3p/2p3p1/2Pp4/P3pN2/1N5P/1b4P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1p1nB2p/2p3p1/2Pp4/P3pN2/1N5P/1b4P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1pBn3p/2p3p1/2Pp4/P3pN2/1N5P/1b4P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2p3p1/2PpB3/P3pN2/1N5P/1b4P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2N1/2Pp4/P3p3/1N5P/1b4P1/R3K3 b - - 0 30".to_string(),
                "r5k1/1p1n3p/2pBN1p1/2Pp4/P3p3/1N5P/1b4P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp3N/P3p3/1N5P/1b4P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2PN4/P3p3/1N5P/1b4P1/R3K3 b - - 0 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3p3/1N1N3P/1b4P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3p3/1N5P/1b2N1P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/N1Pp4/P3pN2/7P/1b4P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P2NpN2/7P/1b4P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/7P/1b1N2P1/R3K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/7P/1b4P1/R1N1K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N5P/1b3KP1/R7 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N5P/1b2K1P1/R7 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N5P/1b1K2P1/R7 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N5P/1b4P1/R4K2 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N5P/1b4P1/R2K4 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/RN5P/1b4P1/4K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N5P/Rb4P1/4K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N5P/1b4P1/3RK3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N5P/1b4P1/2R1K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N5P/1b4P1/1R2K3 b - - 1 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/P1Pp4/4pN2/1N5P/1b4P1/R3K3 b - - 0 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN1P/1N6/1b4P1/R3K3 b - - 0 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pN2/1N4PP/1b6/R3K3 b - - 0 30".to_string(),
                "r5k1/1p1n3p/2pB2p1/2Pp4/P3pNP1/1N5P/1b6/R3K3 b - - 0 30".to_string(),
            ],
        ));
        scenarios.push((
            "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/8/4N1BK/B3b3 w - - 0 30".to_string(),
            vec![
                "2qQ1r2/2p4n/1p1pP1k1/3P3p/1PP3b1/8/4N1BK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p1Q2n/1p1pP1k1/3P3p/1PP3b1/8/4N1BK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pPQk1/3P3p/1PP3b1/8/4N1BK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3Q/1PP3b1/8/4N1BK/B3b3 b - - 0 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P2Qp/1PP3b1/8/4N1BK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3Q1/8/4N1BK/B3b3 b - - 0 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3b1/7Q/4N1BK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3b1/6Q1/4N1BK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3b1/8/4NQBK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3b1/8/4N1BK/B3Q3 b - - 0 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/8/4N1B1/B3b2K b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/8/4N1B1/B3b1K1 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP1B1bQ/8/4N2K/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/7B/4N2K/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/5B2/4N2K/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/8/4N2K/B3b2B b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/8/4N2K/B3bB2 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP2NbQ/8/6BK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PPN2bQ/8/6BK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/6N1/6BK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/2N5/6BK/B3b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/8/6BK/B3b1N1 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/8/6BK/B1N1b3 b - - 1 30".to_string(),
                "2q2r1B/2p4n/1p1pP1k1/3P3p/1PP3bQ/8/4N1BK/4b3 b - - 1 30".to_string(),
                "2q2r2/2p3Bn/1p1pP1k1/3P3p/1PP3bQ/8/4N1BK/4b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pPBk1/3P3p/1PP3bQ/8/4N1BK/4b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3PB2p/1PP3bQ/8/4N1BK/4b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PPB2bQ/8/4N1BK/4b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/2B5/4N1BK/4b3 b - - 1 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/3P3p/1PP3bQ/8/1B2N1BK/4b3 b - - 1 30".to_string(),
                "2q2r2/2p1P2n/1p1p2k1/3P3p/1PP3bQ/8/4N1BK/B3b3 b - - 0 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/2PP3p/1P4bQ/8/4N1BK/B3b3 b - - 0 30".to_string(),
                "2q2r2/2p4n/1p1pP1k1/1P1P3p/2P3bQ/8/4N1BK/B3b3 b - - 0 30".to_string(),
            ],
        ));
        scenarios.push((
            "r4rk1/pbq1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 b - - 0 12".to_string(),
            vec![
                "r4r1k/pbq1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4r2/pbq1bpkp/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r3r1k1/pbq1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r2r2k1/pbq1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r1r3k1/pbq1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "rr4k1/pbq1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "4rrk1/pbq1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "3r1rk1/pbq1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "2r2rk1/pbq1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "1r3rk1/pbq1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r2b1rk1/pbq2p1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pbq2p1p/1p2pbp1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pbq2p1p/1p1bp1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pbq2p1p/1p2p1p1/3pP1b1/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pbq2p1p/1p2p1p1/2bpP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pbq2p1p/1p2p1p1/3pP3/P1pPN2b/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pbq2p1p/1p2p1p1/3pP3/PbpPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pbq2p1p/1p2p1p1/3pP3/P1pPN3/b1P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r2q1rk1/pb2bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r1q2rk1/pb2bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "rq3rk1/pb2bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pb1qbp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pb2bp1p/1p1qp1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pb2bp1p/1pq1p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pb2bp1p/1p2p1p1/3pq3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 0 13".to_string(),
                "r4rk1/pb2bp1p/1p2p1p1/2qpP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r1b2rk1/p1q1bp1p/1p2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/p1q1bp1p/1pb1p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/p1q1bp1p/bp2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 1 13".to_string(),
                "r4rk1/pbq1bp1p/1p2p1p1/4P3/P1pPp3/2P3R1/1PB3PP/R1BQ2K1 w - - 0 13".to_string(),
                "r4rk1/pbq1bp2/1p2p1pp/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 0 13".to_string(),
                "r4rk1/pbq1b2p/1p2ppp1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 0 13".to_string(),
                "r4rk1/1bq1bp1p/pp2p1p1/3pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 0 13".to_string(),
                "r4rk1/pbq1bp1p/1p2p3/3pP1p1/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 0 13".to_string(),
                "r4rk1/pbq1bp1p/4p1p1/1p1pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 0 13".to_string(),
                "r4rk1/pbq1bp2/1p2p1p1/3pP2p/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 0 13".to_string(),
                "r4rk1/pbq1b2p/1p2p1p1/3pPp2/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - f6 0 13".to_string(),
                "r4rk1/1bq1bp1p/1p2p1p1/p2pP3/P1pPN3/2P3R1/1PB3PP/R1BQ2K1 w - - 0 13".to_string(),
            ],
        ));
        scenarios.push((
            "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQ1/P3N1PP/R1BR2K1 w kq - 1 10".to_string(),
            vec![
                "r2qk2r/1p2npbp/p2pb1Q1/2p1p3/2P1P3/NPP2P2/P3N1PP/R1BR2K1 b kq - 0 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p1Q1/2P1P3/NPP2P2/P3N1PP/R1BR2K1 b kq - 2 10"
                    .to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1Q3/2P1P3/NPP2P2/P3N1PP/R1BR2K1 b kq - 0 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P2Q/NPP2P2/P3N1PP/R1BR2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P1Q1/NPP2P2/P3N1PP/R1BR2K1 b kq - 2 10"
                    .to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1PQ2/NPP2P2/P3N1PP/R1BR2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2P1Q/P3N1PP/R1BR2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2P2/P3NQPP/R1BR2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2P2/P3N1PP/R1BRQ1K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/1Np1p3/2P1P3/1PP2PQ1/P3N1PP/R1BR2K1 b kq - 2 10"
                    .to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/1PP2PQ1/P1N1N1PP/R1BR2K1 b kq - 2 10"
                    .to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/1PP2PQ1/P3N1PP/RNBR2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1PN2/NPP2PQ1/P5PP/R1BR2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2PNP3/NPP2PQ1/P5PP/R1BR2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQ1/P3NKPP/R1BR4 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQ1/P3N1PP/R1BR3K b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQ1/P3N1PP/R1BR1K2 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2Rb1p1/2p1p3/2P1P3/NPP2PQ1/P3N1PP/R1B3K1 b kq - 0 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2pRp3/2P1P3/NPP2PQ1/P3N1PP/R1B3K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2PRP3/NPP2PQ1/P3N1PP/R1B3K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPPR1PQ1/P3N1PP/R1B3K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQ1/P2RN1PP/R1B3K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQ1/P3N1PP/R1B2RK1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQ1/P3N1PP/R1B1R1K1 b kq - 2 10"
                    .to_string(),
                "r2qk2r/1p2npbp/p2pb1pB/2p1p3/2P1P3/NPP2PQ1/P3N1PP/R2R2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p1B1/2P1P3/NPP2PQ1/P3N1PP/R2R2K1 b kq - 2 10"
                    .to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1PB2/NPP2PQ1/P3N1PP/R2R2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP1BPQ1/P3N1PP/R2R2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQ1/P2BN1PP/R2R2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQ1/PB2N1PP/R2R2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQ1/P3N1PP/1RBR2K1 b kq - 2 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1PP2/NPP3Q1/P3N1PP/R1BR2K1 b kq - 0 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/1PP1P3/N1P2PQ1/P3N1PP/R1BR2K1 b kq - 0 10"
                    .to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P3/NPP2PQP/P3N1P1/R1BR2K1 b kq - 0 10".to_string(),
                "r2qk2r/1p2npbp/p2pb1p1/2p1p3/2P1P2P/NPP2PQ1/P3N1P1/R1BR2K1 b kq - 0 10"
                    .to_string(),
            ],
        ));
        scenarios.push((
            "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P2R1P1P/3R2K1 w - - 3 28".to_string(),
            vec![
                "r5k1/2Q2p1p/1p1q2p1/r2P4/8/6P1/P2R1P1P/3R2K1 b - - 0 28".to_string(),
                "r5k1/2p2p1p/1pQq2p1/r2P4/8/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/Qp1q2p1/r2P4/8/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r1QP4/8/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/rQ1P4/8/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/7Q/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/6Q1/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/5Q2/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/4Q3/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/3Q4/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/1Q6/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/Q7/6P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/8/3Q2P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/8/2Q3P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/8/1Q4P1/P2R1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/8/6P1/P2RQP1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/8/6P1/P1QR1P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/8/6P1/P2R1P1P/3R1QK1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/8/6P1/P2R1P1P/2QR2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2QR4/6P1/P4P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/3R2P1/P4P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P3RP1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P1R2P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/PR3P1P/3R2K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P2R1PKP/3R4 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P2R1P1P/3R3K b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P2R1P1P/3R1K2 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P2R1P1P/5RK1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P2R1P1P/4R1K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P2R1P1P/2R3K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P2R1P1P/1R4K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6P1/P2R1P1P/R5K1 b - - 4 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q3P1/8/P2R1P1P/3R2K1 b - - 0 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/6PP/P2R1P2/3R2K1 b - - 0 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/5PP1/P2R3P/3R2K1 b - - 0 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q5/P5P1/3R1P1P/3R2K1 b - - 0 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q4P/6P1/P2R1P2/3R2K1 b - - 0 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/2Q2P2/6P1/P2R3P/3R2K1 b - - 0 28".to_string(),
                "r5k1/2p2p1p/1p1q2p1/r2P4/P1Q5/6P1/3R1P1P/3R2K1 b - - 0 28".to_string(),
            ],
        ));
        scenarios.push((
            "7k/1p4pp/p1P5/8/4pN1P/5RPK/P2r4/7q w - - 3 43".to_string(),
            vec!["7k/1p4pp/p1P5/8/4pNKP/5RP1/P2r4/7q b - - 4 43".to_string()],
        ));
        scenarios.push((
            "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPR1NPB1/2RQ2K1 w - - 0 17".to_string(),
            vec![
                "2rq1rk1/1b3p2/ppn1pb1p/3p2N1/3P3P/4P1P1/PPR1NPB1/2RQ2K1 b - - 0 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3pN1p1/3P3P/4P1P1/PPR1NPB1/2RQ2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4P1P1/PPR1NPBN/2RQ2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4P1P1/PPRNNPB1/2RQ2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4P1P1/PPR1NPB1/2RQN1K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNPB/PPR1NP2/2RQ2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPR1NP2/2RQ2KB b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPR1NP2/2RQ1BK1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P1N1P/4PNP1/PPR2PB1/2RQ2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/2N1PNP1/PPR2PB1/2RQ2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppR1pb1p/3p2p1/3P3P/4PNP1/PP2NPB1/2RQ2K1 b - - 0 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/2Rp2p1/3P3P/4PNP1/PP2NPB1/2RQ2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/2RP3P/4PNP1/PP2NPB1/2RQ2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/2R1PNP1/PP2NPB1/2RQ2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PP1RNPB1/2RQ2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPR1NPBK/2RQ4 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPR1NPB1/2RQ3K b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPR1NPB1/2RQ1K2 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/3QPNP1/PPR1NPB1/2R3K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPRQNPB1/2R3K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPR1NPB1/2R2QK1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPR1NPB1/2R1Q1K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPR1NPB1/1R1Q2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/4PNP1/PPR1NPB1/R2Q2K1 b - - 1 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2P1/3P4/4PNP1/PPR1NPB1/2RQ2K1 b - - 0 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2pP/3P4/4PNP1/PPR1NPB1/2RQ2K1 b - - 0 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P2PP/4PN2/PPR1NPB1/2RQ2K1 b - - 0 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3PP2P/5NP1/PPR1NPB1/2RQ2K1 b - - 0 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/1P2PNP1/P1R1NPB1/2RQ2K1 b - - 0 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/3P3P/P3PNP1/1PR1NPB1/2RQ2K1 b - - 0 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/1P1P3P/4PNP1/P1R1NPB1/2RQ2K1 b - - 0 17".to_string(),
                "2rq1rk1/1b3p2/ppn1pb1p/3p2p1/P2P3P/4PNP1/1PR1NPB1/2RQ2K1 b - - 0 17".to_string(),
            ],
        ));
        scenarios.push((
            "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/2R2PP1/2R3K1 w - - 4 25".to_string(),
            vec![
                "r2q2k1/1pNr1pb1/2n1pnpp/8/3P4/PB2BQ1P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/Np1r1pb1/2n1pnpp/8/3P4/PB2BQ1P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2nNpnpp/8/3P4/PB2BQ1P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/8/3P4/PBN1BQ1P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pQpp/1N6/3P4/PB2B2P/2R2PP1/2R3K1 b - - 0 25".to_string(),
                "r2q2k1/1p1r1pb1/2Q1pnpp/1N6/3P4/PB2B2P/2R2PP1/2R3K1 b - - 0 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N5Q/3P4/PB2B2P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N3Q2/3P4/PB2B2P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N1Q4/3P4/PB2B2P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P2Q1/PB2B2P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P1Q2/PB2B2P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3PQ3/PB2B2P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2B1QP/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2B2P/2R1QPP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2B2P/2R2PP1/2RQ2K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpB/1N6/3P4/PB3Q1P/2R2PP1/2R3K1 b - - 0 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N4B1/3P4/PB3Q1P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P1B2/PB3Q1P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB3Q1P/2RB1PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1Bnpp/1N6/3P4/P3BQ1P/2R2PP1/2R3K1 b - - 0 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N1B4/3P4/P3BQ1P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/2BP4/P3BQ1P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/B2P4/P3BQ1P/2R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/P3BQ1P/B1R2PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2R1pnpp/1N6/3P4/PB2BQ1P/5PP1/2R3K1 b - - 0 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1NR5/3P4/PB2BQ1P/5PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/2RP4/PB2BQ1P/5PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PBR1BQ1P/5PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/4RPP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/3R1PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/1R3PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/R4PP1/2R3K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/2R2PPK/2R5 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/2R2PP1/2R4K b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/2R2PP1/2R2K2 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/2R2PP1/5RK1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/2R2PP1/4R1K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/2R2PP1/3R2K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/2R2PP1/1R4K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQ1P/2R2PP1/R5K1 b - - 5 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N1P4/8/PB2BQ1P/2R2PP1/2R3K1 b - - 0 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P3P/PB2BQ2/2R2PP1/2R3K1 b - - 0 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/P2P4/1B2BQ1P/2R2PP1/2R3K1 b - - 0 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P4/PB2BQPP/2R2P2/2R3K1 b - - 0 25".to_string(),
                "r2q2k1/1p1r1pb1/2n1pnpp/1N6/3P2P1/PB2BQ1P/2R2P2/2R3K1 b - - 0 25".to_string(),
            ],
        ));
        scenarios.push((
            "7r/6k1/R7/8/5P1p/7K/5P2/8 w - - 10 73".to_string(),
            vec![
                "R6r/6k1/8/8/5P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/R5k1/8/8/5P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/7R/8/5P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/6R1/8/5P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/5R2/8/5P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/4R3/8/5P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/3R4/8/5P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/2R5/8/5P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/1R6/8/5P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/8/R7/5P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/8/8/R4P1p/7K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/8/8/5P1p/R6K/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/8/8/5P1p/7K/R4P2/8 b - - 11 73".to_string(),
                "7r/6k1/8/8/5P1p/7K/5P2/R7 b - - 11 73".to_string(),
                "7r/6k1/R7/8/5PKp/8/5P2/8 b - - 11 73".to_string(),
                "7r/6k1/R7/8/5P1p/8/5P1K/8 b - - 11 73".to_string(),
                "7r/6k1/R7/8/5P1p/8/5PK1/8 b - - 11 73".to_string(),
                "7r/6k1/R7/5P2/7p/7K/5P2/8 b - - 0 73".to_string(),
                "7r/6k1/R7/8/5P1p/5P1K/8/8 b - - 0 73".to_string(),
            ],
        ));
        scenarios.push((
            "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N1K1/2R2R2 w - - 5 18".to_string(),
            vec![
                "2r1r3/pp1q1p1k/2n1p1Qb/3p3p/3P3P/PP2PPP1/1B2N1K1/2R2R2 b - - 0 18".to_string(),
                "2r1r3/pp1q1p1k/Q1n1p1pb/3p3p/3P3P/PP2PPP1/1B2N1K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p1Q1p/3P3P/PP2PPP1/1B2N1K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/1Q1p3p/3P3P/PP2PPP1/1B2N1K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3PQ2P/PP2PPP1/1B2N1K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/2QP3P/PP2PPP1/1B2N1K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PPQ1PPP1/1B2N1K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP2PPP1/1B1QN1K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP2PPP1/1BQ1N1K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP2PPP1/1B2N1K1/2RQ1R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP2PPP1/1B2N1K1/1QR2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPPK/1B2N3/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N2K/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2NK2/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N3/2R2R1K b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N3/2R2RK1 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P1N1P/PP1QPPP1/1B4K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PPNQPPP1/1B4K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B4K1/2R2RN1 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PPBQPPP1/4N1K1/2R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/4N1K1/B1R2R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2NRK1/2R5 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N1K1/2R4R b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N1K1/2R3R1 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N1K1/2R1R3 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N1K1/2RR4 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2R1p1pb/3p3p/3P3P/PP1QPPP1/1B2N1K1/5R2 b - - 0 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/2Rp3p/3P3P/PP1QPPP1/1B2N1K1/5R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/2RP3P/PP1QPPP1/1B2N1K1/5R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PPRQPPP1/1B2N1K1/5R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1BR1N1K1/5R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N1K1/4RR2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N1K1/3R1R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N1K1/1R3R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P3P/PP1QPPP1/1B2N1K1/R4R2 b - - 6 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P2PP/PP1QPP2/1B2N1K1/2R2R2 b - - 0 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3P1P1P/PP1QP1P1/1B2N1K1/2R2R2 b - - 0 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/3PP2P/PP1Q1PP1/1B2N1K1/2R2R2 b - - 0 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/1P1P3P/P2QPPP1/1B2N1K1/2R2R2 b - - 0 18".to_string(),
                "2r1r3/pp1q1p1k/2n1p1pb/3p3p/P2P3P/1P1QPPP1/1B2N1K1/2R2R2 b - - 0 18".to_string(),
            ],
        ));
        scenarios.push((
            "8/1p3p2/7p/3k4/3bp2P/3p4/p5K1/3R4 b - - 1 61".to_string(),
            vec![
                "8/1p3p2/4k2p/8/3bp2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/3k3p/8/3bp2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/2k4p/8/3bp2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/7p/4k3/3bp2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/7p/2k5/3bp2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/7p/8/2kbp2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "7b/1p3p2/7p/3k4/4p2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3pb1/7p/3k4/4p2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/bp3p2/7p/3k4/4p2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/5b1p/3k4/4p2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/1b5p/3k4/4p2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/7p/3kb3/4p2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/7p/2bk4/4p2P/3p4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/7p/3k4/4p2P/3pb3/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/7p/3k4/4p2P/2bp4/p5K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/7p/3k4/4p2P/3p4/p4bK1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/7p/3k4/4p2P/3p4/pb4K1/3R4 w - - 2 62".to_string(),
                "8/1p3p2/7p/3k4/4p2P/3p4/p5K1/3R2b1 w - - 2 62".to_string(),
                "8/1p3p2/7p/3k4/4p2P/3p4/p5K1/b2R4 w - - 2 62".to_string(),
                "8/1p6/5p1p/3k4/3bp2P/3p4/p5K1/3R4 w - - 0 62".to_string(),
                "8/5p2/1p5p/3k4/3bp2P/3p4/p5K1/3R4 w - - 0 62".to_string(),
                "8/1p3p2/8/3k3p/3bp2P/3p4/p5K1/3R4 w - - 0 62".to_string(),
                "8/1p3p2/7p/3k4/3b3P/3pp3/p5K1/3R4 w - - 0 62".to_string(),
                "8/1p3p2/7p/3k4/3bp2P/8/p2p2K1/3R4 w - - 0 62".to_string(),
                "8/1p3p2/7p/3k4/3bp2P/3p4/6K1/q2R4 w - - 0 62".to_string(),
                "8/1p3p2/7p/3k4/3bp2P/3p4/6K1/r2R4 w - - 0 62".to_string(),
                "8/1p3p2/7p/3k4/3bp2P/3p4/6K1/b2R4 w - - 0 62".to_string(),
                "8/1p3p2/7p/3k4/3bp2P/3p4/6K1/n2R4 w - - 0 62".to_string(),
                "8/1p6/7p/3k1p2/3bp2P/3p4/p5K1/3R4 w - - 0 62".to_string(),
                "8/5p2/7p/1p1k4/3bp2P/3p4/p5K1/3R4 w - - 0 62".to_string(),
            ],
        ));
        scenarios.push((
            "8/8/4K3/8/p1nkN3/4p3/8/8 w - - 6 92".to_string(),
            vec![
                "8/5K2/8/8/p1nkN3/4p3/8/8 b - - 7 92".to_string(),
                "8/4K3/8/8/p1nkN3/4p3/8/8 b - - 7 92".to_string(),
                "8/3K4/8/8/p1nkN3/4p3/8/8 b - - 7 92".to_string(),
                "8/8/5K2/8/p1nkN3/4p3/8/8 b - - 7 92".to_string(),
                "8/8/8/5K2/p1nkN3/4p3/8/8 b - - 7 92".to_string(),
                "8/8/4KN2/8/p1nk4/4p3/8/8 b - - 7 92".to_string(),
                "8/8/3NK3/8/p1nk4/4p3/8/8 b - - 7 92".to_string(),
                "8/8/4K3/6N1/p1nk4/4p3/8/8 b - - 7 92".to_string(),
                "8/8/4K3/2N5/p1nk4/4p3/8/8 b - - 7 92".to_string(),
                "8/8/4K3/8/p1nk4/4p1N1/8/8 b - - 7 92".to_string(),
                "8/8/4K3/8/p1nk4/2N1p3/8/8 b - - 7 92".to_string(),
                "8/8/4K3/8/p1nk4/4p3/5N2/8 b - - 7 92".to_string(),
                "8/8/4K3/8/p1nk4/4p3/3N4/8 b - - 7 92".to_string(),
            ],
        ));
        scenarios.push((
            "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/2QB2PP/6K1 w - - 2 36".to_string(),
            vec![
                "1r4kR/pr4q1/3b1p2/2pPp1p1/4P2p/PP3R2/2QB2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4qR/3b1p2/2pPp1p1/4P2p/PP3R2/2QB2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p1R/2pPp1p1/4P2p/PP3R2/2QB2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1R1/4P2p/PP3R2/2QB2PP/6K1 b - - 0 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1p1/4P2R/PP3R2/2QB2PP/6K1 b - - 0 36".to_string(),
                "1r4k1/pr4q1/3b1R2/2pPp1pR/4P2p/PP6/2QB2PP/6K1 b - - 0 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPpRpR/4P2p/PP6/2QB2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4PR1p/PP6/2QB2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP5R/2QB2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP4R1/2QB2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP2R3/2QB2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP1R4/2QB2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PPR5/2QB2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP6/2QB1RPP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP6/2QB2PP/5RK1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1BR/4P2p/PP3R2/2Q3PP/6K1 b - - 0 36".to_string(),
                "1r4k1/pr4q1/3b1p2/B1pPp1pR/4P2p/PP3R2/2Q3PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4PB1p/PP3R2/2Q3PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/1B2P2p/PP3R2/2Q3PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP2BR2/2Q3PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PPB2R2/2Q3PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/2Q3PP/4B1K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/2Q3PP/2B3K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2QPp1pR/4P2p/PP3R2/3B2PP/6K1 b - - 0 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/2Q1P2p/PP3R2/3B2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP1Q1R2/3B2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PPQ2R2/3B2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/1Q1B2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/Q2B2PP/6K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/3B2PP/3Q2K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/3B2PP/2Q3K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/3B2PP/1Q4K1 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/2QB1KPP/8 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/2QB2PP/7K b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R2/2QB2PP/5K2 b - - 3 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/1P2P2p/P4R2/2QB2PP/6K1 b - - 0 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/P3P2p/1P3R2/2QB2PP/6K1 b - - 0 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3R1P/2QB2P1/6K1 b - - 0 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P2p/PP3RP1/2QB3P/6K1 b - - 0 36".to_string(),
                "1r4k1/pr4q1/3b1p2/2pPp1pR/4P1Pp/PP3R2/2QB3P/6K1 b - g3 0 36".to_string(),
            ],
        ));
        scenarios.push((
            "8/8/1N3k2/2pK4/2P5/r7/8/6b1 b - - 7 66".to_string(),
            vec![
                "8/6k1/1N6/2pK4/2P5/r7/8/6b1 w - - 8 67".to_string(),
                "8/5k2/1N6/2pK4/2P5/r7/8/6b1 w - - 8 67".to_string(),
                "8/4k3/1N6/2pK4/2P5/r7/8/6b1 w - - 8 67".to_string(),
                "8/8/1N4k1/2pK4/2P5/r7/8/6b1 w - - 8 67".to_string(),
                "8/8/1N6/2pK2k1/2P5/r7/8/6b1 w - - 8 67".to_string(),
                "8/8/1N6/2pK1k2/2P5/r7/8/6b1 w - - 8 67".to_string(),
                "r7/8/1N3k2/2pK4/2P5/8/8/6b1 w - - 8 67".to_string(),
                "8/r7/1N3k2/2pK4/2P5/8/8/6b1 w - - 8 67".to_string(),
                "8/8/rN3k2/2pK4/2P5/8/8/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/r1pK4/2P5/8/8/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/r1P5/8/8/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/7r/8/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/6r1/8/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/5r2/8/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/4r3/8/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/3r4/8/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/2r5/8/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/1r6/8/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/8/r7/6b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/8/8/r5b1 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2Pb4/r7/8/8 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/r3b3/8/8 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/r7/7b/8 w - - 8 67".to_string(),
                "8/8/1N3k2/2pK4/2P5/r7/5b2/8 w - - 8 67".to_string(),
            ],
        ));
        scenarios.push((
            "5k2/1n3p2/4n1pp/4R3/7P/3PPKP1/5P2/8 b - - 0 33".to_string(),
            vec![
                "6k1/1n3p2/4n1pp/4R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "4k3/1n3p2/4n1pp/4R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "8/1n3pk1/4n1pp/4R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "8/1n2kp2/4n1pp/4R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "3n1k2/5p2/4n1pp/4R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "5k2/5p2/3nn1pp/4R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "5k2/5p2/4n1pp/2n1R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "5k2/5p2/4n1pp/n3R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "3n1k2/1n3p2/6pp/4R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "5k2/1n3pn1/6pp/4R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "5k2/1nn2p2/6pp/4R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "5k2/1n3p2/6pp/4R1n1/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "5k2/1n3p2/6pp/2n1R3/7P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "5k2/1n3p2/6pp/4R3/5n1P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "5k2/1n3p2/6pp/4R3/3n3P/3PPKP1/5P2/8 w - - 1 34".to_string(),
                "5k2/1n6/4nppp/4R3/7P/3PPKP1/5P2/8 w - - 0 34".to_string(),
                "5k2/1n3p2/4n1p1/4R2p/7P/3PPKP1/5P2/8 w - - 0 34".to_string(),
                "5k2/1n3p2/4n2p/4R1p1/7P/3PPKP1/5P2/8 w - - 0 34".to_string(),
                "5k2/1n6/4n1pp/4Rp2/7P/3PPKP1/5P2/8 w - - 0 34".to_string(),
            ],
        ));
        scenarios.push((
            "r2qk1nr/6pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 b kq - 0 8".to_string(),
            vec![
                "r2qk2r/4n1pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2qk2r/6pp/ppbb1p1n/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2q1knr/6pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w - - 1 9".to_string(),
                "r2q2nr/5kpp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w - - 1 9".to_string(),
                "r2q2nr/4k1pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w - - 1 9".to_string(),
                "r2q2nr/3k2pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w - - 1 9".to_string(),
                "r1q1k1nr/6pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "rq2k1nr/6pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r3k1nr/4q1pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r3k1nr/3q2pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r3k1nr/2q3pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "2rqk1nr/6pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w k - 1 9".to_string(),
                "1r1qk1nr/6pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w k - 1 9".to_string(),
                "3qk1nr/r5pp/ppbb1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w k - 1 9".to_string(),
                "r2qkbnr/6pp/ppb2p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "rb1qk1nr/6pp/ppb2p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2qk1nr/4b1pp/ppb2p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2qk1nr/2b3pp/ppb2p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2qk1nr/6pp/ppb2p2/2ppb3/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2qk1nr/6pp/ppb2p2/2pp4/P2P1b2/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2qk1nr/6pp/ppb2p2/2pp4/P2P4/1P2PNb1/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2qk1nr/6pp/ppb2p2/2pp4/P2P4/1P2PN2/5PPb/RNBQ1RK1 w kq - 0 9".to_string(),
                "r2qk1nr/3b2pp/pp1b1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2qk1nr/1b4pp/pp1b1p2/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2qk1nr/6pp/pp1b1p2/1bpp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 1 9".to_string(),
                "r2qk1nr/6pp/pp1b1p2/2pp4/b2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 0 9".to_string(),
                "r2qk1nr/6pp/ppbb1p2/3p4/P2p4/1P2PN2/5PPP/RNBQ1RK1 w kq - 0 9".to_string(),
                "r2qk1nr/6p1/ppbb1p1p/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 0 9".to_string(),
                "r2qk1nr/7p/ppbb1pp1/2pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 0 9".to_string(),
                "r2qk1nr/6pp/ppbb4/2pp1p2/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 0 9".to_string(),
                "r2qk1nr/6pp/p1bb1p2/1ppp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 0 9".to_string(),
                "r2qk1nr/6pp/1pbb1p2/p1pp4/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 0 9".to_string(),
                "r2qk1nr/6pp/ppbb1p2/3p4/P1pP4/1P2PN2/5PPP/RNBQ1RK1 w kq - 0 9".to_string(),
                "r2qk1nr/6p1/ppbb1p2/2pp3p/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 0 9".to_string(),
                "r2qk1nr/7p/ppbb1p2/2pp2p1/P2P4/1P2PN2/5PPP/RNBQ1RK1 w kq - 0 9".to_string(),
            ],
        ));
        scenarios.push((
            "8/2r1n3/2P1pkp1/1B2p2p/8/2K3P1/7P/5R2 b - - 11 41".to_string(),
            vec![
                "8/2r1n1k1/2P1p1p1/1B2p2p/8/2K3P1/7P/5R2 w - - 12 42".to_string(),
                "8/2r1n3/2P1p1p1/1B2p1kp/8/2K3P1/7P/5R2 w - - 12 42".to_string(),
                "8/2r5/2P1pkp1/1B2pn1p/8/2K3P1/7P/5R2 w - - 12 42".to_string(),
            ],
        ));
        scenarios.push((
            "rr6/3q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 b - - 10 25".to_string(),
            vec![
                "r6r/3q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r5r1/3q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r4r2/3q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r3r3/3q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r2r4/3q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r1r5/3q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r7/1r1q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r7/3q1k2/1rQbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r7/3q1k2/2Qbppp1/1r3b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r7/3q1k2/2Qbppp1/5b1p/pr1Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r7/3q1k2/2Qbppp1/5b1p/p2Pp2P/1r2B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "r7/3q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/Pr3P2/R1R2BK1 w - - 11 26".to_string(),
                "r7/3q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/RrR2BK1 w - - 11 26".to_string(),
                "1r6/r2q1k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "1r6/3q1k2/r1Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "1r6/3q1k2/2Qbppp1/r4b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr4k1/3q4/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr3k2/3q4/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr2k3/3q4/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3q2k1/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3qk3/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr2q3/5k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr1q4/5k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rrq5/5k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/4qk2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/2q2k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/1q3k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/q4k2/2Qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/5k2/2qbppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 0 26".to_string(),
                "rr3b2/3q1k2/2Q1ppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3qbk2/2Q1ppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/2bq1k2/2Q1ppp1/5b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3q1k2/2Q1ppp1/4bb1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3q1k2/2Q1ppp1/2b2b1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3q1k2/2Q1ppp1/5b1p/p2Ppb1P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3q1k2/2Q1ppp1/5b1p/pb1Pp2P/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3q1k2/2Q1ppp1/5b1p/p2Pp2P/4B1b1/P4P2/R1R2BK1 w - - 0 26".to_string(),
                "rr6/3q1k2/2Q1ppp1/5b1p/p2Pp2P/b3B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3q1k2/2Qbppp1/7p/p2Pp1bP/4B1P1/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3q1k2/2Qbppp1/7p/p2Pp2P/4B1Pb/P4P2/R1R2BK1 w - - 11 26".to_string(),
                "rr6/3q1k2/2Qbpp2/5bpp/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 0 26".to_string(),
                "rr6/3q1k2/2Qb1pp1/4pb1p/p2Pp2P/4B1P1/P4P2/R1R2BK1 w - - 0 26".to_string(),
                "rr6/3q1k2/2Qbppp1/5b1p/3Pp2P/p3B1P1/P4P2/R1R2BK1 w - - 0 26".to_string(),
            ],
        ));
        scenarios.push((
            "7R/8/3r4/8/1Kbk4/8/8/8 b - - 93 111".to_string(),
            vec![
                "3r3R/8/8/8/1Kbk4/8/8/8 w - - 94 112".to_string(),
                "7R/3r4/8/8/1Kbk4/8/8/8 w - - 94 112".to_string(),
                "7R/8/7r/8/1Kbk4/8/8/8 w - - 94 112".to_string(),
                "7R/8/6r1/8/1Kbk4/8/8/8 w - - 94 112".to_string(),
                "7R/8/5r2/8/1Kbk4/8/8/8 w - - 94 112".to_string(),
                "7R/8/4r3/8/1Kbk4/8/8/8 w - - 94 112".to_string(),
                "7R/8/2r5/8/1Kbk4/8/8/8 w - - 94 112".to_string(),
                "7R/8/1r6/8/1Kbk4/8/8/8 w - - 94 112".to_string(),
                "7R/8/r7/8/1Kbk4/8/8/8 w - - 94 112".to_string(),
                "7R/8/8/3r4/1Kbk4/8/8/8 w - - 94 112".to_string(),
                "7R/8/3r4/4k3/1Kb5/8/8/8 w - - 94 112".to_string(),
                "7R/8/3r4/3k4/1Kb5/8/8/8 w - - 94 112".to_string(),
                "7R/8/3r4/8/1Kb1k3/8/8/8 w - - 94 112".to_string(),
                "7R/8/3r4/8/1Kb5/4k3/8/8 w - - 94 112".to_string(),
                "7R/8/3r4/8/1Kb5/3k4/8/8 w - - 94 112".to_string(),
                "6bR/8/3r4/8/1K1k4/8/8/8 w - - 94 112".to_string(),
                "7R/5b2/3r4/8/1K1k4/8/8/8 w - - 94 112".to_string(),
                "7R/8/3rb3/8/1K1k4/8/8/8 w - - 94 112".to_string(),
                "7R/8/b2r4/8/1K1k4/8/8/8 w - - 94 112".to_string(),
                "7R/8/3r4/3b4/1K1k4/8/8/8 w - - 94 112".to_string(),
                "7R/8/3r4/1b6/1K1k4/8/8/8 w - - 94 112".to_string(),
                "7R/8/3r4/8/1K1k4/3b4/8/8 w - - 94 112".to_string(),
                "7R/8/3r4/8/1K1k4/1b6/8/8 w - - 94 112".to_string(),
                "7R/8/3r4/8/1K1k4/8/4b3/8 w - - 94 112".to_string(),
                "7R/8/3r4/8/1K1k4/8/b7/8 w - - 94 112".to_string(),
                "7R/8/3r4/8/1K1k4/8/8/5b2 w - - 94 112".to_string(),
            ],
        ));
        scenarios.push((
            "8/p7/2K3k1/1p4p1/r7/PR6/6P1/8 b - - 1 43".to_string(),
            vec![
                "8/p6k/2K5/1p4p1/r7/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p5k1/2K5/1p4p1/r7/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p4k2/2K5/1p4p1/r7/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K4k/1p4p1/r7/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K2k2/1p4p1/r7/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K5/1p4pk/r7/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K5/1p3kp1/r7/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/r1K3k1/1p4p1/8/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K3k1/rp4p1/8/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K3k1/1p4p1/7r/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K3k1/1p4p1/6r1/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K3k1/1p4p1/5r2/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K3k1/1p4p1/4r3/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K3k1/1p4p1/3r4/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K3k1/1p4p1/2r5/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K3k1/1p4p1/1r6/PR6/6P1/8 w - - 2 44".to_string(),
                "8/p7/2K3k1/1p4p1/8/rR6/6P1/8 w - - 0 44".to_string(),
                "8/8/p1K3k1/1p4p1/r7/PR6/6P1/8 w - - 0 44".to_string(),
                "8/p7/2K3k1/1p6/r5p1/PR6/6P1/8 w - - 0 44".to_string(),
                "8/p7/2K3k1/6p1/rp6/PR6/6P1/8 w - - 0 44".to_string(),
                "8/8/2K3k1/pp4p1/r7/PR6/6P1/8 w - - 0 44".to_string(),
            ],
        ));
        scenarios.push((
            "8/1p6/2b5/8/p1R5/6R1/1k3K2/8 b - - 13 57".to_string(),
            vec![
                "4b3/1p6/8/8/p1R5/6R1/1k3K2/8 w - - 14 58".to_string(),
                "8/1p1b4/8/8/p1R5/6R1/1k3K2/8 w - - 14 58".to_string(),
                "8/1p6/8/3b4/p1R5/6R1/1k3K2/8 w - - 14 58".to_string(),
                "8/1p6/8/1b6/p1R5/6R1/1k3K2/8 w - - 14 58".to_string(),
                "8/1p6/8/8/p1R1b3/6R1/1k3K2/8 w - - 14 58".to_string(),
                "8/1p6/8/8/p1R5/5bR1/1k3K2/8 w - - 14 58".to_string(),
                "8/1p6/8/8/p1R5/6R1/1k3Kb1/8 w - - 14 58".to_string(),
                "8/1p6/8/8/p1R5/6R1/1k3K2/7b w - - 14 58".to_string(),
                "8/1p6/2b5/8/p1R5/6R1/k4K2/8 w - - 14 58".to_string(),
                "8/1p6/2b5/8/p1R5/6R1/5K2/1k6 w - - 14 58".to_string(),
                "8/1p6/2b5/8/p1R5/6R1/5K2/k7 w - - 14 58".to_string(),
                "8/8/1pb5/8/p1R5/6R1/1k3K2/8 w - - 0 58".to_string(),
                "8/1p6/2b5/8/2R5/p5R1/1k3K2/8 w - - 0 58".to_string(),
                "8/8/2b5/1p6/p1R5/6R1/1k3K2/8 w - - 0 58".to_string(),
            ],
        ));
        scenarios.push((
            "8/5kp1/4R3/3P4/4p1pK/3r4/5P2/8 b - - 1 43".to_string(),
            vec![
                "6k1/6p1/4R3/3P4/4p1pK/3r4/5P2/8 w - - 2 44".to_string(),
                "5k2/6p1/4R3/3P4/4p1pK/3r4/5P2/8 w - - 2 44".to_string(),
                "8/5kp1/4R3/3r4/4p1pK/8/5P2/8 w - - 0 44".to_string(),
                "8/5kp1/4R3/3P4/3rp1pK/8/5P2/8 w - - 2 44".to_string(),
                "8/5kp1/4R3/3P4/4p1pK/7r/5P2/8 w - - 2 44".to_string(),
                "8/5kp1/4R3/3P4/4p1pK/6r1/5P2/8 w - - 2 44".to_string(),
                "8/5kp1/4R3/3P4/4p1pK/5r2/5P2/8 w - - 2 44".to_string(),
                "8/5kp1/4R3/3P4/4p1pK/4r3/5P2/8 w - - 2 44".to_string(),
                "8/5kp1/4R3/3P4/4p1pK/2r5/5P2/8 w - - 2 44".to_string(),
                "8/5kp1/4R3/3P4/4p1pK/1r6/5P2/8 w - - 2 44".to_string(),
                "8/5kp1/4R3/3P4/4p1pK/r7/5P2/8 w - - 2 44".to_string(),
                "8/5kp1/4R3/3P4/4p1pK/8/3r1P2/8 w - - 2 44".to_string(),
                "8/5kp1/4R3/3P4/4p1pK/8/5P2/3r4 w - - 2 44".to_string(),
                "8/5k2/4R1p1/3P4/4p1pK/3r4/5P2/8 w - - 0 44".to_string(),
                "8/5kp1/4R3/3P4/4p2K/3r2p1/5P2/8 w - - 0 44".to_string(),
                "8/5kp1/4R3/3P4/6pK/3rp3/5P2/8 w - - 0 44".to_string(),
                "8/5k2/4R3/3P2p1/4p1pK/3r4/5P2/8 w - - 0 44".to_string(),
            ],
        ));
        scenarios.push((
            "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1B3/5RK1 w - - 0 27".to_string(),
            vec![
                "2r1r1k1/1b1q2bp/6N1/1P2np2/p2Bp3/P3P2P/RQP1B3/5RK1 b - - 0 27".to_string(),
                "2r1r1k1/1b1q2bp/4N1p1/1P2np2/p2Bp3/P3P2P/RQP1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np1N/p2Bp3/P3P2P/RQP1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P1Nnp2/p2Bp3/P3P2P/RQP1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2Bp3/P2NP2P/RQP1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2Bp3/P3P2P/RQP1B1N1/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/Bb1q2bp/6p1/1P2np2/p3pN2/P3P2P/RQP1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/1B4p1/1P2np2/p3pN2/P3P2P/RQP1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2Bp2/p3pN2/P3P2P/RQP1B3/5RK1 b - - 0 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1PB1np2/p3pN2/P3P2P/RQP1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p3pN2/P1B1P2P/RQP1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np1B/p2BpN2/P3P2P/RQP5/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpNB1/P3P2P/RQP5/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p1BBpN2/P3P2P/RQP5/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3PB1P/RQP5/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P2BP2P/RQP5/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP5/3B1RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/pQ1BpN2/P3P2P/R1P1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P1Q1P2P/R1P1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/PQ2P2P/R1P1B3/5RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/R1P1B3/2Q2RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/R1P1B3/1Q3RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/R1P1B3/Q4RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/1QP1B3/R4RK1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1B2K/5R2 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1B1K1/5R2 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1BK2/5R2 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1B3/5R1K b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3PR1P/RQP1B3/6K1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1BR2/6K1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1B3/4R1K1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1B3/3R2K1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1B3/2R3K1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1B3/1R4K1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P3P2P/RQP1B3/R5K1 b - - 1 27".to_string(),
                "2r1r1k1/1b1q2bp/1P4p1/4np2/p2BpN2/P3P2P/RQP1B3/5RK1 b - - 0 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN1P/P3P3/RQP1B3/5RK1 b - - 0 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p2BpN2/P1P1P2P/RQ2B3/5RK1 b - - 0 27".to_string(),
                "2r1r1k1/1b1q2bp/6p1/1P2np2/p1PBpN2/P3P2P/RQ2B3/5RK1 b - - 0 27".to_string(),
            ],
        ));
        scenarios.push((
            "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/PBR3KP/8 w - - 3 42".to_string(),
            vec![
                "5rrk/2p1b1p1/1p3p1R/p2PP3/q1P2P2/3Q4/PBR3KP/8 b - - 0 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP1R1/q1P2P2/3Q4/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PPR2/q1P2P2/3Q4/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP3/q1P2P1R/3Q4/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP3/q1P2P2/3Q3R/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1pQ/1p3p1p/p2PP2R/q1P2P2/8/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3pQp/p2PP2R/q1P2P2/8/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PPQ1R/q1P2P2/8/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P1QP2/8/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1PQ1P2/8/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/7Q/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/6Q1/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/5Q2/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/4Q3/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/2Q5/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/1Q6/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/Q7/PBR3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/8/PBR1Q1KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/8/PBRQ2KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/8/PBR3KP/5Q2 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/8/PBR3KP/3Q4 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q3K/PBR4P/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q2K1/PBR4P/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q1K2/PBR4P/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/PBR2K1P/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/PBR4P/7K b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/PBR4P/6K1 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/PBR4P/5K2 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/2RQ4/PB4KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/PB3RKP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/PB2R1KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/PB1R2KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/PB4KP/2R5 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1PB1P2/3Q4/P1R3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/2BQ4/P1R3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/B2Q4/P1R3KP/8 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/P1R3KP/2B5 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q4/P1R3KP/B7 b - - 4 42".to_string(),
                "5rrk/2p1b1p1/1p3P1p/p2P3R/q1P2P2/3Q4/PBR3KP/8 b - - 0 42".to_string(),
                "5rrk/2p1b1p1/1p2Pp1p/p2P3R/q1P2P2/3Q4/PBR3KP/8 b - - 0 42".to_string(),
                "5rrk/2p1b1p1/1p1P1p1p/p3P2R/q1P2P2/3Q4/PBR3KP/8 b - - 0 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PPP1R/q1P5/3Q4/PBR3KP/8 b - - 0 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p1PPP2R/q4P2/3Q4/PBR3KP/8 b - - 0 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/3Q3P/PBR3K1/8 b - - 0 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P2/P2Q4/1BR3KP/8 b - - 0 42".to_string(),
                "5rrk/2p1b1p1/1p3p1p/p2PP2R/q1P2P1P/3Q4/PBR3K1/8 b - - 0 42".to_string(),
            ],
        ));
        scenarios.push((
            "r1b2rk1/2pn1ppp/1p3q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 b - - 3 11".to_string(),
            vec![
                "r1b2r1k/2pn1ppp/1p3q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b1r1k1/2pn1ppp/1p3q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1br2k1/2pn1ppp/1p3q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r4rk1/1bpn1ppp/1p3q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r4rk1/2pn1ppp/bp3q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "1rb2rk1/2pn1ppp/1p3q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "2b2rk1/r1pn1ppp/1p3q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "2b2rk1/2pn1ppp/rp3q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "rnb2rk1/2p2ppp/1p3q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2p2ppp/1p3q2/pPn1p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1bq1rk1/2pn1ppp/1p6/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pnqppp/1p6/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pn1ppp/1p5q/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pn1ppp/1p4q1/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pn1ppp/1p2q3/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pn1ppp/1p1q4/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pn1ppp/1pq5/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pn1ppp/1p6/pP2p1q1/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pn1ppp/1p6/pP2pq2/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pn1ppp/1p6/pP2p3/2B1P2q/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pn1ppp/1p6/pP2p3/2B1Pq2/2P1QN2/P4PPP/R4RK1 w - - 4 12".to_string(),
                "r1b2rk1/2pn1ppp/1p6/pP2p3/2B1P3/2P1Qq2/P4PPP/R4RK1 w - - 0 12".to_string(),
                "r1b2rk1/2pn1pp1/1p3q1p/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 0 12".to_string(),
                "r1b2rk1/2pn1p1p/1p3qp1/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 0 12".to_string(),
                "r1b2rk1/3n1ppp/1pp2q2/pP2p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 0 12".to_string(),
                "r1b2rk1/2pn1ppp/1p3q2/1P2p3/p1B1P3/2P1QN2/P4PPP/R4RK1 w - - 0 12".to_string(),
                "r1b2rk1/2pn1pp1/1p3q2/pP2p2p/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 0 12".to_string(),
                "r1b2rk1/2pn1p1p/1p3q2/pP2p1p1/2B1P3/2P1QN2/P4PPP/R4RK1 w - - 0 12".to_string(),
                "r1b2rk1/3n1ppp/1p3q2/pPp1p3/2B1P3/2P1QN2/P4PPP/R4RK1 w - c6 0 12".to_string(),
            ],
        ));
        scenarios.push((
            "3R4/5p2/4b1k1/5p2/p1P2KpP/r5P1/3N4/8 b - - 3 52".to_string(),
            vec![
                "3R4/5p1k/4b3/5p2/p1P2KpP/r5P1/3N4/8 w - - 4 53".to_string(),
                "3R4/5pk1/4b3/5p2/p1P2KpP/r5P1/3N4/8 w - - 4 53".to_string(),
                "3R4/5p2/4b2k/5p2/p1P2KpP/r5P1/3N4/8 w - - 4 53".to_string(),
                "3R4/5p2/4bk2/5p2/p1P2KpP/r5P1/3N4/8 w - - 4 53".to_string(),
                "3R4/5p2/4b3/5p1k/p1P2KpP/r5P1/3N4/8 w - - 4 53".to_string(),
                "2bR4/5p2/6k1/5p2/p1P2KpP/r5P1/3N4/8 w - - 4 53".to_string(),
                "3R4/3b1p2/6k1/5p2/p1P2KpP/r5P1/3N4/8 w - - 4 53".to_string(),
                "3R4/5p2/6k1/3b1p2/p1P2KpP/r5P1/3N4/8 w - - 4 53".to_string(),
                "3R4/5p2/6k1/5p2/p1b2KpP/r5P1/3N4/8 w - - 0 53".to_string(),
                "3R4/5p2/4b1k1/5p2/p1P2KpP/6r1/3N4/8 w - - 0 53".to_string(),
                "3R4/5p2/4b1k1/5p2/p1P2KpP/5rP1/3N4/8 w - - 4 53".to_string(),
                "3R4/5p2/4b1k1/5p2/p1P2KpP/4r1P1/3N4/8 w - - 4 53".to_string(),
                "3R4/5p2/4b1k1/5p2/p1P2KpP/3r2P1/3N4/8 w - - 4 53".to_string(),
                "3R4/5p2/4b1k1/5p2/p1P2KpP/2r3P1/3N4/8 w - - 4 53".to_string(),
                "3R4/5p2/4b1k1/5p2/p1P2KpP/1r4P1/3N4/8 w - - 4 53".to_string(),
                "3R4/5p2/4b1k1/5p2/p1P2KpP/6P1/r2N4/8 w - - 4 53".to_string(),
                "3R4/5p2/4b1k1/5p2/p1P2KpP/6P1/3N4/r7 w - - 4 53".to_string(),
                "3R4/8/4bpk1/5p2/p1P2KpP/r5P1/3N4/8 w - - 0 53".to_string(),
            ],
        ));
        scenarios.push((
            "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P1BPPBP/R2Q1RK1 w - - 5 11".to_string(),
            vec![
                "r4rk1/pp1nqppp/3bpnN1/P2p3b/3P4/4N1P1/1P1BPPBP/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p1N1b/3P4/4N1P1/1P1BPPBP/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P4/4NNP1/1P1BPPBP/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p1N1b/3P3N/6P1/1P1BPPBP/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2N3b/3P3N/6P1/1P1BPPBP/R2Q1RK1 b - - 0 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P2NN/6P1/1P1BPPBP/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/2NP3N/6P1/1P1BPPBP/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/6P1/1PNBPPBP/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2B3b/3P3N/4N1P1/1P1BPP1P/R2Q1RK1 b - - 0 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3PB2N/4N1P1/1P1BPP1P/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1PB/1P1BPP1P/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4NBP1/1P1BPP1P/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P1BPP1P/R2Q1RKB b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/1B1P3N/4N1P1/1P2PPBP/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/2B1N1P1/1P2PPBP/R2Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P2PPBP/R2QBRK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P2PPBP/R1BQ1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P1BPPBP/R2Q1R1K b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P1BPPBP/R2QR1K1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/Q2P3N/4N1P1/1P1BPPBP/R4RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/1Q2N1P1/1P1BPPBP/R4RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1PQBPPBP/R4RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P1BPPBP/R3QRK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P1BPPBP/R1Q2RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P1BPPBP/RQ3RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/R2P3N/4N1P1/1P1BPPBP/3Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/R3N1P1/1P1BPPBP/3Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/RP1BPPBP/3Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P1BPPBP/2RQ1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1P1/1P1BPPBP/1R1Q1RK1 b - - 6 11".to_string(),
                "r4rk1/pp1nqppp/P2bpn2/3p3b/3P3N/4N1P1/1P1BPPBP/R2Q1RK1 b - - 0 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P2PN/4N3/1P1BPPBP/R2Q1RK1 b - - 0 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4N1PP/1P1BPPB1/R2Q1RK1 b - - 0 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/4NPP1/1P1BP1BP/R2Q1RK1 b - - 0 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P3N/1P2N1P1/3BPPBP/R2Q1RK1 b - - 0 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/3P1P1N/4N1P1/1P1BP1BP/R2Q1RK1 b - - 0 11".to_string(),
                "r4rk1/pp1nqppp/3bpn2/P2p3b/1P1P3N/4N1P1/3BPPBP/R2Q1RK1 b - - 0 11".to_string(),
            ],
        ));
        scenarios.push((
            "8/P5k1/1b6/3K4/4B3/8/8/8 w - - 0 65".to_string(),
            vec![
                "8/P5k1/1b2K3/8/4B3/8/8/8 b - - 1 65".to_string(),
                "8/P5k1/1b1K4/8/4B3/8/8/8 b - - 1 65".to_string(),
                "8/P5k1/1bK5/8/4B3/8/8/8 b - - 1 65".to_string(),
                "8/P5k1/1b6/4K3/4B3/8/8/8 b - - 1 65".to_string(),
                "8/P5k1/1b6/8/2K1B3/8/8/8 b - - 1 65".to_string(),
                "8/P5kB/1b6/3K4/8/8/8/8 b - - 1 65".to_string(),
                "8/P5k1/1b4B1/3K4/8/8/8/8 b - - 1 65".to_string(),
                "8/P5k1/1b6/3K1B2/8/8/8/8 b - - 1 65".to_string(),
                "8/P5k1/1b6/3K4/8/5B2/8/8 b - - 1 65".to_string(),
                "8/P5k1/1b6/3K4/8/3B4/8/8 b - - 1 65".to_string(),
                "8/P5k1/1b6/3K4/8/8/6B1/8 b - - 1 65".to_string(),
                "8/P5k1/1b6/3K4/8/8/2B5/8 b - - 1 65".to_string(),
                "8/P5k1/1b6/3K4/8/8/8/7B b - - 1 65".to_string(),
                "8/P5k1/1b6/3K4/8/8/8/1B6 b - - 1 65".to_string(),
                "Q7/6k1/1b6/3K4/4B3/8/8/8 b - - 0 65".to_string(),
                "R7/6k1/1b6/3K4/4B3/8/8/8 b - - 0 65".to_string(),
                "B7/6k1/1b6/3K4/4B3/8/8/8 b - - 0 65".to_string(),
                "N7/6k1/1b6/3K4/4B3/8/8/8 b - - 0 65".to_string(),
            ],
        ));
        scenarios.push((
            "8/1k6/2Q3KP/3p1p2/1N1P1P2/2p5/8/7q b - - 7 72".to_string(),
            vec![
                "1k6/8/2Q3KP/3p1p2/1N1P1P2/2p5/8/7q w - - 8 73".to_string(),
                "8/k7/2Q3KP/3p1p2/1N1P1P2/2p5/8/7q w - - 8 73".to_string(),
            ],
        ));
        scenarios.push((
            "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PP1/R4RK1 w k - 0 15".to_string(),
            vec![
                "2r1kb1r/3b3p/1pBq1pp1/4p3/P3Q3/4PN1P/1B3PP1/R4RK1 b k - 0 15".to_string(),
                "2r1kb1r/3b3p/Bpnq1pp1/4p3/P3Q3/4PN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/4p3/P1B1Q3/4PN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/4p3/P3Q3/3BPN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/4p3/P3Q3/4PN1P/1B2BPP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pQ1/1B2p3/P7/4PN1P/1B3PP1/R4RK1 b k - 0 15".to_string(),
                "2r1kb1r/3b3p/1pQq1pp1/1B2p3/P7/4PN1P/1B3PP1/R4RK1 b k - 0 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2pQ2/P7/4PN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2Q3/P7/4PN1P/1B3PP1/R4RK1 b k - 0 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B1Qp3/P7/4PN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P6Q/4PN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P5Q1/4PN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P4Q2/4PN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P2Q4/4PN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P1Q5/4PN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/PQ6/4PN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P7/3QPN1P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P7/4PN1P/1BQ2PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P7/4PN1P/1B3PP1/RQ3RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p1N1/P3Q3/4P2P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2N3/P3Q3/4P2P/1B3PP1/R4RK1 b k - 0 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q2N/4P2P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P2NQ3/4P2P/1B3PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4P2P/1B3PPN/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4P2P/1B1N1PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4P2P/1B3PP1/R3NRK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2B3/P3Q3/4PN1P/5PP1/R4RK1 b k - 0 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P2BQ3/4PN1P/5PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/2B1PN1P/5PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/B3PN1P/5PP1/R4RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/5PP1/R1B2RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PPK/R4R2 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PP1/R4R1K b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PP1/R3R1K1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PP1/R2R2K1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PP1/R1R3K1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PP1/RR4K1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/R3PN1P/1B3PP1/5RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/RB3PP1/5RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PP1/4RRK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PP1/3R1RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PP1/2R2RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PN1P/1B3PP1/1R3RK1 b k - 1 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/PB2p3/4Q3/4PN1P/1B3PP1/R4RK1 b k - 0 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q2P/4PN2/1B3PP1/R4RK1 b k - 0 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q3/4PNPP/1B3P2/R4RK1 b k - 0 15".to_string(),
                "2r1kb1r/3b3p/1pnq1pp1/1B2p3/P3Q1P1/4PN1P/1B3P2/R4RK1 b k - 0 15".to_string(),
            ],
        ));
        scenarios.push((
            "2R5/8/8/5P1p/8/k4K2/8/2q3r1 w - - 0 67".to_string(),
            vec![
                "7R/8/8/5P1p/8/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "6R1/8/8/5P1p/8/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "5R2/8/8/5P1p/8/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "4R3/8/8/5P1p/8/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "3R4/8/8/5P1p/8/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "1R6/8/8/5P1p/8/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "R7/8/8/5P1p/8/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "8/2R5/8/5P1p/8/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "8/8/2R5/5P1p/8/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "8/8/8/2R2P1p/8/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "8/8/8/5P1p/2R5/k4K2/8/2q3r1 b - - 1 67".to_string(),
                "8/8/8/5P1p/8/k1R2K2/8/2q3r1 b - - 1 67".to_string(),
                "8/8/8/5P1p/8/k4K2/2R5/2q3r1 b - - 1 67".to_string(),
                "8/8/8/5P1p/8/k4K2/8/2R3r1 b - - 0 67".to_string(),
                "2R5/8/8/5P1p/4K3/k7/8/2q3r1 b - - 1 67".to_string(),
                "2R5/8/8/5P1p/8/k7/5K2/2q3r1 b - - 1 67".to_string(),
                "2R5/8/8/5P1p/8/k7/4K3/2q3r1 b - - 1 67".to_string(),
                "2R5/8/5P2/7p/8/k4K2/8/2q3r1 b - - 0 67".to_string(),
            ],
        ));
        scenarios.push((
            "3qr1k1/3n1pp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K b - - 0 24".to_string(),
            vec![
                "3qr2k/3n1pp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qrk2/3n1pp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr3/3n1ppk/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3q1rk1/3n1pp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3q2k1/3nrpp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3q2k1/3n1pp1/2r1rn2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "2q1r1k1/3n1pp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "1q2r1k1/3n1pp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "q3r1k1/3n1pp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "4r1k1/3nqpp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "4r1k1/2qn1pp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "4r1k1/3n1pp1/1qr2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qrnk1/5pp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "1n1qr1k1/5pp1/2r2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr1k1/5pp1/1nr2n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr1k1/3n1ppn/2r5/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr1k1/3n1pp1/2r5/p1pnpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 0 25".to_string(),
                "3qr1k1/3n1pp1/2r5/p1pNpP1p/PpP1P1n1/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr1k1/3n1pp1/2r5/p1pNpP1p/PpP1n3/1P4PP/3N1Q2/3RR2K w - - 0 25".to_string(),
                "2rqr1k1/3n1pp1/5n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr1k1/2rn1pp1/5n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr1k1/3n1pp1/4rn2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr1k1/3n1pp1/3r1n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr1k1/3n1pp1/1r3n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr1k1/3n1pp1/r4n2/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 1 25".to_string(),
                "3qr1k1/3n1p2/2r2np1/p1pNpP1p/PpP1P3/1P4PP/3N1Q2/3RR2K w - - 0 25".to_string(),
                "3qr1k1/3n1pp1/2r2n2/p1pNpP2/PpP1P2p/1P4PP/3N1Q2/3RR2K w - - 0 25".to_string(),
                "3qr1k1/3n1p2/2r2n2/p1pNpPpp/PpP1P3/1P4PP/3N1Q2/3RR2K w - g6 0 25".to_string(),
            ],
        ));
        scenarios.push((
            "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/6PP/R1Q1B1K1 w - - 4 25".to_string(),
            vec![
                "1r1q2k1/2p2b1p/2PbN1p1/pP3p2/P3p3/2P1P3/6PP/R1Q1B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3N2/P3p3/2P1P3/6PP/R1Q1B1K1 b - - 0 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P3p3/2P1PN2/6PP/R1Q1B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P3p3/1NP1P3/6PP/R1Q1B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P3p3/2P1P3/4N1PP/R1Q1B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P3p3/2P1P3/2N3PP/R1Q1B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/5KPP/R1Q1B3 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/6PP/R1Q1B2K b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/6PP/R1Q1BK2 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np2B/2P1P3/6PP/R1Q3K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P1B1/6PP/R1Q3K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/5BPP/R1Q3K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/3B2PP/R1Q3K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/Q1P1P3/6PP/R3B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/3Q2PP/R3B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/2Q3PP/R3B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/1Q4PP/R3B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/6PP/R2QB1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/6PP/RQ2B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/R1P1P3/6PP/2Q1B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/R5PP/2Q1B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P3/6PP/1RQ1B1K1 b - - 5 25".to_string(),
                "1r1q2k1/2p2b1p/1PPb2p1/p4p2/P2Np3/2P1P3/6PP/R1Q1B1K1 b - - 0 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P1PNp3/4P3/6PP/R1Q1B1K1 b - - 0 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P2P/6P1/R1Q1B1K1 b - - 0 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np3/2P1P1P1/7P/R1Q1B1K1 b - - 0 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np2P/2P1P3/6P1/R1Q1B1K1 b - - 0 25".to_string(),
                "1r1q2k1/2p2b1p/2Pb2p1/pP3p2/P2Np1P1/2P1P3/7P/R1Q1B1K1 b - - 0 25".to_string(),
            ],
        ));
        scenarios.push((
            "r4rk1/1bq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 b - - 5 16".to_string(),
            vec![
                "r4r1k/1bq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r3r1k1/1bq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r2r2k1/1bq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r1r3k1/1bq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "rr4k1/1bq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "4rrk1/1bq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "3r1rk1/1bq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "2r2rk1/1bq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "1r3rk1/1bq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "5rk1/rbq1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r2b1rk1/1bq2ppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1bq2ppp/p3pb2/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1bq2ppp/p2bp3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1bq2ppp/p3p3/1pnpP1b1/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1bq2ppp/p3p3/1pnpP3/3N1B1b/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r2q1rk1/1b2bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r1q2rk1/1b2bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "rq3rk1/1b2bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1b1qbppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1b2bppp/p2qp3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1b2bppp/p1q1p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1b2bppp/pq2p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1b2bppp/p3p3/1pnpq3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 0 17".to_string(),
                "r4rk1/1b2bppp/p3p3/qpnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r1b2rk1/2q1bppp/p3p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/2q1bppp/p1b1p3/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1bqnbppp/p3p3/1p1pP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1bq1bppp/p3p3/1p1pP3/3NnB2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1bq1bppp/p3p3/1p1pP3/n2N1B2/4Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1bq1bppp/p3p3/1p1pP3/3N1B2/3nQ3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1bq1bppp/p3p3/1p1pP3/3N1B2/1n2Q3/1PP1BPPP/RR4K1 w - - 6 17".to_string(),
                "r4rk1/1bq1bpp1/p3p2p/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 0 17".to_string(),
                "r4rk1/1bq1bp1p/p3p1p1/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 0 17".to_string(),
                "r4rk1/1bq1b1pp/p3pp2/1pnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 0 17".to_string(),
                "r4rk1/1bq1bppp/4p3/ppnpP3/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 0 17".to_string(),
                "r4rk1/1bq1bppp/p3p3/2npP3/1p1N1B2/4Q3/1PP1BPPP/RR4K1 w - - 0 17".to_string(),
                "r4rk1/1bq1bpp1/p3p3/1pnpP2p/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 0 17".to_string(),
                "r4rk1/1bq1bp1p/p3p3/1pnpP1p1/3N1B2/4Q3/1PP1BPPP/RR4K1 w - - 0 17".to_string(),
                "r4rk1/1bq1b1pp/p3p3/1pnpPp2/3N1B2/4Q3/1PP1BPPP/RR4K1 w - f6 0 17".to_string(),
            ],
        ));
        scenarios.push((
            "5rk1/p2nbppp/bp2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 b - - 1 13".to_string(),
            vec![
                "5r1k/p2nbppp/bp2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "4r1k1/p2nbppp/bp2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "3r2k1/p2nbppp/bp2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "2r3k1/p2nbppp/bp2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "1r4k1/p2nbppp/bp2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "r5k1/p2nbppp/bp2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "3b1rk1/p2n1ppp/bp2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p2n1ppp/bp2pb2/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p2n1ppp/bp1bp3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p2n1ppp/bp2p3/6b1/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p2n1ppp/bp2p3/2b5/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p2n1ppp/bp2p3/8/2p1PB1b/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p2n1ppp/bp2p3/8/1bp1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p2n1ppp/bp2p3/8/2p1PB2/b5PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "1n3rk1/p3bppp/bp2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p3bppp/bp2pn2/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p3bppp/bp2p3/4n3/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p3bppp/bp2p3/2n5/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "2b2rk1/p2nbppp/1p2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/pb1nbppp/1p2p3/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p2nbppp/1p2p3/1b6/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 2 14".to_string(),
                "5rk1/p2nbpp1/bp2p2p/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 0 14".to_string(),
                "5rk1/p2nbp1p/bp2p1p1/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 0 14".to_string(),
                "5rk1/p2nb1pp/bp2pp2/8/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 0 14".to_string(),
                "5rk1/p2nbppp/bp6/4p3/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 0 14".to_string(),
                "5rk1/p2nbppp/b3p3/1p6/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 0 14".to_string(),
                "5rk1/p2nbppp/bp2p3/8/4PB2/2p3PP/PPP2PB1/1N2R1K1 w - - 0 14".to_string(),
                "5rk1/p2nbpp1/bp2p3/7p/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 0 14".to_string(),
                "5rk1/p2nbp1p/bp2p3/6p1/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 0 14".to_string(),
                "5rk1/p2nb1pp/bp2p3/5p2/2p1PB2/6PP/PPP2PB1/1N2R1K1 w - - 0 14".to_string(),
            ],
        ));
        scenarios.push((
            "r1bqkb1r/1pp2pp1/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R b KQkq - 2 3".to_string(),
            vec![
                "r1bqkbr1/1pp2pp1/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQq - 3 4".to_string(),
                "r1bqkb2/1pp2ppr/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQq - 3 4".to_string(),
                "r1bqk2r/1pp1bpp1/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "r1bqk2r/1pp2pp1/p1nbpn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4".to_string(),
                "r1bqk2r/1pp2pp1/p1n1pn1p/P1bp4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "r1bqk2r/1pp2pp1/p1n1pn1p/P2p4/1b1P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "r1bqk2r/1pp2pp1/p1n1pn1p/P2p4/3P4/bP2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4".to_string(),
                "r1bq1b1r/1pp1kpp1/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQ - 3 4".to_string(),
                "r1bq1b1r/1ppk1pp1/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQ - 3 4".to_string(),
                "r1b1kb1r/1pp1qpp1/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "r1b1kb1r/1ppq1pp1/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "r1b1kb1r/1pp2pp1/p1nqpn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "r2qkb1r/1ppb1pp1/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "1rbqkb1r/1pp2pp1/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQk - 3 4".to_string(),
                "2bqkb1r/rpp2pp1/p1n1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQk - 3 4".to_string(),
                "r1bqkbnr/1pp2pp1/p1n1p2p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4".to_string(),
                "r1bqkb1r/1pp2ppn/p1n1p2p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4".to_string(),
                "r1bqkb1r/1ppn1pp1/p1n1p2p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "r1bqkb1r/1pp2pp1/p1n1p2p/P2p3n/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "r1bqkb1r/1pp2pp1/p1n1p2p/P2p4/3P2n1/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "r1bqkb1r/1pp2pp1/p1n1p2p/P2p4/3Pn3/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "rnbqkb1r/1pp2pp1/p3pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4".to_string(),
                "r1bqkb1r/1pp1npp1/p3pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4".to_string(),
                "r1bqkb1r/npp2pp1/p3pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4".to_string(),
                "r1bqkb1r/1pp2pp1/p3pn1p/P2pn3/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4".to_string(),
                "r1bqkb1r/1pp2pp1/p3pn1p/n2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 0 4".to_string(),
                "r1bqkb1r/1pp2pp1/p3pn1p/P2p4/3n4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 0 4".to_string(),
                "r1bqkb1r/1pp2pp1/p3pn1p/P2p4/1n1P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 3 4"
                    .to_string(),
                "r1bqkb1r/1pp2p2/p1n1pnpp/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 0 4".to_string(),
                "r1bqkb1r/2p2pp1/ppn1pn1p/P2p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 0 4".to_string(),
                "r1bqkb1r/1pp2pp1/p1n1pn2/P2p3p/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 0 4"
                    .to_string(),
                "r1bqkb1r/1pp2pp1/p1n2n1p/P2pp3/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 0 4"
                    .to_string(),
                "r1bqkb1r/1pp2p2/p1n1pn1p/P2p2p1/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq - 0 4"
                    .to_string(),
                "r1bqkb1r/2p2pp1/p1n1pn1p/Pp1p4/3P4/1P2PN2/2P1BPPP/RNBQK2R w KQkq b6 0 4"
                    .to_string(),
            ],
        ));
        scenarios.push((
            "8/5kp1/5rr1/1p3QP1/8/8/6K1/8 w - - 15 61".to_string(),
            vec![
                "2Q5/5kp1/5rr1/1p4P1/8/8/6K1/8 b - - 16 61".to_string(),
                "8/3Q1kp1/5rr1/1p4P1/8/8/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rQ1/1p4P1/8/8/6K1/8 b - - 0 61".to_string(),
                "8/5kp1/5Qr1/1p4P1/8/8/6K1/8 b - - 0 61".to_string(),
                "8/5kp1/4Qrr1/1p4P1/8/8/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p2Q1P1/8/8/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p1Q2P1/8/8/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1pQ3P1/8/8/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1Q4P1/8/8/6K1/8 b - - 0 61".to_string(),
                "8/5kp1/5rr1/1p4P1/6Q1/8/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p4P1/5Q2/8/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p4P1/4Q3/8/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p4P1/8/7Q/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p4P1/8/5Q2/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p4P1/8/3Q4/6K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p4P1/8/8/5QK1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p4P1/8/8/2Q3K1/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p4P1/8/8/6K1/5Q2 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p4P1/8/8/6K1/1Q6 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p3QP1/8/7K/8/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p3QP1/8/6K1/8/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p3QP1/8/5K2/8/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p3QP1/8/8/7K/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p3QP1/8/8/5K2/8 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p3QP1/8/8/8/7K b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p3QP1/8/8/8/6K1 b - - 16 61".to_string(),
                "8/5kp1/5rr1/1p3QP1/8/8/8/5K2 b - - 16 61".to_string(),
            ],
        ));
        scenarios.push((
            "8/5p2/B2k4/n4K2/8/6P1/8/8 b - - 1 49".to_string(),
            vec![
                "8/4kp2/B7/n4K2/8/6P1/8/8 w - - 2 50".to_string(),
                "8/3k1p2/B7/n4K2/8/6P1/8/8 w - - 2 50".to_string(),
                "8/2k2p2/B7/n4K2/8/6P1/8/8 w - - 2 50".to_string(),
                "8/5p2/B1k5/n4K2/8/6P1/8/8 w - - 2 50".to_string(),
                "8/5p2/B7/n2k1K2/8/6P1/8/8 w - - 2 50".to_string(),
                "8/5p2/B7/n1k2K2/8/6P1/8/8 w - - 2 50".to_string(),
                "8/1n3p2/B2k4/5K2/8/6P1/8/8 w - - 2 50".to_string(),
                "8/5p2/B1nk4/5K2/8/6P1/8/8 w - - 2 50".to_string(),
                "8/5p2/B2k4/5K2/2n5/6P1/8/8 w - - 2 50".to_string(),
                "8/5p2/B2k4/5K2/8/1n4P1/8/8 w - - 2 50".to_string(),
                "8/8/B2k1p2/n4K2/8/6P1/8/8 w - - 0 50".to_string(),
            ],
        ));
        scenarios.push((
            "8/3k4/8/1R1P1K2/1p6/3n4/8/8 w - - 3 68".to_string(),
            vec![
                "8/3k4/6K1/1R1P4/1p6/3n4/8/8 b - - 4 68".to_string(),
                "8/3k4/5K2/1R1P4/1p6/3n4/8/8 b - - 4 68".to_string(),
                "8/3k4/8/1R1P2K1/1p6/3n4/8/8 b - - 4 68".to_string(),
                "8/3k4/8/1R1P4/1p4K1/3n4/8/8 b - - 4 68".to_string(),
                "8/3k4/8/1R1P4/1p2K3/3n4/8/8 b - - 4 68".to_string(),
                "1R6/3k4/8/3P1K2/1p6/3n4/8/8 b - - 4 68".to_string(),
                "8/1R1k4/8/3P1K2/1p6/3n4/8/8 b - - 4 68".to_string(),
                "8/3k4/1R6/3P1K2/1p6/3n4/8/8 b - - 4 68".to_string(),
                "8/3k4/8/2RP1K2/1p6/3n4/8/8 b - - 4 68".to_string(),
                "8/3k4/8/R2P1K2/1p6/3n4/8/8 b - - 4 68".to_string(),
                "8/3k4/8/3P1K2/1R6/3n4/8/8 b - - 0 68".to_string(),
                "8/3k4/3P4/1R3K2/1p6/3n4/8/8 b - - 0 68".to_string(),
            ],
        ));
        scenarios.push((
            "1R6/8/5K2/nk1p4/3Pb3/7r/1B6/8 b - - 5 70".to_string(),
            vec![
                "1R6/8/2k2K2/n2p4/3Pb3/7r/1B6/8 w - - 6 71".to_string(),
                "1R6/8/k4K2/n2p4/3Pb3/7r/1B6/8 w - - 6 71".to_string(),
                "1R6/8/5K2/n2p4/2kPb3/7r/1B6/8 w - - 6 71".to_string(),
                "1R6/8/5K2/n2p4/k2Pb3/7r/1B6/8 w - - 6 71".to_string(),
                "1R6/1n6/5K2/1k1p4/3Pb3/7r/1B6/8 w - - 6 71".to_string(),
            ],
        ));
        scenarios.push((
            "2r2k2/5p1p/pR4p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 b - - 2 28".to_string(),
            vec![
                "2r3k1/5p1p/pR4p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r1k3/5p1p/pR4p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r5/5pkp/pR4p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r5/4kp1p/pR4p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "4rk2/5p1p/pR4p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "3r1k2/5p1p/pR4p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "1r3k2/5p1p/pR4p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "r4k2/5p1p/pR4p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "5k2/2r2p1p/pR4p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "5k2/5p1p/pRr3p1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2rr1k2/5p1p/pR4p1/n1n1P3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/3r1p1p/pR4p1/n1n1P3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR1r2p1/n1n1P3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR4p1/n1n1r3/2p2PPP/N1P1BK2/1PB5/8 w - - 0 29".to_string(),
                "2r2k2/5p1p/pR4p1/n1n1P3/2pr1PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR4p1/n1n1P3/2p2PPP/N1PrBK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR4p1/n1n1P3/2p2PPP/N1P1BK2/1PBr4/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR4p1/n1n1P3/2p2PPP/N1P1BK2/1PB5/3r4 w - - 3 29".to_string(),
                "2r2k2/3n1p1p/pR4p1/n2rP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/1n3p1p/pR4p1/n2rP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR2n1p1/n2rP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR4p1/n2rP3/2p1nPPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR4p1/n2rP3/n1p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR4p1/n2rP3/2p2PPP/N1PnBK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR4p1/n2rP3/2p2PPP/NnP1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/1n3p1p/pR4p1/2nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pRn3p1/2nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p1p/pR4p1/2nrP3/2p2PPP/NnP1BK2/1PB5/8 w - - 3 29".to_string(),
                "2r2k2/5p2/pR4pp/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 0 29".to_string(),
                "2r2k2/7p/pR3pp1/n1nrP3/2p2PPP/N1P1BK2/1PB5/8 w - - 0 29".to_string(),
                "2r2k2/5p1p/pR6/n1nrP1p1/2p2PPP/N1P1BK2/1PB5/8 w - - 0 29".to_string(),
                "2r2k2/5p2/pR4p1/n1nrP2p/2p2PPP/N1P1BK2/1PB5/8 w - - 0 29".to_string(),
                "2r2k2/7p/pR4p1/n1nrPp2/2p2PPP/N1P1BK2/1PB5/8 w - f6 0 29".to_string(),
            ],
        ));
        scenarios.push((
            "8/5k2/7R/2N5/8/4K3/8/3r4 b - - 74 132".to_string(),
            vec![
                "6k1/8/7R/2N5/8/4K3/8/3r4 w - - 75 133".to_string(),
                "5k2/8/7R/2N5/8/4K3/8/3r4 w - - 75 133".to_string(),
                "4k3/8/7R/2N5/8/4K3/8/3r4 w - - 75 133".to_string(),
                "8/6k1/7R/2N5/8/4K3/8/3r4 w - - 75 133".to_string(),
                "8/4k3/7R/2N5/8/4K3/8/3r4 w - - 75 133".to_string(),
                "3r4/5k2/7R/2N5/8/4K3/8/8 w - - 75 133".to_string(),
                "8/3r1k2/7R/2N5/8/4K3/8/8 w - - 75 133".to_string(),
                "8/5k2/3r3R/2N5/8/4K3/8/8 w - - 75 133".to_string(),
                "8/5k2/7R/2Nr4/8/4K3/8/8 w - - 75 133".to_string(),
                "8/5k2/7R/2N5/3r4/4K3/8/8 w - - 75 133".to_string(),
                "8/5k2/7R/2N5/8/3rK3/8/8 w - - 75 133".to_string(),
                "8/5k2/7R/2N5/8/4K3/3r4/8 w - - 75 133".to_string(),
                "8/5k2/7R/2N5/8/4K3/8/7r w - - 75 133".to_string(),
                "8/5k2/7R/2N5/8/4K3/8/6r1 w - - 75 133".to_string(),
                "8/5k2/7R/2N5/8/4K3/8/5r2 w - - 75 133".to_string(),
                "8/5k2/7R/2N5/8/4K3/8/4r3 w - - 75 133".to_string(),
                "8/5k2/7R/2N5/8/4K3/8/2r5 w - - 75 133".to_string(),
                "8/5k2/7R/2N5/8/4K3/8/1r6 w - - 75 133".to_string(),
                "8/5k2/7R/2N5/8/4K3/8/r7 w - - 75 133".to_string(),
            ],
        ));
        scenarios.push((
            "8/3b2r1/R3p3/1P1k4/P7/8/1r6/4R2K w - - 5 44".to_string(),
            vec![
                "R7/3b2r1/4p3/1P1k4/P7/8/1r6/4R2K b - - 6 44".to_string(),
                "8/R2b2r1/4p3/1P1k4/P7/8/1r6/4R2K b - - 6 44".to_string(),
                "8/3b2r1/4R3/1P1k4/P7/8/1r6/4R2K b - - 0 44".to_string(),
                "8/3b2r1/3Rp3/1P1k4/P7/8/1r6/4R2K b - - 6 44".to_string(),
                "8/3b2r1/2R1p3/1P1k4/P7/8/1r6/4R2K b - - 6 44".to_string(),
                "8/3b2r1/1R2p3/1P1k4/P7/8/1r6/4R2K b - - 6 44".to_string(),
                "8/3b2r1/4p3/RP1k4/P7/8/1r6/4R2K b - - 6 44".to_string(),
                "8/3b2r1/R3R3/1P1k4/P7/8/1r6/7K b - - 0 44".to_string(),
                "8/3b2r1/R3p3/1P1kR3/P7/8/1r6/7K b - - 6 44".to_string(),
                "8/3b2r1/R3p3/1P1k4/P3R3/8/1r6/7K b - - 6 44".to_string(),
                "8/3b2r1/R3p3/1P1k4/P7/4R3/1r6/7K b - - 6 44".to_string(),
                "8/3b2r1/R3p3/1P1k4/P7/8/1r2R3/7K b - - 6 44".to_string(),
                "8/3b2r1/R3p3/1P1k4/P7/8/1r6/6RK b - - 6 44".to_string(),
                "8/3b2r1/R3p3/1P1k4/P7/8/1r6/5R1K b - - 6 44".to_string(),
                "8/3b2r1/R3p3/1P1k4/P7/8/1r6/3R3K b - - 6 44".to_string(),
                "8/3b2r1/R3p3/1P1k4/P7/8/1r6/2R4K b - - 6 44".to_string(),
                "8/3b2r1/R3p3/1P1k4/P7/8/1r6/1R5K b - - 6 44".to_string(),
                "8/3b2r1/R3p3/1P1k4/P7/8/1r6/R6K b - - 6 44".to_string(),
                "8/3b2r1/RP2p3/3k4/P7/8/1r6/4R2K b - - 0 44".to_string(),
                "8/3b2r1/R3p3/PP1k4/8/8/1r6/4R2K b - - 0 44".to_string(),
            ],
        ));
        scenarios.push((
            "6k1/5pp1/8/2NR4/pr6/6P1/7P/1n5K w - - 2 49".to_string(),
            vec![
                "3R2k1/5pp1/8/2N5/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/3R1pp1/8/2N5/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/3R4/2N5/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/2N4R/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/2N3R1/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/2N2R2/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/2N1R3/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/2N5/pr1R4/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/2N5/pr6/3R2P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/2N5/pr6/6P1/3R3P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/2N5/pr6/6P1/7P/1n1R3K b - - 3 49".to_string(),
                "6k1/3N1pp1/8/3R4/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/1N3pp1/8/3R4/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/4N3/3R4/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/N7/3R4/pr6/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/3R4/pr2N3/6P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/3R4/Nr6/6P1/7P/1n5K b - - 0 49".to_string(),
                "6k1/5pp1/8/3R4/pr6/3N2P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/3R4/pr6/1N4P1/7P/1n5K b - - 3 49".to_string(),
                "6k1/5pp1/8/2NR4/pr6/6P1/6KP/1n6 b - - 3 49".to_string(),
                "6k1/5pp1/8/2NR4/pr6/6P1/7P/1n4K1 b - - 3 49".to_string(),
                "6k1/5pp1/8/2NR4/pr4P1/8/7P/1n5K b - - 0 49".to_string(),
                "6k1/5pp1/8/2NR4/pr6/6PP/8/1n5K b - - 0 49".to_string(),
                "6k1/5pp1/8/2NR4/pr5P/6P1/8/1n5K b - - 0 49".to_string(),
            ],
        ));
        scenarios.push((
            "2r4k/N4ppp/4pb2/1R6/2n5/6P1/4RPKP/8 b - - 2 35".to_string(),
            vec![
                "2r3k1/N4ppp/4pb2/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "6rk/N4ppp/4pb2/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "5r1k/N4ppp/4pb2/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "4r2k/N4ppp/4pb2/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "3r3k/N4ppp/4pb2/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "1r5k/N4ppp/4pb2/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "r6k/N4ppp/4pb2/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "7k/N1r2ppp/4pb2/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "7k/N4ppp/2r1pb2/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "7k/N4ppp/4pb2/1Rr5/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2rb3k/N4ppp/4p3/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N3bppp/4p3/1R6/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4p3/1R4b1/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4p3/1R2b3/2n5/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4p3/1R6/2n4b/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4p3/1R6/2nb4/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4p3/1R6/2n5/2b3P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4p3/1R6/2n5/6P1/1b2RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4p3/1R6/2n5/6P1/4RPKP/b7 w - - 3 36".to_string(),
                "2r4k/N4ppp/3npb2/1R6/8/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/1n2pb2/1R6/8/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4pb2/1R2n3/8/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4pb2/nR6/8/6P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4pb2/1R6/8/4n1P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4pb2/1R6/8/n5P1/4RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4pb2/1R6/8/6P1/3nRPKP/8 w - - 3 36".to_string(),
                "2r4k/N4ppp/4pb2/1R6/8/6P1/1n2RPKP/8 w - - 3 36".to_string(),
                "2r4k/N4pp1/4pb1p/1R6/2n5/6P1/4RPKP/8 w - - 0 36".to_string(),
                "2r4k/N4p1p/4pbp1/1R6/2n5/6P1/4RPKP/8 w - - 0 36".to_string(),
                "2r4k/N4ppp/5b2/1R2p3/2n5/6P1/4RPKP/8 w - - 0 36".to_string(),
                "2r4k/N4pp1/4pb2/1R5p/2n5/6P1/4RPKP/8 w - - 0 36".to_string(),
                "2r4k/N4p1p/4pb2/1R4p1/2n5/6P1/4RPKP/8 w - - 0 36".to_string(),
            ],
        ));
        scenarios.push((
            "2rr2k1/p1q1npp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 b - - 0 18".to_string(),
            vec![
                "2rr3k/p1q1npp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr1k2/p1q1npp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr4/p1q1nppk/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2r2rk1/p1q1npp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2r1r1k1/p1q1npp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2r3k1/p1qrnpp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2r3k1/p1q1npp1/1p1rp2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2r3k1/p1q1npp1/1p2p2p/3rP3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2r3k1/p1q1npp1/1p2p2p/4P3/PP1rR3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2r3k1/p1q1npp1/1p2p2p/4P3/PP2R3/2Pr4/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2r3k1/p1q1npp1/1p2p2p/4P3/PP2R3/2P5/1BQr1PPP/2R3K1 w - - 1 19".to_string(),
                "2r3k1/p1q1npp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2Rr2K1 w - - 1 19".to_string(),
                "1r1r2k1/p1q1npp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "r2r2k1/p1q1npp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/p1q2pp1/1p2p1np/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/p1q2pp1/1pn1p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/p1q2pp1/1p2p2p/4Pn2/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/p1q2pp1/1p2p2p/3nP3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "1qrr2k1/p3npp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/p2qnpp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/pq2npp1/1p2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/p3npp1/1p1qp2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/p3npp1/1pq1p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/p3npp1/1p2p2p/4q3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 0 19".to_string(),
                "2rr2k1/p3npp1/1p2p2p/2q1P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/p3npp1/1p2p2p/4P3/PPq1R3/2P5/1BQ2PPP/2R3K1 w - - 1 19".to_string(),
                "2rr2k1/p3npp1/1p2p2p/4P3/PP2R3/2q5/1BQ2PPP/2R3K1 w - - 0 19".to_string(),
                "2rr2k1/p1q1np2/1p2p1pp/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 0 19".to_string(),
                "2rr2k1/p1q1n1p1/1p2pp1p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 0 19".to_string(),
                "2rr2k1/2q1npp1/pp2p2p/4P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 0 19".to_string(),
                "2rr2k1/p1q1npp1/1p2p3/4P2p/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 0 19".to_string(),
                "2rr2k1/p1q1npp1/4p2p/1p2P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 0 19".to_string(),
                "2rr2k1/p1q1np2/1p2p2p/4P1p1/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 0 19".to_string(),
                "2rr2k1/p1q1n1p1/1p2p2p/4Pp2/PP2R3/2P5/1BQ2PPP/2R3K1 w - f6 0 19".to_string(),
                "2rr2k1/2q1npp1/1p2p2p/p3P3/PP2R3/2P5/1BQ2PPP/2R3K1 w - - 0 19".to_string(),
            ],
        ));
        scenarios.push((
            "8/8/5ppk/4b2p/2r4P/4RBPK/8/8 b - - 5 60".to_string(),
            vec![
                "8/7k/5pp1/4b2p/2r4P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/6k1/5pp1/4b2p/2r4P/4RBPK/8/8 w - - 6 61".to_string(),
                "1b6/8/5ppk/7p/2r4P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/2b5/5ppk/7p/2r4P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/3b1ppk/7p/2r4P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/7p/2r2b1P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/7p/2rb3P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/7p/2r4P/4RBbK/8/8 w - - 0 61".to_string(),
                "8/8/5ppk/7p/2r4P/2b1RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/7p/2r4P/4RBPK/1b6/8 w - - 6 61".to_string(),
                "8/8/5ppk/7p/2r4P/4RBPK/8/b7 w - - 6 61".to_string(),
                "2r5/8/5ppk/4b2p/7P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/2r5/5ppk/4b2p/7P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/2r2ppk/4b2p/7P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/2r1b2p/7P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/4b2p/7r/4RBPK/8/8 w - - 0 61".to_string(),
                "8/8/5ppk/4b2p/6rP/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/4b2p/5r1P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/4b2p/4r2P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/4b2p/3r3P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/4b2p/1r5P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/4b2p/r6P/4RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/4b2p/7P/2r1RBPK/8/8 w - - 6 61".to_string(),
                "8/8/5ppk/4b2p/7P/4RBPK/2r5/8 w - - 6 61".to_string(),
                "8/8/5ppk/4b2p/7P/4RBPK/8/2r5 w - - 6 61".to_string(),
                "8/8/5p1k/4b1pp/2r4P/4RBPK/8/8 w - - 0 61".to_string(),
                "8/8/6pk/4bp1p/2r4P/4RBPK/8/8 w - - 0 61".to_string(),
            ],
        ));
        scenarios.push((
            "r3r1k1/1ppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 b - - 5 11".to_string(),
            vec![
                "r3r2k/1ppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12".to_string(),
                "r3rk2/1ppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12".to_string(),
                "r3r3/1ppqbppk/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12".to_string(),
                "r4rk1/1ppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12".to_string(),
                "r2r2k1/1ppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r1r3k1/1ppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "rr4k1/1ppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12".to_string(),
                "3rr1k1/1ppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "2r1r1k1/1ppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "1r2r1k1/1ppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "4r1k1/rppqbpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12".to_string(),
                "r3rbk1/1ppq1pp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r2br1k1/1ppq1pp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r2qr1k1/1pp1bpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r1q1r1k1/1pp1bpp1/p1np1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1pp1bpp1/p1npqnbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1pp1bpp1/p1np1nbp/4pq2/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1pp1bpp1/p1np1nbp/4p3/1PP1N1q1/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1pp1bpp1/p1np1nbp/4p3/1PP1N3/P2PPN1q/1BQ1BPP1/2RR2K1 w - - 0 12"
                    .to_string(),
                "r3r1k1/1ppqbppb/p1np1n1p/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1ppqbpp1/p1np1n1p/4p2b/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1ppqbpp1/p1np1n1p/4pb2/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1ppqbpp1/p1np1n1p/4p3/1PP1b3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 0 12"
                    .to_string(),
                "r3r1k1/1ppqbppn/p1np2bp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12".to_string(),
                "r3r1k1/1ppqbpp1/p1np2bp/4p2n/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1ppqbpp1/p1np2bp/3np3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1ppqbpp1/p1np2bp/4p3/1PP1N1n1/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1ppqbpp1/p1np2bp/4p3/1PP1n3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 0 12".to_string(),
                "r2nr1k1/1ppqbpp1/p2p1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "rn2r1k1/1ppqbpp1/p2p1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/nppqbpp1/p2p1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12".to_string(),
                "r3r1k1/1ppqbpp1/p2p1nbp/n3p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12"
                    .to_string(),
                "r3r1k1/1ppqbpp1/p2p1nbp/4p3/1PPnN3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 6 12".to_string(),
                "r3r1k1/1ppqbpp1/p2p1nbp/4p3/1nP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 0 12".to_string(),
                "r3r1k1/2pqbpp1/ppnp1nbp/4p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 0 12".to_string(),
                "r3r1k1/1ppqbpp1/p1np1nb1/4p2p/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 0 12"
                    .to_string(),
                "r3r1k1/1ppqbpp1/p1n2nbp/3pp3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 0 12"
                    .to_string(),
                "r3r1k1/1ppqbpp1/2np1nbp/p3p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 0 12"
                    .to_string(),
                "r3r1k1/2pqbpp1/p1np1nbp/1p2p3/1PP1N3/P2PPN1P/1BQ1BPP1/2RR2K1 w - - 0 12"
                    .to_string(),
            ],
        ));
        scenarios.push((
            "rr4k1/1p3pp1/2p5/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 b - - 7 19".to_string(),
            vec![
                "rr5k/1p3pp1/2p5/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr6/1p3ppk/2p5/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "r4rk1/1p3pp1/2p5/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "r3r1k1/1p3pp1/2p5/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "r2r2k1/1p3pp1/2p5/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "r1r3k1/1p3pp1/2p5/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "1r4k1/rp3pp1/2p5/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "1r4k1/1p3pp1/r1p5/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr1b2k1/1p3pp1/2p5/2B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1pb2pp1/2p5/2B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/1bp5/2B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/2B5/1bqP1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/2B5/2qP1Q2/2b1PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/2B5/2qP1Q2/4PN1P/3b1PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/2B5/2qP1Q2/4PN1P/5PP1/4bRK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p1q3/b1B5/3P1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/q1p5/b1B5/3P1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1Bq4/3P1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1q5/3P1Q2/4PN1P/5PP1/5RK1 w - - 0 20".to_string(),
                "rr4k1/1p3pp1/2p5/bqB5/3P1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/3q1Q2/4PN1P/5PP1/5RK1 w - - 0 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/1q1P1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/q2P1Q2/4PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/3P1Q2/3qPN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/3P1Q2/2q1PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/3P1Q2/1q2PN1P/5PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/3P1Q2/4PN1P/4qPP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/3P1Q2/4PN1P/2q2PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/3P1Q2/4PN1P/q4PP1/5RK1 w - - 8 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/3P1Q2/4PN1P/5PP1/5qK1 w - - 0 20".to_string(),
                "rr4k1/1p3pp1/2p5/b1B5/3P1Q2/4PN1P/5PP1/2q2RK1 w - - 8 20".to_string(),
                "rr4k1/1p3p2/2p3p1/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 0 20".to_string(),
                "rr4k1/1p4p1/2p2p2/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 0 20".to_string(),
                "rr4k1/5pp1/1pp5/b1B5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 0 20".to_string(),
                "rr4k1/1p3p2/2p5/b1B3p1/2qP1Q2/4PN1P/5PP1/5RK1 w - - 0 20".to_string(),
                "rr4k1/1p4p1/2p5/b1B2p2/2qP1Q2/4PN1P/5PP1/5RK1 w - - 0 20".to_string(),
                "rr4k1/5pp1/2p5/bpB5/2qP1Q2/4PN1P/5PP1/5RK1 w - - 0 20".to_string(),
            ],
        ));
        scenarios.push((
            "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BPQ4/1K1R4 w - - 2 18".to_string(),
            vec![
                "2b2rk1/1p3ppn/2p2qN1/p1b5/P2pP2r/1PN2PRP/1BPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p1Nq2/p1b5/P2pP2r/1PN2PRP/1BPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b4N/P2pP2r/1PN2PRP/1BPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1bN4/P2pP2r/1PN2PRP/1BPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pP2r/1PNN1PRP/1BPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pP2r/1PN2PRP/1BPQ2N1/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pP2r/1PN2PRP/1BPQN3/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3pRn/2p2q2/p1b5/P2pPN1r/1PN2P1P/1BPQ4/1K1R4 b - - 0 18".to_string(),
                "2b2rk1/1p3ppn/2p2qR1/p1b5/P2pPN1r/1PN2P1P/1BPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b3R1/P2pPN1r/1PN2P1P/1BPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPNRr/1PN2P1P/1BPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2P1P/1BPQ2R1/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2P1P/1BPQ4/1K1R2R1 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1bN4/P2pPN1r/1P3PRP/1BPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/pNb5/P2pPN1r/1P3PRP/1BPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1P3PRP/1BPQN3/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1P3PRP/NBPQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2QPN1r/1PN2PRP/1BP5/1K1R4 b - - 0 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN1QPRP/1BP5/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PNQ1PRP/1BP5/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BP4Q/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BP3Q1/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BP2Q2/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BP1Q3/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BP5/1K1RQ3 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BP5/1KQR4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/BPN2PRP/2PQ4/1K1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/2PQ4/1KBR4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/2PQ4/BK1R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BPQ4/1K5R b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BPQ4/1K4R1 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BPQ4/1K3R2 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BPQ4/1K2R3 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BPQ4/1KR5 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/KBPQ4/3R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BPQ4/2KR4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/P2pPN1r/1PN2PRP/1BPQ4/K2R4 b - - 3 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b1P3/P2p1N1r/1PN2PRP/1BPQ4/1K1R4 b - - 0 18".to_string(),
                "2b2rk1/1p3ppn/2p2q2/p1b5/PP1pPN1r/2N2PRP/1BPQ4/1K1R4 b - - 0 18".to_string(),
            ],
        ));
        scenarios.push((
            "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R w - - 1 13".to_string(),
            vec![
                "r1Qq1k1r/p3ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "rQ1q1k1r/p3ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "Q2q1k1r/p3ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/p3Qpb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/p2Qppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/p1Q1ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/Q3ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/p3ppb1/2Q3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/p3ppb1/1Qn3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/p3ppb1/Q1n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/p3ppb1/2n3p1/1Q1Nn2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/p3ppb1/2n3p1/3Nn2p/1Q1pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2Npb1/2n3p1/4n2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/pQN1ppb1/2n3p1/4n2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n2Np1/4n2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/1Nn3p1/4n2p/3pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/4n2p/1N1pPB2/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/4n2p/3pPB2/1P2NP2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/4n2p/3pPB2/1PN2P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3pB/3Nn2p/3pP3/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn1Bp/3pP3/1P3P2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3NB2p/3pP3/1P3P2/P1P4P/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pP3/1P3PB1/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pP3/1P2BP2/P1P4P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pP3/1P3P2/P1PB3P/2KR1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR1BR1 b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/B1n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2KR3R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/1B1Nn2p/3pPB2/1P3P2/P1P4P/2KR3R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/2BpPB2/1P3P2/P1P4P/2KR3R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P1B/P1P4P/2KR3R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P1B1P2/P1P4P/2KR3R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P3BP/2KR3R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P1B2P/2KR3R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3RPB2/1P3P2/P1P4P/2K2B1R b - - 0 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P1R1P2/P1P4P/2K2B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1PR3P/2K2B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/2K1RB1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1PK3P/3R1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/PKP4P/3R1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P2/P1P4P/1K1R1B1R b - - 2 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/1P1pPB2/5P2/P1P4P/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1P3P1P/P1P5/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/1PP2P2/P6P/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB2/PP3P2/2P4P/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/3pPB1P/1P3P2/P1P5/2KR1B1R b - - 0 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/2PpPB2/1P3P2/P6P/2KR1B1R b - c3 0 13".to_string(),
                "r2q1k1r/pQ2ppb1/2n3p1/3Nn2p/P2pPB2/1P3P2/2P4P/2KR1B1R b - - 0 13".to_string(),
            ],
        ));
        scenarios.push((
            "2r3k1/1p4b1/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 b - - 0 25".to_string(),
            vec![
                "2r4k/1p4b1/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r5/1p4bk/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r5/1p3kb1/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "5rk1/1p4b1/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "4r1k1/1p4b1/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "3r2k1/1p4b1/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "1r4k1/1p4b1/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "r5k1/1p4b1/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "6k1/1pr3b1/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3kb/1p6/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r2bk1/1p6/p1n1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r2rk1/1p4b1/p1n1b2p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p3rb1/p1n1b2p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p4b1/p1n1b1rp/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p3bb1/p1n2r1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p1b2b1/p1n2r1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2rn2k1/1p4b1/p3br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "1nr3k1/1p4b1/p3br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p2n1b1/p3br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/np4b1/p3br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p4b1/p3br1p/q1Bpnpp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p4b1/p3br1p/q1Bp1pp1/1P1n4/5NPP/4QP1K/1BR1R3 w - - 0 26".to_string(),
                "2r3k1/1p4b1/p3br1p/q1Bp1pp1/1n1P4/5NPP/4QP1K/1BR1R3 w - - 0 26".to_string(),
                "2rq2k1/1p4b1/p1n1br1p/2Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1pq3b1/p1n1br1p/2Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p4b1/pqn1br1p/2Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p4b1/p1n1br1p/2qp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 0 26".to_string(),
                "2r3k1/1p4b1/p1n1br1p/1qBp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p4b1/p1n1br1p/2Bp1pp1/1q1P4/5NPP/4QP1K/1BR1R3 w - - 0 26".to_string(),
                "2r3k1/1p4b1/p1n1br1p/2Bp1pp1/qP1P4/5NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p4b1/p1n1br1p/2Bp1pp1/1P1P4/q4NPP/4QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p4b1/p1n1br1p/2Bp1pp1/1P1P4/5NPP/q3QP1K/1BR1R3 w - - 1 26".to_string(),
                "2r3k1/1p4b1/p1n1br1p/2Bp1pp1/1P1P4/5NPP/4QP1K/qBR1R3 w - - 1 26".to_string(),
                "2r3k1/6b1/ppn1br1p/q1Bp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 0 26".to_string(),
                "2r3k1/1p4b1/p1n1br2/q1Bp1ppp/1P1P4/5NPP/4QP1K/1BR1R3 w - - 0 26".to_string(),
                "2r3k1/1p4b1/p1n1br1p/q1Bp1p2/1P1P2p1/5NPP/4QP1K/1BR1R3 w - - 0 26".to_string(),
                "2r3k1/1p4b1/p1n1br1p/q1Bp2p1/1P1P1p2/5NPP/4QP1K/1BR1R3 w - - 0 26".to_string(),
                "2r3k1/6b1/p1n1br1p/qpBp1pp1/1P1P4/5NPP/4QP1K/1BR1R3 w - - 0 26".to_string(),
            ],
        ));
        scenarios.push((
            "2R5/8/8/1p4P1/1k2Kp2/2p4r/8/8 w - - 2 53".to_string(),
            vec![
                "7R/8/8/1p4P1/1k2Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "6R1/8/8/1p4P1/1k2Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "5R2/8/8/1p4P1/1k2Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "4R3/8/8/1p4P1/1k2Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "3R4/8/8/1p4P1/1k2Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "1R6/8/8/1p4P1/1k2Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "R7/8/8/1p4P1/1k2Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "8/2R5/8/1p4P1/1k2Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "8/8/2R5/1p4P1/1k2Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "8/8/8/1pR3P1/1k2Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "8/8/8/1p4P1/1kR1Kp2/2p4r/8/8 b - - 3 53".to_string(),
                "8/8/8/1p4P1/1k2Kp2/2R4r/8/8 b - - 0 53".to_string(),
                "2R5/8/8/1p3KP1/1k3p2/2p4r/8/8 b - - 3 53".to_string(),
                "2R5/8/8/1p2K1P1/1k3p2/2p4r/8/8 b - - 3 53".to_string(),
                "2R5/8/8/1p1K2P1/1k3p2/2p4r/8/8 b - - 3 53".to_string(),
                "2R5/8/8/1p4P1/1k3K2/2p4r/8/8 b - - 0 53".to_string(),
                "2R5/8/8/1p4P1/1k1K1p2/2p4r/8/8 b - - 3 53".to_string(),
                "2R5/8/6P1/1p6/1k2Kp2/2p4r/8/8 b - - 0 53".to_string(),
            ],
        ));
        scenarios.push((
            "8/8/2P2k2/8/1PR5/7K/2p3P1/8 w - - 0 61".to_string(),
            vec![
                "8/8/2P2k2/2R5/1P6/7K/2p3P1/8 b - - 1 61".to_string(),
                "8/8/2P2k2/8/1P5R/7K/2p3P1/8 b - - 1 61".to_string(),
                "8/8/2P2k2/8/1P4R1/7K/2p3P1/8 b - - 1 61".to_string(),
                "8/8/2P2k2/8/1P3R2/7K/2p3P1/8 b - - 1 61".to_string(),
                "8/8/2P2k2/8/1P2R3/7K/2p3P1/8 b - - 1 61".to_string(),
                "8/8/2P2k2/8/1P1R4/7K/2p3P1/8 b - - 1 61".to_string(),
                "8/8/2P2k2/8/1P6/2R4K/2p3P1/8 b - - 1 61".to_string(),
                "8/8/2P2k2/8/1P6/7K/2R3P1/8 b - - 0 61".to_string(),
                "8/8/2P2k2/8/1PR4K/8/2p3P1/8 b - - 1 61".to_string(),
                "8/8/2P2k2/8/1PR3K1/8/2p3P1/8 b - - 1 61".to_string(),
                "8/8/2P2k2/8/1PR5/6K1/2p3P1/8 b - - 1 61".to_string(),
                "8/8/2P2k2/8/1PR5/8/2p3PK/8 b - - 1 61".to_string(),
                "8/2P5/5k2/8/1PR5/7K/2p3P1/8 b - - 0 61".to_string(),
                "8/8/2P2k2/1P6/2R5/7K/2p3P1/8 b - - 0 61".to_string(),
                "8/8/2P2k2/8/1PR5/6PK/2p5/8 b - - 0 61".to_string(),
                "8/8/2P2k2/8/1PR3P1/7K/2p5/8 b - - 0 61".to_string(),
            ],
        ));
        scenarios.push((
            "8/8/8/1p6/3q4/K4k2/8/7q b - - 3 65".to_string(),
            vec![
                "7q/8/8/1p6/8/K4k2/8/7q w - - 4 66".to_string(),
                "3q4/8/8/1p6/8/K4k2/8/7q w - - 4 66".to_string(),
                "8/6q1/8/1p6/8/K4k2/8/7q w - - 4 66".to_string(),
                "8/3q4/8/1p6/8/K4k2/8/7q w - - 4 66".to_string(),
                "8/q7/8/1p6/8/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/5q2/1p6/8/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/3q4/1p6/8/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/1q6/1p6/8/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p2q3/8/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p1q4/8/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1pq5/8/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/7q/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/6q1/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/5q2/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/4q3/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/2q5/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/1q6/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/q7/K4k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/8/K3qk2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/8/K2q1k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/8/K1q2k2/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/8/K4k2/5q2/7q w - - 4 66".to_string(),
                "8/8/8/1p6/8/K4k2/3q4/7q w - - 4 66".to_string(),
                "8/8/8/1p6/8/K4k2/1q6/7q w - - 4 66".to_string(),
                "8/8/8/1p6/8/K4k2/8/6qq w - - 4 66".to_string(),
                "8/8/8/1p6/8/K4k2/8/3q3q w - - 4 66".to_string(),
                "8/8/8/1p6/8/K4k2/8/q6q w - - 4 66".to_string(),
                "8/8/8/1p6/3q2k1/K7/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/3q1k2/K7/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/3qk3/K7/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K5k1/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K3k3/8/7q w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K7/6k1/7q w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K7/5k2/7q w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K7/4k3/7q w - - 4 66".to_string(),
                "7q/8/8/1p6/3q4/K4k2/8/8 w - - 4 66".to_string(),
                "8/7q/8/1p6/3q4/K4k2/8/8 w - - 4 66".to_string(),
                "8/8/7q/1p6/3q4/K4k2/8/8 w - - 4 66".to_string(),
                "8/8/8/1p5q/3q4/K4k2/8/8 w - - 4 66".to_string(),
                "8/8/8/1p6/3q3q/K4k2/8/8 w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K4k1q/8/8 w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K4k2/7q/8 w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K4k2/6q1/8 w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K4k2/8/6q1 w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K4k2/8/5q2 w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K4k2/8/4q3 w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K4k2/8/3q4 w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K4k2/8/2q5 w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K4k2/8/1q6 w - - 4 66".to_string(),
                "8/8/8/1p6/3q4/K4k2/8/q7 w - - 4 66".to_string(),
                "8/8/8/8/1p1q4/K4k2/8/7q w - - 0 66".to_string(),
            ],
        ));
        scenarios.push((
            "rn1q1rk1/pp2n1bp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 b - - 2 8".to_string(),
            vec![
                "rn1q1r1k/pp2n1bp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1r2/pp2nkbp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1qr1k1/pp2n1bp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q2k1/pp2nrbp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q2k1/pp2n1bp/2p2rp1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q2k1/pp2n1bp/2p3p1/3ppr2/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q2k1/pp2n1bp/2p3p1/3pp3/1P2Prb1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q2k1/pp2n1bp/2p3p1/3pp3/1P2P1b1/P4rP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn2qrk1/pp2n1bp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rnq2rk1/pp2n1bp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn3rk1/pp1qn1bp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn3rk1/ppq1n1bp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn3rk1/pp2n1bp/2pq2p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn3rk1/pp2n1bp/1qp3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn3rk1/pp2n1bp/2p3p1/q2pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "r2q1rk1/pp1nn1bp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "r2q1rk1/pp2n1bp/n1p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1rkb/pp2n2p/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1rk1/pp2n2p/2p3pb/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1rk1/pp2n2p/2p2bp1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rnnq1rk1/pp4bp/2p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1rk1/pp4bp/2p3p1/3ppn2/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rnbq1rk1/pp2n1bp/2p3p1/3pp3/1P2P3/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1rk1/pp1bn1bp/2p3p1/3pp3/1P2P3/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1rk1/pp2n1bp/2p1b1p1/3pp3/1P2P3/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1rk1/pp2n1bp/2p3p1/3pp2b/1P2P3/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1rk1/pp2n1bp/2p3p1/3ppb2/1P2P3/P4NP1/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1rk1/pp2n1bp/2p3p1/3pp3/1P2P3/P4NPb/2PN1PBP/R1B1QRK1 w - - 3 9".to_string(),
                "rn1q1rk1/pp2n1bp/2p3p1/3pp3/1P2P3/P4bP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn1q1rk1/pp2n1bp/2p3p1/4p3/1P2p1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn1q1rk1/pp2n1b1/2p3pp/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn1q1rk1/p3n1bp/1pp3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn1q1rk1/1p2n1bp/p1p3p1/3pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn1q1rk1/pp2n1bp/2p5/3pp1p1/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn1q1rk1/pp2n1bp/6p1/2ppp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn1q1rk1/pp2n1bp/2p3p1/4p3/1P1pP1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn1q1rk1/pp2n1b1/2p3p1/3pp2p/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn1q1rk1/p3n1bp/2p3p1/1p1pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
                "rn1q1rk1/1p2n1bp/2p3p1/p2pp3/1P2P1b1/P4NP1/2PN1PBP/R1B1QRK1 w - - 0 9".to_string(),
            ],
        ));
        scenarios.push((
            "3r1Rk1/p2p3p/1p4p1/6n1/2P1P3/1PN5/P2R2p1/6K1 b - - 0 27".to_string(),
            vec![
                "3r1k2/p2p3p/1p4p1/6n1/2P1P3/1PN5/P2R2p1/6K1 w - - 0 28".to_string(),
                "3r1R2/p2p2kp/1p4p1/6n1/2P1P3/1PN5/P2R2p1/6K1 w - - 1 28".to_string(),
                "5rk1/p2p3p/1p4p1/6n1/2P1P3/1PN5/P2R2p1/6K1 w - - 0 28".to_string(),
            ],
        ));
        scenarios.push((
            "8/8/7p/r5k1/2R5/r7/6K1/8 w - - 0 61".to_string(),
            vec![
                "2R5/8/7p/r5k1/8/r7/6K1/8 b - - 1 61".to_string(),
                "8/2R5/7p/r5k1/8/r7/6K1/8 b - - 1 61".to_string(),
                "8/8/2R4p/r5k1/8/r7/6K1/8 b - - 1 61".to_string(),
                "8/8/7p/r1R3k1/8/r7/6K1/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/7R/r7/6K1/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/6R1/r7/6K1/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/5R2/r7/6K1/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/4R3/r7/6K1/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/3R4/r7/6K1/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/1R6/r7/6K1/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/R7/r7/6K1/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/8/r1R5/6K1/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/8/r7/2R3K1/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/8/r7/6K1/2R5 b - - 1 61".to_string(),
                "8/8/7p/r5k1/2R5/r7/7K/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/2R5/r7/5K2/8 b - - 1 61".to_string(),
                "8/8/7p/r5k1/2R5/r7/8/7K b - - 1 61".to_string(),
                "8/8/7p/r5k1/2R5/r7/8/6K1 b - - 1 61".to_string(),
                "8/8/7p/r5k1/2R5/r7/8/5K2 b - - 1 61".to_string(),
            ],
        ));
        scenarios.push((
            "r5k1/3n1pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K b - - 6 29".to_string(),
            vec![
                "r6k/3n1pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r4k2/3n1pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r7/3n1ppk/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "5rk1/3n1pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "4r1k1/3n1pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "3r2k1/3n1pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "2r3k1/3n1pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "1r4k1/3n1pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "6k1/r2n1pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "6k1/3n1pp1/rpb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r4nk1/5pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "rn4k1/5pp1/1pb3rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/5pp1/1pb2nrp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/5pp1/1pb3rp/p1n1pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb2r1p/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb1r2p/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pbr3p/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb4p/p3pqr1/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb4p/p3pq2/1PPp2r1/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb4p/p3pq2/1PPp4/P2P1PrP/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb4p/p3pq2/1PPp4/P2P1P1P/1Q1BB1r1/2R1R2K w - - 0 30".to_string(),
                "r5k1/1b1n1pp1/1p4rp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1p4rp/p2bpq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1p4rp/pb2pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1p4rp/p3pq2/1PPpb3/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1p4rp/p3pq2/bPPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1p4rp/p3pq2/1PPp4/P2P1b1P/1Q1BB1P1/2R1R2K w - - 0 30".to_string(),
                "r5k1/3n1pp1/1pb2qrp/p3p3/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb1q1rp/p3p3/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/p3p2q/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/p3p1q1/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/p3p3/1PPp2q1/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/p3p3/1PPp1q2/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/p3p3/1PPpq3/P2P1P1P/1Q1BB1P1/2R1R2K w - - 7 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/p3p3/1PPp4/P2P1P1q/1Q1BB1P1/2R1R2K w - - 0 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/p3p3/1PPp4/P2P1q1P/1Q1BB1P1/2R1R2K w - - 0 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/p3p3/1PPp4/P2q1P1P/1Q1BB1P1/2R1R2K w - - 0 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/4pq2/1pPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 0 30".to_string(),
                "r5k1/3n2p1/1pb2prp/p3pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 0 30".to_string(),
                "r5k1/3n1pp1/1pb3r1/p3pq1p/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 0 30".to_string(),
                "r5k1/3n1pp1/2b3rp/pp2pq2/1PPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 0 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/p4q2/1PPpp3/P2P1P1P/1Q1BB1P1/2R1R2K w - - 0 30".to_string(),
                "r5k1/3n1pp1/1pb3rp/4pq2/pPPp4/P2P1P1P/1Q1BB1P1/2R1R2K w - - 0 30".to_string(),
            ],
        ));
        scenarios.push((
            "4Q3/3r1bk1/6p1/3B3p/1p1K3P/6P1/3N4/8 b - - 0 64".to_string(),
            vec![
                "4Q3/3r1b1k/6p1/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4Q3/3r1b2/6pk/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4Q3/3r1b2/5kp1/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4Q1b1/3r2k1/6p1/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4b3/3r2k1/6p1/3B3p/1p1K3P/6P1/3N4/8 w - - 0 65".to_string(),
                "4Q3/3r2k1/4b1p1/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4Q3/3r2k1/6p1/3b3p/1p1K3P/6P1/3N4/8 w - - 0 65".to_string(),
                "3rQ3/5bk1/6p1/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4Q3/4rbk1/6p1/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4Q3/2r2bk1/6p1/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4Q3/1r3bk1/6p1/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4Q3/r4bk1/6p1/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4Q3/5bk1/3r2p1/3B3p/1p1K3P/6P1/3N4/8 w - - 1 65".to_string(),
                "4Q3/5bk1/6p1/3r3p/1p1K3P/6P1/3N4/8 w - - 0 65".to_string(),
                "4Q3/3r1bk1/8/3B2pp/1p1K3P/6P1/3N4/8 w - - 0 65".to_string(),
                "4Q3/3r1bk1/6p1/3B3p/3K3P/1p4P1/3N4/8 w - - 0 65".to_string(),
            ],
        ));
        scenarios.push((
            "r4rk1/1p1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 b - - 0 8".to_string(),
            vec![
                "r4r1k/1p1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r3r1k1/1p1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "r2r2k1/1p1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "r1r3k1/1p1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "rr4k1/1p1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "4rrk1/1p1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "3r1rk1/1p1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "2r2rk1/1p1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "1r3rk1/1p1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "5rk1/rp1bbppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "5rk1/1p1bbppp/rqnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r2b1rk1/1p1b1ppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "r3brk1/1p2bppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r1b2rk1/1p2bppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "r4rk1/1p2bppp/1qnpbn2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p2bppp/1qnp1n2/p1p1pb2/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p2bppp/1qnp1n2/p1p1p3/P1P1P1b1/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "r4rk1/1p2bppp/1qnp1n2/p1p1p3/P1P1P3/1PNPBN1b/4BPP1/R2Q1RK1 w - - 0 9".to_string(),
                "r3nrk1/1p1bbppp/1qnp4/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p1bbppp/1qnp4/p1p1p2n/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p1bbppp/1qnp4/p1pnp3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p1bbppp/1qnp4/p1p1p3/P1P1P1n1/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p1bbppp/1qnp4/p1p1p3/P1P1n3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 0 9".to_string(),
                "r2n1rk1/1p1bbppp/1q1p1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "rn3rk1/1p1bbppp/1q1p1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "r4rk1/np1bbppp/1q1p1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p1bbppp/1q1p1n2/p1p1p3/P1PnP3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p1bbppp/1q1p1n2/p1p1p3/PnP1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r2q1rk1/1p1bbppp/2np1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9"
                    .to_string(),
                "r4rk1/1pqbbppp/2np1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/qp1bbppp/2np1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p1bbppp/q1np1n2/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p1bbppp/2np1n2/pqp1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p1bbppp/2np1n2/p1p1p3/PqP1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 1 9".to_string(),
                "r4rk1/1p1bbppp/2np1n2/p1p1p3/P1P1P3/1qNPBN1P/4BPP1/R2Q1RK1 w - - 0 9".to_string(),
                "r4rk1/1p1bbpp1/1qnp1n1p/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 0 9"
                    .to_string(),
                "r4rk1/1p1bbp1p/1qnp1np1/p1p1p3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 0 9"
                    .to_string(),
                "r4rk1/1p1bbppp/1qn2n2/p1ppp3/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 0 9".to_string(),
                "r4rk1/1p1bbpp1/1qnp1n2/p1p1p2p/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 0 9"
                    .to_string(),
                "r4rk1/1p1bbp1p/1qnp1n2/p1p1p1p1/P1P1P3/1PNPBN1P/4BPP1/R2Q1RK1 w - - 0 9"
                    .to_string(),
            ],
        ));
        scenarios.push((
            "2R5/8/3PP3/1k2K3/8/8/r7/8 w - - 3 76".to_string(),
            vec![
                "7R/8/3PP3/1k2K3/8/8/r7/8 b - - 4 76".to_string(),
                "6R1/8/3PP3/1k2K3/8/8/r7/8 b - - 4 76".to_string(),
                "5R2/8/3PP3/1k2K3/8/8/r7/8 b - - 4 76".to_string(),
                "4R3/8/3PP3/1k2K3/8/8/r7/8 b - - 4 76".to_string(),
                "3R4/8/3PP3/1k2K3/8/8/r7/8 b - - 4 76".to_string(),
                "1R6/8/3PP3/1k2K3/8/8/r7/8 b - - 4 76".to_string(),
                "R7/8/3PP3/1k2K3/8/8/r7/8 b - - 4 76".to_string(),
                "8/2R5/3PP3/1k2K3/8/8/r7/8 b - - 4 76".to_string(),
                "8/8/2RPP3/1k2K3/8/8/r7/8 b - - 4 76".to_string(),
                "8/8/3PP3/1kR1K3/8/8/r7/8 b - - 4 76".to_string(),
                "8/8/3PP3/1k2K3/2R5/8/r7/8 b - - 4 76".to_string(),
                "8/8/3PP3/1k2K3/8/2R5/r7/8 b - - 4 76".to_string(),
                "8/8/3PP3/1k2K3/8/8/r1R5/8 b - - 4 76".to_string(),
                "8/8/3PP3/1k2K3/8/8/r7/2R5 b - - 4 76".to_string(),
                "2R5/8/3PPK2/1k6/8/8/r7/8 b - - 4 76".to_string(),
                "2R5/8/3PP3/1k3K2/8/8/r7/8 b - - 4 76".to_string(),
                "2R5/8/3PP3/1k1K4/8/8/r7/8 b - - 4 76".to_string(),
                "2R5/8/3PP3/1k6/5K2/8/r7/8 b - - 4 76".to_string(),
                "2R5/8/3PP3/1k6/4K3/8/r7/8 b - - 4 76".to_string(),
                "2R5/8/3PP3/1k6/3K4/8/r7/8 b - - 4 76".to_string(),
                "2R5/4P3/3P4/1k2K3/8/8/r7/8 b - - 0 76".to_string(),
                "2R5/3P4/4P3/1k2K3/8/8/r7/8 b - - 0 76".to_string(),
            ],
        ));
        scenarios.push((
            "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3PKP/2R2R2 w k - 3 12".to_string(),
            vec![
                "1r1qk2r/ppp1bpp1/3p1n1p/3N4/2P5/NQ2P1P1/PP3PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/1N6/2P5/NQ2P1P1/PP3PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P1N3/NQ2P1P1/PP3PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/N1P5/NQ2P1P1/PP3PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQ2P1P1/PP2NPKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQ2P1P1/PP3PKP/2RN1R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQ2P1P1/PP3PKP/1NR2R2 b k - 4 12".to_string(),
                "1r1qk2r/pQp1bpp1/3p1n1p/8/2P5/N1N1P1P1/PP3PKP/2R2R2 b k - 0 12".to_string(),
                "1r1qk2r/ppp1bpp1/1Q1p1n1p/8/2P5/N1N1P1P1/PP3PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/1Q6/2P5/N1N1P1P1/PP3PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/1QP5/N1N1P1P1/PP3PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/Q1P5/N1N1P1P1/PP3PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/N1N1P1P1/PPQ2PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/N1N1P1P1/PP3PKP/2RQ1R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/1N6/2P5/1QN1P1P1/PP3PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/1QN1P1P1/PPN2PKP/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/1QN1P1P1/PP3PKP/1NR2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1PK/PP3P1P/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1PKP1/PP3P1P/2R2R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3P1P/2R2R1K b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3P1P/2R2RK1 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3PKP/2R4R b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3PKP/2R3R1 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3PKP/2R1R3 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3PKP/2RR4 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PPR2PKP/5R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3PKP/4RR2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3PKP/3R1R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3PKP/1R3R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1P1/PP3PKP/R4R2 b k - 4 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/2P5/8/NQN1P1P1/PP3PKP/2R2R2 b k - 0 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P3P1/NQN1P3/PP3PKP/2R2R2 b k - 0 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P1P3/NQN3P1/PP3PKP/2R2R2 b k - 0 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1P1PP/PP3PK1/2R2R2 b k - 0 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P5/NQN1PPP1/PP4KP/2R2R2 b k - 0 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P4P/NQN1P1P1/PP3PK1/2R2R2 b k - 0 12".to_string(),
                "1r1qk2r/ppp1bpp1/3p1n1p/8/2P2P2/NQN1P1P1/PP4KP/2R2R2 b k - 0 12".to_string(),
            ],
        ));
        scenarios.push((
            "5k2/r7/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 b - - 5 46".to_string(),
            vec![
                "6k1/r7/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "4k3/r7/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "8/r5k1/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "8/r4k2/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "8/r3k3/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "r4k2/8/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/7r/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/6r1/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/5r2/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/4r3/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/3r4/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/2r5/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/1r6/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/8/r1n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/8/2n1p1q1/rp1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/8/2n1p1q1/1p1pPp1p/rPnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/8/2n1p1q1/1p1pPp1p/1PnP1P1b/r1N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/8/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/r5P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/8/2n1p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/r2QB1K1 w - - 6 47".to_string(),
                "5kq1/r7/2n1p3/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "4qk2/r7/2n1p3/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r6q/2n1p3/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r5q1/2n1p3/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r4q2/2n1p3/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p2q/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1pq2/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p3/1p1pPpqp/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p3/1p1pPp1p/1PnP1Pqb/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p3/1p1pPp1p/1PnP1P1b/2N2Rq1/6P1/3QB1K1 w - - 0 47".to_string(),
                "3n1k2/r7/4p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "1n3k2/r7/4p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r3n3/4p1q1/1p1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/4p1q1/1p1pnp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 0 47".to_string(),
                "5k2/r7/4p1q1/np1pPp1p/1PnP1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/4p1q1/1p1pPp1p/1Pnn1P1b/2N2RN1/6P1/3QB1K1 w - - 0 47".to_string(),
                "5k2/r7/4p1q1/1p1pPp1p/1nnP1P1b/2N2RN1/6P1/3QB1K1 w - - 0 47".to_string(),
                "3b1k2/r7/2n1p1q1/1p1pPp1p/1PnP1P2/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r3b3/2n1p1q1/1p1pPp1p/1PnP1P2/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1pbq1/1p1pPp1p/1PnP1P2/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p1q1/1p1pPpbp/1PnP1P2/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p1q1/1p1pPp1p/1PnP1P2/2N2Rb1/6P1/3QB1K1 w - - 0 47".to_string(),
                "5k2/r7/2nnp1q1/1p1pPp1p/1P1P1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/1nn1p1q1/1p1pPp1p/1P1P1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p1q1/1p1pnp1p/1P1P1P1b/2N2RN1/6P1/3QB1K1 w - - 0 47".to_string(),
                "5k2/r7/2n1p1q1/np1pPp1p/1P1P1P1b/2N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p1q1/1p1pPp1p/1P1P1P1b/2N1nRN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p1q1/1p1pPp1p/1P1P1P1b/n1N2RN1/6P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p1q1/1p1pPp1p/1P1P1P1b/2N2RN1/3n2P1/3QB1K1 w - - 6 47".to_string(),
                "5k2/r7/2n1p1q1/1p1pPp1p/1P1P1P1b/2N2RN1/1n4P1/3QB1K1 w - - 6 47".to_string(),
            ],
        ));
        scenarios.push((
            "3r4/8/1K5R/8/2B2nkP/8/8/8 b - - 10 43".to_string(),
            vec![
                "7r/8/1K5R/8/2B2nkP/8/8/8 w - - 11 44".to_string(),
                "6r1/8/1K5R/8/2B2nkP/8/8/8 w - - 11 44".to_string(),
                "5r2/8/1K5R/8/2B2nkP/8/8/8 w - - 11 44".to_string(),
                "4r3/8/1K5R/8/2B2nkP/8/8/8 w - - 11 44".to_string(),
                "2r5/8/1K5R/8/2B2nkP/8/8/8 w - - 11 44".to_string(),
                "1r6/8/1K5R/8/2B2nkP/8/8/8 w - - 11 44".to_string(),
                "r7/8/1K5R/8/2B2nkP/8/8/8 w - - 11 44".to_string(),
                "8/3r4/1K5R/8/2B2nkP/8/8/8 w - - 11 44".to_string(),
                "8/8/1K1r3R/8/2B2nkP/8/8/8 w - - 11 44".to_string(),
                "8/8/1K5R/3r4/2B2nkP/8/8/8 w - - 11 44".to_string(),
                "8/8/1K5R/8/2Br1nkP/8/8/8 w - - 11 44".to_string(),
                "8/8/1K5R/8/2B2nkP/3r4/8/8 w - - 11 44".to_string(),
                "8/8/1K5R/8/2B2nkP/8/3r4/8 w - - 11 44".to_string(),
                "8/8/1K5R/8/2B2nkP/8/8/3r4 w - - 11 44".to_string(),
                "3r4/8/1K5R/5k2/2B2n1P/8/8/8 w - - 11 44".to_string(),
                "3r4/8/1K5R/8/2B2n1P/7k/8/8 w - - 11 44".to_string(),
                "3r4/8/1K5R/8/2B2n1P/6k1/8/8 w - - 11 44".to_string(),
                "3r4/8/1K5R/8/2B2n1P/5k2/8/8 w - - 11 44".to_string(),
                "3r4/8/1K4nR/8/2B3kP/8/8/8 w - - 11 44".to_string(),
                "3r4/8/1K2n2R/8/2B3kP/8/8/8 w - - 11 44".to_string(),
                "3r4/8/1K5R/7n/2B3kP/8/8/8 w - - 11 44".to_string(),
                "3r4/8/1K5R/3n4/2B3kP/8/8/8 w - - 11 44".to_string(),
                "3r4/8/1K5R/8/2B3kP/7n/8/8 w - - 11 44".to_string(),
                "3r4/8/1K5R/8/2B3kP/3n4/8/8 w - - 11 44".to_string(),
                "3r4/8/1K5R/8/2B3kP/8/6n1/8 w - - 11 44".to_string(),
                "3r4/8/1K5R/8/2B3kP/8/4n3/8 w - - 11 44".to_string(),
            ],
        ));
        scenarios.push((
            "rnb1r3/p5pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 b - - 0 12".to_string(),
            vec![
                "rnb4r/p5pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb3r1/p5pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb2r2/p5pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnbr4/p5pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb5/p3r1pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb5/p5pk/1ppqrn1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb5/p5pk/1ppq1n1p/3prp2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb5/p5pk/1ppq1n1p/3p1p2/3PrN1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb5/p5pk/1ppq1n1p/3p1p2/3P1N1P/2P1r1P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb5/p5pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP2rPB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb5/p5pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1QrRK1 w - - 1 13".to_string(),
                "rn2r3/p2b2pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rn2r3/pb4pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rn2r3/p5pk/1ppqbn1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rn2r3/p5pk/bppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "r1b1r3/p2n2pk/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "r1b1r3/p5pk/nppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r2k/p5p1/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r1k1/p5p1/1ppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r1n1/p5pk/1ppq3p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p2n2pk/1ppq3p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p5pk/1ppq3p/3p1p1n/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p5pk/1ppq3p/3p1p2/3P1NnP/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p5pk/1ppq3p/3p1p2/3PnN1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1rq2/p5pk/1pp2n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnbqr3/p5pk/1pp2n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p3q1pk/1pp2n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p2q2pk/1pp2n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p1q3pk/1pp2n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p5pk/1pp1qn1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p5pk/1pp2n1p/3pqp2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p5pk/1pp2n1p/2qp1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p5pk/1pp2n1p/3p1p2/3P1q1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 0 13".to_string(),
                "rnb1r3/p5pk/1pp2n1p/3p1p2/1q1P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p5pk/1pp2n1p/3p1p2/3P1N1P/q1P3P1/PP3PB1/RN1Q1RK1 w - - 1 13".to_string(),
                "rnb1r3/p6k/1ppq1npp/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 0 13".to_string(),
                "rnb1r3/6pk/pppq1n1p/3p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 0 13".to_string(),
                "rnb1r3/p5pk/1ppq1n2/3p1p1p/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 0 13".to_string(),
                "rnb1r3/p5pk/1p1q1n1p/2pp1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 0 13".to_string(),
                "rnb1r3/p5pk/2pq1n1p/1p1p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 0 13".to_string(),
                "rnb1r3/p6k/1ppq1n1p/3p1pp1/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 0 13".to_string(),
                "rnb1r3/6pk/1ppq1n1p/p2p1p2/3P1N1P/2P3P1/PP3PB1/RN1Q1RK1 w - - 0 13".to_string(),
            ],
        ));
        scenarios.push((
            "8/B7/4p3/1p1pP3/1PnPk3/p1K5/8/8 w - - 0 86".to_string(),
            vec![
                "1B6/8/4p3/1p1pP3/1PnPk3/p1K5/8/8 b - - 1 86".to_string(),
                "8/8/1B2p3/1p1pP3/1PnPk3/p1K5/8/8 b - - 1 86".to_string(),
                "8/8/4p3/1pBpP3/1PnPk3/p1K5/8/8 b - - 1 86".to_string(),
                "8/B7/4p3/1p1pP3/1PnPk3/pK6/8/8 b - - 1 86".to_string(),
                "8/B7/4p3/1p1pP3/1PnPk3/p7/2K5/8 b - - 1 86".to_string(),
            ],
        ));
        scenarios.push((
            "8/5p2/2R4p/1Pn1kP2/2b3P1/pr4N1/7P/5BK1 b - - 1 44".to_string(),
            vec![
                "8/5p2/2R4p/1Pnk1P2/2b3P1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1Pn2P2/2b2kP1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1Pn2P2/2bk2P1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/3n1p2/2R4p/1P2kP2/2b3P1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/1n3p2/2R4p/1P2kP2/2b3P1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R1n2p/1P2kP2/2b3P1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/n1R4p/1P2kP2/2b3P1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1P2kP2/2b1n1P1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1P2kP2/n1b3P1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1P2kP2/2b3P1/pr1n2N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R1b2p/1Pn1kP2/6P1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1PnbkP2/6P1/pr4N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1bn1kP2/6P1/pr4N1/7P/5BK1 w - - 0 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/6P1/pr1b2N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/6P1/pr4N1/4b2P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/6P1/pr4N1/7P/5bK1 w - - 0 45".to_string(),
                "8/5p2/2R4p/1rn1kP2/2b3P1/p5N1/7P/5BK1 w - - 0 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/1rb3P1/p5N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/2b3P1/p5r1/7P/5BK1 w - - 0 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/2b3P1/p4rN1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/2b3P1/p3r1N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/2b3P1/p2r2N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/2b3P1/p1r3N1/7P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/2b3P1/p5N1/1r5P/5BK1 w - - 2 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/2b3P1/p5N1/7P/1r3BK1 w - - 2 45".to_string(),
                "8/8/2R2p1p/1Pn1kP2/2b3P1/pr4N1/7P/5BK1 w - - 0 45".to_string(),
                "8/5p2/2R5/1Pn1kP1p/2b3P1/pr4N1/7P/5BK1 w - - 0 45".to_string(),
                "8/5p2/2R4p/1Pn1kP2/2b3P1/1r4N1/p6P/5BK1 w - - 0 45".to_string(),
            ],
        ));
        scenarios.push((
            "8/8/4N3/3Pk3/1bP5/5K1P/8/8 w - - 7 71".to_string(),
            vec![
                "5N2/8/8/3Pk3/1bP5/5K1P/8/8 b - - 8 71".to_string(),
                "3N4/8/8/3Pk3/1bP5/5K1P/8/8 b - - 8 71".to_string(),
                "8/6N1/8/3Pk3/1bP5/5K1P/8/8 b - - 8 71".to_string(),
                "8/2N5/8/3Pk3/1bP5/5K1P/8/8 b - - 8 71".to_string(),
                "8/8/8/3Pk1N1/1bP5/5K1P/8/8 b - - 8 71".to_string(),
                "8/8/8/2NPk3/1bP5/5K1P/8/8 b - - 8 71".to_string(),
                "8/8/8/3Pk3/1bP2N2/5K1P/8/8 b - - 8 71".to_string(),
                "8/8/8/3Pk3/1bPN4/5K1P/8/8 b - - 8 71".to_string(),
                "8/8/4N3/3Pk3/1bP3K1/7P/8/8 b - - 8 71".to_string(),
                "8/8/4N3/3Pk3/1bP5/6KP/8/8 b - - 8 71".to_string(),
                "8/8/4N3/3Pk3/1bP5/4K2P/8/8 b - - 8 71".to_string(),
                "8/8/4N3/3Pk3/1bP5/7P/6K1/8 b - - 8 71".to_string(),
                "8/8/4N3/3Pk3/1bP5/7P/5K2/8 b - - 8 71".to_string(),
                "8/8/4N3/3Pk3/1bP5/7P/4K3/8 b - - 8 71".to_string(),
                "8/8/3PN3/4k3/1bP5/5K1P/8/8 b - - 0 71".to_string(),
                "8/8/4N3/2PPk3/1b6/5K1P/8/8 b - - 0 71".to_string(),
                "8/8/4N3/3Pk3/1bP4P/5K2/8/8 b - - 0 71".to_string(),
            ],
        ));
        scenarios.push((
            "r7/p2k4/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 b - - 0 65".to_string(),
            vec![
                "7r/p2k4/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "6r1/p2k4/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "5r2/p2k4/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "4r3/p2k4/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "3r4/p2k4/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "2r5/p2k4/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "1r6/p2k4/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r3k3/p7/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r2k4/p7/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r1k5/p7/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r7/p3k3/1np1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r4n2/p2k4/1np3p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r2n4/p2k4/1np3p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r7/p2k2n1/1np3p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r7/p1nk4/1np3p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r7/p2k4/1np3p1/3pBpnp/pRPP3P/5P2/P3BK2/8 w - - 0 66".to_string(),
                "r7/p2k4/1np3p1/2npBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r7/p2k4/1np3p1/3pBpPp/pRPP1n1P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r7/p2k4/1np3p1/3pBpPp/pRPn3P/5P2/P3BK2/8 w - - 0 66".to_string(),
                "r1n5/p2k4/2p1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 1 66".to_string(),
                "r7/p2k4/2p1n1p1/3pBpPp/pRnP3P/5P2/P3BK2/8 w - - 0 66".to_string(),
                "r7/p2k4/1np1n1p1/4BpPp/pRpP3P/5P2/P3BK2/8 w - - 0 66".to_string(),
                "r7/3k4/pnp1n1p1/3pBpPp/pRPP3P/5P2/P3BK2/8 w - - 0 66".to_string(),
                "r7/p2k4/1n2n1p1/2ppBpPp/pRPP3P/5P2/P3BK2/8 w - - 0 66".to_string(),
                "r7/p2k4/1np1n1p1/3pB1Pp/pRPP1p1P/5P2/P3BK2/8 w - - 0 66".to_string(),
                "r7/p2k4/1np1n1p1/3pBpPp/1RPP3P/p4P2/P3BK2/8 w - - 0 66".to_string(),
                "r7/3k4/1np1n1p1/p2pBpPp/pRPP3P/5P2/P3BK2/8 w - - 0 66".to_string(),
            ],
        ));
        scenarios.push((
            "7r/2qn2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 b - - 1 30".to_string(),
            vec![
                "6r1/2qn2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "5r2/2qn2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "4r3/2qn2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "3r4/2qn2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "2r5/2qn2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "1r6/2qn2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "r7/2qn2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "8/2qn2kr/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "8/2qn2k1/2b1n2r/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "6kr/2qn4/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "5k1r/2qn4/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn3k/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn1k2/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn4/2b1n1k1/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn4/2b1nk2/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "5n1r/2q3k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "1n5r/2q3k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2q3k1/2b1nn2/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2q3k1/1nb1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "3q3r/3n2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "2q4r/3n2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "1q5r/3n2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/1q1n2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/q2n2k1/2b1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/3n2k1/2bqn3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/3n2k1/1qb1n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/3n2k1/2b1n3/q1p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "5n1r/2qn2k1/2b5/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "3n3r/2qn2k1/2b5/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/2b5/2p1ppnp/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/2b5/2p1pp1p/2P2n2/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/2b5/2p1pp1p/2Pn4/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "b6r/2qn2k1/4n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/1bqn2k1/4n3/2p1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/4n3/2pbpp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/4n3/1bp1pp1p/2P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/4n3/2p1pp1p/2P1b3/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/4n3/2p1pp1p/b1P5/r1B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/4n3/2p1pp1p/2P5/r1B2b1P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/4n3/2p1pp1p/2P5/r1B4P/1R3PbK/2QBR3 w - - 0 31".to_string(),
                "r6r/2qn2k1/2b1n3/2p1pp1p/2P5/2B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/r1qn2k1/2b1n3/2p1pp1p/2P5/2B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/r1b1n3/2p1pp1p/2P5/2B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/2b1n3/r1p1pp1p/2P5/2B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/2b1n3/2p1pp1p/r1P5/2B4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/2b1n3/2p1pp1p/2P5/2r4P/1R3PPK/2QBR3 w - - 0 31".to_string(),
                "7r/2qn2k1/2b1n3/2p1pp1p/2P5/1rB4P/1R3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/2b1n3/2p1pp1p/2P5/2B4P/rR3PPK/2QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/2b1n3/2p1pp1p/2P5/2B4P/1R3PPK/r1QBR3 w - - 2 31".to_string(),
                "7r/2qn2k1/2b1n3/2p1pp2/2P4p/r1B4P/1R3PPK/2QBR3 w - - 0 31".to_string(),
                "7r/2qn2k1/2b1n3/2p1p2p/2P2p2/r1B4P/1R3PPK/2QBR3 w - - 0 31".to_string(),
            ],
        ));
        scenarios.push((
            "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP2QRBP/3R2K1 w - - 0 16".to_string(),
            vec![
                "3r1rk1/1pq1bppp/p1b1pN2/4P3/3P4/4n1P1/PP2QRBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1bNp3/4P3/3P4/4n1P1/PP2QRBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P1N1/3P4/4n1P1/PP2QRBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/2N1P3/3P4/4n1P1/PP2QRBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3P4/2N1n1P1/PP2QRBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3P4/4n1P1/PP1NQRBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1PB/PP2QR1P/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4nBP1/PP2QR1P/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP2QR1P/3R2KB b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP2QR1P/3R1BK1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bRpp/p1b1p3/4P3/3PN3/4n1P1/PP2Q1BP/3R2K1 b - - 0 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1pR2/4P3/3PN3/4n1P1/PP2Q1BP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4PR2/3PN3/4n1P1/PP2Q1BP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PNR2/4n1P1/PP2Q1BP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4nRP1/PP2Q1BP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP2Q1BP/3R1RK1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/Q1b1p3/4P3/3PN3/4n1P1/PP3RBP/3R2K1 b - - 0 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P2Q/3PN3/4n1P1/PP3RBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/1Q2P3/3PN3/4n1P1/PP3RBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN1Q1/4n1P1/PP3RBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/2QPN3/4n1P1/PP3RBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4nQP1/PP3RBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4Q1P1/PP3RBP/3R2K1 b - - 0 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/3Qn1P1/PP3RBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP1Q1RBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PPQ2RBP/3R2K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP3RBP/3R1QK1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP3RBP/3RQ1K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP2QRBP/3R3K b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/3Rn1P1/PP2QRBP/6K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP1RQRBP/6K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP2QRBP/5RK1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP2QRBP/4R1K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP2QRBP/2R3K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP2QRBP/1R4K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1P1/PP2QRBP/R5K1 b - - 1 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/3PP3/4N3/4n1P1/PP2QRBP/3R2K1 b - - 0 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN1P1/4n3/PP2QRBP/3R2K1 b - - 0 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/4n1PP/PP2QRB1/3R2K1 b - - 0 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/1P2n1P1/P3QRBP/3R2K1 b - - 0 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN3/P3n1P1/1P2QRBP/3R2K1 b - - 0 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/3PN2P/4n1P1/PP2QRB1/3R2K1 b - - 0 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/1P1PN3/4n1P1/P3QRBP/3R2K1 b - - 0 16".to_string(),
                "3r1rk1/1pq1bppp/p1b1p3/4P3/P2PN3/4n1P1/1P2QRBP/3R2K1 b - - 0 16".to_string(),
            ],
        ));
        scenarios.push((
            "5k2/2r2p2/R1B2npp/3p4/3Pn3/4P2P/4NP2/6K1 w - - 0 31".to_string(),
            vec![
                "4Bk2/2r2p2/R4npp/3p4/3Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "B4k2/2r2p2/R4npp/3p4/3Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "5k2/2rB1p2/R4npp/3p4/3Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "5k2/1Br2p2/R4npp/3p4/3Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/R4npp/3B4/3Pn3/4P2P/4NP2/6K1 b - - 0 31".to_string(),
                "5k2/2r2p2/R4npp/1B1p4/3Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/R4npp/3p4/B2Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "R4k2/2r2p2/2B2npp/3p4/3Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "5k2/R1r2p2/2B2npp/3p4/3Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/1RB2npp/3p4/3Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/2B2npp/R2p4/3Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/2B2npp/3p4/R2Pn3/4P2P/4NP2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/2B2npp/3p4/3Pn3/R3P2P/4NP2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/2B2npp/3p4/3Pn3/4P2P/R3NP2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/2B2npp/3p4/3Pn3/4P2P/4NP2/R5K1 b - - 1 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3PnN2/4P2P/5P2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3Pn3/4P1NP/5P2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3Pn3/2N1P2P/5P2/6K1 b - - 1 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3Pn3/4P2P/5P2/2N3K1 b - - 1 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3Pn3/4P2P/4NP1K/8 b - - 1 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3Pn3/4P2P/4NPK1/8 b - - 1 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3Pn3/4P2P/4NP2/7K b - - 1 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3Pn3/4P2P/4NP2/5K2 b - - 1 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3Pn2P/4P3/4NP2/6K1 b - - 0 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3Pn3/4PP1P/4N3/6K1 b - - 0 31".to_string(),
                "5k2/2r2p2/R1B2npp/3p4/3PnP2/4P2P/4N3/6K1 b - - 0 31".to_string(),
            ],
        ));
        scenarios.push((
            "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q1KP2/7R w - - 0 33".to_string(),
            vec![
                "6k1/2n3rB/1p6/P2P4/2P2r2/q7/2Q1KP2/7R b - - 0 33".to_string(),
                "6k1/2n3rp/1p4B1/P2P4/2P2r2/q7/2Q1KP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P1B2/2P2r2/q7/2Q1KP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P2r2/q4B2/2Q1KP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P2r2/q2B4/2Q1KP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P2r2/q7/2Q1KPB1/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2QK1P2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q2P2/5K1R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q2P2/4K2R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q2P2/3K3R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/Q1P1Br2/q7/4KP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q2Q4/4KP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q1Q5/4KP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/qQ6/4KP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/3QKP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/1Q2KP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/Q3KP2/7R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/4KP2/3Q3R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/4KP2/2Q4R b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/4KP2/1Q5R b - - 1 33".to_string(),
                "6k1/2n3rR/1p6/P2P4/2P1Br2/q7/2Q1KP2/8 b - - 0 33".to_string(),
                "6k1/2n3rp/1p5R/P2P4/2P1Br2/q7/2Q1KP2/8 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P3R/2P1Br2/q7/2Q1KP2/8 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br1R/q7/2Q1KP2/8 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q6R/2Q1KP2/8 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q1KP1R/8 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q1KP2/6R1 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q1KP2/5R2 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q1KP2/4R3 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q1KP2/3R4 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q1KP2/2R5 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q1KP2/1R6 b - - 1 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q7/2Q1KP2/R7 b - - 1 33".to_string(),
                "6k1/2n3rp/1P6/3P4/2P1Br2/q7/2Q1KP2/7R b - - 0 33".to_string(),
                "6k1/2n3rp/1p1P4/P7/2P1Br2/q7/2Q1KP2/7R b - - 0 33".to_string(),
                "6k1/2n3rp/Pp6/3P4/2P1Br2/q7/2Q1KP2/7R b - - 0 33".to_string(),
                "6k1/2n3rp/1p6/P1PP4/4Br2/q7/2Q1KP2/7R b - - 0 33".to_string(),
                "6k1/2n3rp/1p6/P2P4/2P1Br2/q4P2/2Q1K3/7R b - - 0 33".to_string(),
            ],
        ));
        scenarios.push((
            "5R2/8/8/P5k1/3K4/7n/4b3/8 w - - 27 82".to_string(),
            vec![
                "7R/8/8/P5k1/3K4/7n/4b3/8 b - - 28 82".to_string(),
                "6R1/8/8/P5k1/3K4/7n/4b3/8 b - - 28 82".to_string(),
                "4R3/8/8/P5k1/3K4/7n/4b3/8 b - - 28 82".to_string(),
                "3R4/8/8/P5k1/3K4/7n/4b3/8 b - - 28 82".to_string(),
                "2R5/8/8/P5k1/3K4/7n/4b3/8 b - - 28 82".to_string(),
                "1R6/8/8/P5k1/3K4/7n/4b3/8 b - - 28 82".to_string(),
                "R7/8/8/P5k1/3K4/7n/4b3/8 b - - 28 82".to_string(),
                "8/5R2/8/P5k1/3K4/7n/4b3/8 b - - 28 82".to_string(),
                "8/8/5R2/P5k1/3K4/7n/4b3/8 b - - 28 82".to_string(),
                "8/8/8/P4Rk1/3K4/7n/4b3/8 b - - 28 82".to_string(),
                "8/8/8/P5k1/3K1R2/7n/4b3/8 b - - 28 82".to_string(),
                "8/8/8/P5k1/3K4/5R1n/4b3/8 b - - 28 82".to_string(),
                "8/8/8/P5k1/3K4/7n/4bR2/8 b - - 28 82".to_string(),
                "8/8/8/P5k1/3K4/7n/4b3/5R2 b - - 28 82".to_string(),
                "5R2/8/8/P3K1k1/8/7n/4b3/8 b - - 28 82".to_string(),
                "5R2/8/8/P2K2k1/8/7n/4b3/8 b - - 28 82".to_string(),
                "5R2/8/8/P1K3k1/8/7n/4b3/8 b - - 28 82".to_string(),
                "5R2/8/8/P5k1/4K3/7n/4b3/8 b - - 28 82".to_string(),
                "5R2/8/8/P5k1/8/4K2n/4b3/8 b - - 28 82".to_string(),
                "5R2/8/8/P5k1/8/2K4n/4b3/8 b - - 28 82".to_string(),
                "5R2/8/P7/6k1/3K4/7n/4b3/8 b - - 0 82".to_string(),
            ],
        ));
        scenarios.push((
            "4k2r/2nq1p2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 b k - 1 18".to_string(),
            vec![
                "4k1r1/2nq1p2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w - - 2 19".to_string(),
                "4kr2/2nq1p2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w - - 2 19".to_string(),
                "4k3/2nq1p1r/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w - - 2 19".to_string(),
                "5k1r/2nq1p2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w - - 2 19".to_string(),
                "3k3r/2nq1p2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w - - 2 19".to_string(),
                "7r/2nqkp2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w - - 2 19".to_string(),
                "3qk2r/2n2p2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "2q1k2r/2n2p2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2n1qp2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2n2p2/1p1qp1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2n2p2/1pq1p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2n2p2/1p2p1pb/1q1pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "n3k2r/3q1p2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/3q1p2/np2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/3q1p2/1p2p1pb/1n1pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4kb1r/2nq1p2/1p2p1p1/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1pb1/1p2p1p1/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/1p2p1p1/3pPnbp/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/1p2p1p1/3pPn1p/rN1P1b2/5N2/3B2PP/1RRQ2K1 w k - 0 19".to_string(),
                "4k2r/2nq1pn1/1p2p1pb/3pP2p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nqnp2/1p2p1pb/3pP2p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/1p1np1pb/3pP2p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/1p2p1pb/3pP2p/rN1P1P1n/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/1p2p1pb/3pP2p/rN1n1P2/5N2/3B2PP/1RRQ2K1 w k - 0 19".to_string(),
                "4k2r/2nq1p2/1p2p1pb/3pP2p/rN1P1P2/5Nn1/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/1p2p1pb/3pP2p/rN1P1P2/4nN2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "r3k2r/2nq1p2/1p2p1pb/3pPn1p/1N1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/r1nq1p2/1p2p1pb/3pPn1p/1N1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/rp2p1pb/3pPn1p/1N1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/1p2p1pb/r2pPn1p/1N1P1P2/5N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/1p2p1pb/3pPn1p/1r1P1P2/5N2/3B2PP/1RRQ2K1 w k - 0 19".to_string(),
                "4k2r/2nq1p2/1p2p1pb/3pPn1p/1N1P1P2/r4N2/3B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/1p2p1pb/3pPn1p/1N1P1P2/5N2/r2B2PP/1RRQ2K1 w k - 2 19".to_string(),
                "4k2r/2nq1p2/1p2p1pb/3pPn1p/1N1P1P2/5N2/3B2PP/rRRQ2K1 w k - 2 19".to_string(),
                "5rk1/2nq1p2/1p2p1pb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w - - 2 19".to_string(),
                "4k2r/2nq4/1p2pppb/3pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 0 19".to_string(),
                "4k2r/2nq1p2/1p2p2b/3pPnpp/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 0 19".to_string(),
                "4k2r/2nq1p2/4p1pb/1p1pPn1p/rN1P1P2/5N2/3B2PP/1RRQ2K1 w k - 0 19".to_string(),
                "4k2r/2nq1p2/1p2p1pb/3pPn2/rN1P1P1p/5N2/3B2PP/1RRQ2K1 w k - 0 19".to_string(),
            ],
        ));
        scenarios.push((
            "r3r1k1/p1q2pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 b - - 4 20".to_string(),
            vec![
                "r3r2k/p1q2pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r3/p1q2ppk/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r4rk1/p1q2pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r2r2k1/p1q2pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r1r3k1/p1q2pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "rr4k1/p1q2pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r5k1/p1q1rpp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r5k1/p1q2pp1/4r2p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r5k1/p1q2pp1/7p/Pp2r3/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "3rr1k1/p1q2pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "2r1r1k1/p1q2pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "1r2r1k1/p1q2pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r2qr1k1/p4pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r1q1r1k1/p4pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "rq2r1k1/p4pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p3qpp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p2q1pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/pq3pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p4pp1/3q3p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p4pp1/2q4p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p4pp1/1q5p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p4pp1/7p/Pp2q3/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p4pp1/7p/Ppq5/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p4pp1/7p/qp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 0 21".to_string(),
                "r3r1k1/p4pp1/7p/Pp6/1Bq1nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p4pp1/7p/Pp6/1B2nb2/1Qq2N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p4pp1/7p/Pp6/1B2nb2/1Q3N1P/1Pq2PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p4pp1/7p/Pp6/1B2nb2/1Q3N1P/1P3PP1/2qR1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/3b3p/Pp6/1B2n3/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp4b1/1B2n3/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp2b3/1B2n3/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp6/1B2n3/1Q3NbP/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp6/1B2n3/1Q2bN1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp6/1B2n3/1Q3N1P/1P3PPb/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp6/1B2n3/1Q3N1P/1P1b1PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp6/1B2n3/1Q3N1P/1P3PP1/2bR1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/5n1p/Pp6/1B3b2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/3n3p/Pp6/1B3b2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp4n1/1B3b2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Ppn5/1B3b2/1Q3N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp6/1B3b2/1Q3NnP/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp6/1B3b2/1Qn2N1P/1P3PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp6/1B3b2/1Q3N1P/1P3nP1/3R1RK1 w - - 0 21".to_string(),
                "r3r1k1/p1q2pp1/7p/Pp6/1B3b2/1Q3N1P/1P1n1PP1/3R1RK1 w - - 5 21".to_string(),
                "r3r1k1/p1q2p2/6pp/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 0 21".to_string(),
                "r3r1k1/2q2pp1/p6p/Pp6/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 0 21".to_string(),
                "r3r1k1/p1q2pp1/8/Pp5p/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 0 21".to_string(),
                "r3r1k1/p1q2p2/7p/Pp4p1/1B2nb2/1Q3N1P/1P3PP1/3R1RK1 w - - 0 21".to_string(),
            ],
        ));
        scenarios.push((
            "8/8/1p2Rpk1/p1pP2p1/P4b2/1P5r/4R3/4K3 b - - 1 58".to_string(),
            vec![
                "8/7k/1p2Rp2/p1pP2p1/P4b2/1P5r/4R3/4K3 w - - 2 59".to_string(),
                "8/6k1/1p2Rp2/p1pP2p1/P4b2/1P5r/4R3/4K3 w - - 2 59".to_string(),
                "8/5k2/1p2Rp2/p1pP2p1/P4b2/1P5r/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rp1k/p1pP2p1/P4b2/1P5r/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rp2/p1pP2pk/P4b2/1P5r/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rp2/p1pP1kp1/P4b2/1P5r/4R3/4K3 w - - 2 59".to_string(),
                "1b6/8/1p2Rpk1/p1pP2p1/P7/1P5r/4R3/4K3 w - - 2 59".to_string(),
                "8/2b5/1p2Rpk1/p1pP2p1/P7/1P5r/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p1bRpk1/p1pP2p1/P7/1P5r/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pPb1p1/P7/1P5r/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P7/1P4br/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P7/1P2b2r/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P7/1P5r/4R2b/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P7/1P5r/3bR3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P7/1P5r/4R3/2b1K3 w - - 2 59".to_string(),
                "7r/8/1p2Rpk1/p1pP2p1/P4b2/1P6/4R3/4K3 w - - 2 59".to_string(),
                "8/7r/1p2Rpk1/p1pP2p1/P4b2/1P6/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpkr/p1pP2p1/P4b2/1P6/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2pr/P4b2/1P6/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P4b1r/1P6/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P4b2/1P4r1/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P4b2/1P3r2/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P4b2/1P2r3/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P4b2/1P1r4/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P4b2/1Pr5/4R3/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P4b2/1r6/4R3/4K3 w - - 0 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P4b2/1P6/4R2r/4K3 w - - 2 59".to_string(),
                "8/8/1p2Rpk1/p1pP2p1/P4b2/1P6/4R3/4K2r w - - 2 59".to_string(),
                "8/8/4Rpk1/pppP2p1/P4b2/1P5r/4R3/4K3 w - - 0 59".to_string(),
                "8/8/1p2Rpk1/p1pP4/P4bp1/1P5r/4R3/4K3 w - - 0 59".to_string(),
                "8/8/1p2Rpk1/p2P2p1/P1p2b2/1P5r/4R3/4K3 w - - 0 59".to_string(),
            ],
        ));
        scenarios.push((
            "r1b2rk1/p1q4p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 b - - 0 12".to_string(),
            vec![
                "r1b2r1k/p1q4p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b2r2/p1q3kp/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b2r2/p1q2k1p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b1r1k1/p1q4p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1br2k1/p1q4p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b3k1/p1q2r1p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r4rk1/p1qb3p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r4rk1/pbq4p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r4rk1/p1q4p/1pp1bpp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r4rk1/p1q4p/bpp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r4rk1/p1q4p/1pp2pp1/3pBb2/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r4rk1/p1q4p/1pp2pp1/3pB3/6b1/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r4rk1/p1q4p/1pp2pp1/3pB3/8/2PB3b/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "1rb2rk1/p1q4p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1bq1rk1/p6p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "rqb2rk1/p6p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b2rk1/p5qp/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b2rk1/p4q1p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b2rk1/p3q2p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b2rk1/p2q3p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b2rk1/pq5p/1pp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b2rk1/p6p/1ppq1pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 1 13".to_string(),
                "r1b2rk1/p6p/1pp2pp1/3pq3/8/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "r1b2rk1/p1q4p/1pp3p1/3pp3/8/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "r1b2rk1/p1q5/1pp2ppp/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "r1b2rk1/2q4p/ppp2pp1/3pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "r1b2rk1/p1q4p/1pp2p2/3pB1p1/8/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "r1b2rk1/p1q4p/1pp3p1/3pBp2/8/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "r1b2rk1/p1q4p/1p3pp1/2ppB3/8/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "r1b2rk1/p1q4p/2p2pp1/1p1pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "r1b2rk1/p1q4p/1pp2pp1/4B3/3p4/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "r1b2rk1/p1q5/1pp2pp1/3pB2p/8/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
                "r1b2rk1/2q4p/1pp2pp1/p2pB3/8/2PB3P/PP2QPP1/4RRK1 w - - 0 13".to_string(),
            ],
        ));
        scenarios.push((
            "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP3/6PP/R1B2RK1 w - - 0 13".to_string(),
            vec![
                "3q1rk1/Q3bpp1/1pr2n1p/2p5/1P1p1P2/P1NPP3/6PP/R1B2RK1 b - - 0 13".to_string(),
                "3q1rk1/p3bpp1/1pQ2n1p/2p5/1P1p1P2/P1NPP3/6PP/R1B2RK1 b - - 0 13".to_string(),
                "3q1rk1/p3bpp1/Qpr2n1p/2p5/1P1p1P2/P1NPP3/6PP/R1B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/1Qp5/1P1p1P2/P1NPP3/6PP/R1B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/Q1p5/1P1p1P2/P1NPP3/6PP/R1B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/1P1p1P2/PQNPP3/6PP/R1B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/1P1p1P2/P1NPP3/2Q3PP/R1B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/1P1p1P2/P1NPP3/6PP/R1BQ1RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2pN4/QP1p1P2/P2PP3/6PP/R1B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/1Np5/QP1p1P2/P2PP3/6PP/R1B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1pNP2/P2PP3/6PP/R1B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P2PP3/4N1PP/R1B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P2PP3/N5PP/R1B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P2PP3/6PP/R1BN1RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P2PP3/6PP/RNB2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP3/5KPP/R1B2R2 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP3/6PP/R1B2R1K b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPPR2/6PP/R1B3K1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP3/5RPP/R1B3K1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP3/6PP/R1B1R1K1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP3/6PP/R1BR2K1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP3/3B2PP/R4RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP3/1B4PP/R4RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP3/R5PP/2B2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP3/6PP/1RB2RK1 b - - 1 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2P5/Q2p1P2/P1NPP3/6PP/R1B2RK1 b - - 0 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1P1P2/P1NP4/6PP/R1B2RK1 b - - 0 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p2P2/QP1p4/P1NPP3/6PP/R1B2RK1 b - - 0 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/1Pp5/Q2p1P2/P1NPP3/6PP/R1B2RK1 b - - 0 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1pPP2/P1NP4/6PP/R1B2RK1 b - - 0 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP2P/6P1/R1B2RK1 b - - 0 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P2/P1NPP1P1/7P/R1B2RK1 b - - 0 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1P1P/P1NPP3/6P1/R1B2RK1 b - - 0 13".to_string(),
                "3q1rk1/p3bpp1/1pr2n1p/2p5/QP1p1PP1/P1NPP3/7P/R1B2RK1 b - - 0 13".to_string(),
            ],
        ));
        scenarios.push((
            "8/2p5/3b2k1/1p3q2/5B1P/6Q1/6K1/8 b - - 2 56".to_string(),
            vec![
                "8/2p4k/3b4/1p3q2/5B1P/6Q1/6K1/8 w - - 3 57".to_string(),
                "8/2p2k2/3b4/1p3q2/5B1P/6Q1/6K1/8 w - - 3 57".to_string(),
                "8/2p5/3b1k2/1p3q2/5B1P/6Q1/6K1/8 w - - 3 57".to_string(),
                "8/2p5/3b4/1p3q1k/5B1P/6Q1/6K1/8 w - - 3 57".to_string(),
                "8/2p5/3b2k1/1p4q1/5B1P/6Q1/6K1/8 w - - 3 57".to_string(),
                "8/2p5/3b2k1/1p6/5BqP/6Q1/6K1/8 w - - 3 57".to_string(),
            ],
        ));
        scenarios.push((
            "6k1/pprb2p1/4p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 b - - 1 21".to_string(),
            vec![
                "7k/pprb2p1/4p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "5k2/pprb2p1/4p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "8/pprb2pk/4p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "8/pprb1kp1/4p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "4b1k1/ppr3p1/4p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "2b3k1/ppr3p1/4p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "6k1/ppr3p1/2b1p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "6k1/ppr3p1/4p2p/1b1p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "6k1/ppr3p1/4p2p/3p1p2/b2P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "2r3k1/pp1b2p1/4p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "6k1/pp1b2p1/2r1p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "6k1/pp1b2p1/4p2p/2rp1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "6k1/pp1b2p1/4p2p/3p1p2/2rP3P/3BP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "6k1/pp1b2p1/4p2p/3p1p2/3P3P/2rBP1P1/PP3P2/4R1K1 w - - 2 22".to_string(),
                "6k1/pp1b2p1/4p2p/3p1p2/3P3P/3BP1P1/PPr2P2/4R1K1 w - - 2 22".to_string(),
                "6k1/pp1b2p1/4p2p/3p1p2/3P3P/3BP1P1/PP3P2/2r1R1K1 w - - 2 22".to_string(),
                "6k1/pprb4/4p1pp/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 0 22".to_string(),
                "6k1/p1rb2p1/1p2p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 0 22".to_string(),
                "6k1/1prb2p1/p3p2p/3p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 0 22".to_string(),
                "6k1/pprb2p1/4p3/3p1p1p/3P3P/3BP1P1/PP3P2/4R1K1 w - - 0 22".to_string(),
                "6k1/pprb2p1/7p/3ppp2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 0 22".to_string(),
                "6k1/pprb2p1/4p2p/3p4/3P1p1P/3BP1P1/PP3P2/4R1K1 w - - 0 22".to_string(),
                "6k1/pprb4/4p2p/3p1pp1/3P3P/3BP1P1/PP3P2/4R1K1 w - - 0 22".to_string(),
                "6k1/p1rb2p1/4p2p/1p1p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 0 22".to_string(),
                "6k1/1prb2p1/4p2p/p2p1p2/3P3P/3BP1P1/PP3P2/4R1K1 w - - 0 22".to_string(),
            ],
        ));
        scenarios.push((
            "8/8/5kp1/5p1p/5P1P/6P1/Bp2K3/6b1 b - - 2 54".to_string(),
            vec![
                "8/6k1/6p1/5p1p/5P1P/6P1/Bp2K3/6b1 w - - 3 55".to_string(),
                "8/4k3/6p1/5p1p/5P1P/6P1/Bp2K3/6b1 w - - 3 55".to_string(),
                "8/b7/5kp1/5p1p/5P1P/6P1/Bp2K3/8 w - - 3 55".to_string(),
                "8/8/1b3kp1/5p1p/5P1P/6P1/Bp2K3/8 w - - 3 55".to_string(),
                "8/8/5kp1/2b2p1p/5P1P/6P1/Bp2K3/8 w - - 3 55".to_string(),
                "8/8/5kp1/5p1p/3b1P1P/6P1/Bp2K3/8 w - - 3 55".to_string(),
                "8/8/5kp1/5p1p/5P1P/4b1P1/Bp2K3/8 w - - 3 55".to_string(),
                "8/8/5kp1/5p1p/5P1P/6P1/Bp2K2b/8 w - - 3 55".to_string(),
                "8/8/5kp1/5p1p/5P1P/6P1/Bp2Kb2/8 w - - 3 55".to_string(),
                "8/8/5k2/5ppp/5P1P/6P1/Bp2K3/6b1 w - - 0 55".to_string(),
                "8/8/5kp1/5p1p/5P1P/6P1/B3K3/1q4b1 w - - 0 55".to_string(),
                "8/8/5kp1/5p1p/5P1P/6P1/B3K3/1r4b1 w - - 0 55".to_string(),
                "8/8/5kp1/5p1p/5P1P/6P1/B3K3/1b4b1 w - - 0 55".to_string(),
                "8/8/5kp1/5p1p/5P1P/6P1/B3K3/1n4b1 w - - 0 55".to_string(),
            ],
        ));
        scenarios.push((
            "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1Np2P/1P2QPP1/6K1 w - - 0 23".to_string(),
            vec![
                "R1r3k1/5pp1/1q2p2p/8/1p2n3/1P1Np2P/1P2QPP1/6K1 b - - 1 23".to_string(),
                "2r3k1/R4pp1/1q2p2p/8/1p2n3/1P1Np2P/1P2QPP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/Rq2p2p/8/1p2n3/1P1Np2P/1P2QPP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/R7/1p2n3/1P1Np2P/1P2QPP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/1R2n3/1P1Np2P/1P2QPP1/6K1 b - - 0 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/1p2n3/RP1Np2P/1P2QPP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/1p2n3/1P1Np2P/RP2QPP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/1p2n3/1P1Np2P/1P2QPP1/R5K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/4N3/Rp2n3/1P2p2P/1P2QPP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/2N5/Rp2n3/1P2p2P/1P2QPP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2nN2/1P2p2P/1P2QPP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/RN2n3/1P2p2P/1P2QPP1/6K1 b - - 0 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P2p2P/1P2QPP1/4N1K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P2p2P/1P2QPP1/2N3K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/7Q/Rp2n3/1P1Np2P/1P3PP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n1Q1/1P1Np2P/1P3PP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1NpQ1P/1P3PP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1NQ2P/1P3PP1/6K1 b - - 0 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1Np2P/1P1Q1PP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1Np2P/1PQ2PP1/6K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1Np2P/1P3PP1/5QK1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1Np2P/1P3PP1/4Q1K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1Np2P/1P3PP1/3Q2K1 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1Np2P/1P2QPPK/8 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1Np2P/1P2QPP1/7K b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1Np2P/1P2QPP1/5K2 b - - 1 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1NP2P/1P2Q1P1/6K1 b - - 0 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n2P/1P1Np3/1P2QPP1/6K1 b - - 0 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1Np1PP/1P2QP2/6K1 b - - 0 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n3/1P1NpP1P/1P2Q1P1/6K1 b - - 0 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2n1P1/1P1Np2P/1P2QP2/6K1 b - - 0 23".to_string(),
                "2r3k1/5pp1/1q2p2p/8/Rp2nP2/1P1Np2P/1P2Q1P1/6K1 b - - 0 23".to_string(),
            ],
        ));
        scenarios.push((
            "4rb1k/2r2p1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 b - - 4 36".to_string(),
            vec![
                "4rbk1/2r2p1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4r2k/2r2pbb/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4r2k/2r1bp1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "3r1b1k/2r2p1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "2r2b1k/2r2p1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "1r3b1k/2r2p1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "r4b1k/2r2p1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "5b1k/2r1rp1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "5b1k/2r2p1b/3qrp1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "5b1k/2r2p1b/3q1p1p/3BrP1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "5b1k/2r2p1b/3q1p1p/3B1P1N/pp1Pr1P1/2pQ3P/P1R5/1R4K1 w - - 0 37".to_string(),
                "4rbbk/2r2p2/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p2/3q1pbp/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p2/3q1p1p/3B1b1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 0 37".to_string(),
                "2r1rb1k/5p1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/4rp1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/3r1p1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/1r3p1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/r4p1b/3q1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/5p1b/2rq1p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/5p1b/3q1p1p/2rB1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/5p1b/3q1p1p/3B1P1N/pprPP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "3qrb1k/2r2p1b/5p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r1qp1b/5p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2rq1p1b/5p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p1b/4qp1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p1b/2q2p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p1b/1q3p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p1b/q4p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p1b/5p1p/3BqP1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p1b/5p1p/3q1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 0 37".to_string(),
                "4rb1k/2r2p1b/5p1p/2qB1P1N/pp1PP1P1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p1b/5p1p/3B1P1N/pp1PPqP1/2pQ3P/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p1b/5p1p/3B1P1N/pp1PP1P1/2pQ2qP/P1R5/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p1b/5p1p/3B1P1N/pp1PP1P1/2pQ3P/P1R4q/1R4K1 w - - 5 37".to_string(),
                "4rb1k/2r2p1b/3q1p1p/3B1P1N/p2PP1P1/1ppQ3P/P1R5/1R4K1 w - - 0 37".to_string(),
                "4rb1k/2r2p1b/3q1p1p/3B1P1N/1p1PP1P1/p1pQ3P/P1R5/1R4K1 w - - 0 37".to_string(),
            ],
        ));
        scenarios.push((
            "2nb4/1ppk3Q/4p3/3pP3/3P4/3PRN2/5PK1/3q4 b - - 2 39".to_string(),
            vec![
                "2nbk3/1pp4Q/4p3/3pP3/3P4/3PRN2/5PK1/3q4 w - - 3 40".to_string(),
                "2nb4/1pp4Q/2k1p3/3pP3/3P4/3PRN2/5PK1/3q4 w - - 3 40".to_string(),
                "2n5/1ppkb2Q/4p3/3pP3/3P4/3PRN2/5PK1/3q4 w - - 3 40".to_string(),
                "3b4/1ppkn2Q/4p3/3pP3/3P4/3PRN2/5PK1/3q4 w - - 3 40".to_string(),
            ],
        ));
        scenarios.push((
            "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4P3/3NBPPP/BN1Q1RK1 w - - 0 10".to_string(),
            vec![
                "1n1q1rk1/1bp1ppbp/B5p1/2pp4/1P1Pn3/4P3/3N1PPP/BN1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp3B/1P1Pn3/4P3/3N1PPP/BN1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/1Bpp4/1P1Pn3/4P3/3N1PPP/BN1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn1B1/4P3/3N1PPP/BN1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1PBPn3/4P3/3N1PPP/BN1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4PB2/3N1PPP/BN1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/3BP3/3N1PPP/BN1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1PN3/4P3/4BPPP/BN1Q1RK1 b - - 0 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1PNPn3/4P3/4BPPP/BN1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4PN2/4BPPP/BN1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/1N2P3/4BPPP/BN1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4P3/3NBPPP/BN1Q1R1K b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4P3/3NBPPP/BN1QR1K1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/QP1Pn3/4P3/3NBPPP/BN3RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/1Q2P3/3NBPPP/BN3RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4P3/2QNBPPP/BN3RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4P3/3NBPPP/BN2QRK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4P3/3NBPPP/BNQ2RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/2N1P3/3NBPPP/B2Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/N3P3/3NBPPP/B2Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/2B1P3/3NBPPP/1N1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4P3/1B1NBPPP/1N1Q1RK1 b - - 1 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2Pp4/1P2n3/4P3/3NBPPP/BN1Q1RK1 b - - 0 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2Pp4/3Pn3/4P3/3NBPPP/BN1Q1RK1 b - - 0 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/1Ppp4/3Pn3/4P3/3NBPPP/BN1Q1RK1 b - - 0 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4P2P/3NBPP1/BN1Q1RK1 b - - 0 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4P1P1/3NBP1P/BN1Q1RK1 b - - 0 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn3/4PP2/3NB1PP/BN1Q1RK1 b - - 0 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn2P/4P3/3NBPP1/BN1Q1RK1 b - - 0 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1Pn1P1/4P3/3NBP1P/BN1Q1RK1 b - - 0 10".to_string(),
                "1n1q1rk1/1bp1ppbp/6p1/2pp4/1P1PnP2/4P3/3NB1PP/BN1Q1RK1 b - - 0 10".to_string(),
            ],
        ));
        scenarios.push((
            "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/7R/n2BRKP1/3q4 w - - 0 43".to_string(),
            vec![
                "4r1k1/5pbN/rp1p2p1/p1pP4/4PPQ1/7R/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5Nb1/rp1p2p1/p1pP4/4PPQ1/7R/n2BRKP1/3q4 b - - 0 43".to_string(),
                "4r1k1/5pb1/rp1pN1p1/p1pP4/4PPQ1/7R/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP4/4PPQ1/5N1R/n2BRKP1/3q4 b - - 1 43".to_string(),
                "2Q1r1k1/5pb1/rp1p2p1/p1pP2N1/4PP2/7R/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/3Q1pb1/rp1p2p1/p1pP2N1/4PP2/7R/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1pQ1p1/p1pP2N1/4PP2/7R/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2NQ/4PP2/7R/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP1QN1/4PP2/7R/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PP1Q/7R/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PP2/6QR/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PP2/5Q1R/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1kR/5pb1/rp1p2p1/p1pP2N1/4PPQ1/8/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pbR/rp1p2p1/p1pP2N1/4PPQ1/8/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2pR/p1pP2N1/4PPQ1/8/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2NR/4PPQ1/8/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQR/8/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/6R1/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/5R2/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/4R3/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/3R4/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/2R5/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/1R6/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/R7/n2BRKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/8/n2BRKPR/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/8/n2BRKP1/3q3R b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/6KR/n2BR1P1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/5K1R/n2BR1P1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/4K2R/n2BR1P1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/4R2R/n2B1KP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/7R/n2B1KP1/3qR3 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/B1pP2N1/4PPQ1/7R/n3RKP1/3q4 b - - 0 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/1B2PPQ1/7R/n3RKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/4B2R/n3RKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/2B4R/n3RKP1/3q4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/7R/n3RKP1/3qB3 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/7R/n3RKP1/2Bq4 b - - 1 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP1PN1/4P1Q1/7R/n2BRKP1/3q4 b - - 0 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pPP1N1/5PQ1/7R/n2BRKP1/3q4 b - - 0 43".to_string(),
                "4r1k1/5pb1/rp1p2p1/p1pP2N1/4PPQ1/6PR/n2BRK2/3q4 b - - 0 43".to_string(),
            ],
        ));
        scenarios.push((
            "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P1N1P/R1R2BK1 w - - 6 20".to_string(),
            vec![
                "r1b2rk1/1p4pp/pb3p2/n2p2N1/5P2/4P1P1/3P1N1P/R1R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2pN3/5P2/4P1P1/3P1N1P/R1R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P1N/4P1P1/3P1N1P/R1R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/3N1P2/4P1P1/3P1N1P/R1R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4P1P1/3P1N1P/R1R1NBK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5PN1/4PNP1/3P3P/R1R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/4NP2/4PNP1/3P3P/R1R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNPN/3P3P/R1R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/3NPNP1/3P3P/R1R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P3P/R1R2BKN b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P3P/R1RN1BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P1NKP/R1R2B2 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P1N1P/R1R2B1K b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/Bb3p2/n2p4/5P2/4PNP1/3P1N1P/R1R3K1 b - - 0 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/nB1p4/5P2/4PNP1/3P1N1P/R1R3K1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/2B2P2/4PNP1/3P1N1P/R1R3K1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNPB/3P1N1P/R1R3K1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/3BPNP1/3P1N1P/R1R3K1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P1NBP/R1R3K1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3PBN1P/R1R3K1 b - - 7 20".to_string(),
                "r1R2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P1N1P/R4BK1 b - - 0 20".to_string(),
                "r1b2rk1/1pR3pp/pb3p2/n2p4/5P2/4PNP1/3P1N1P/R4BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pbR2p2/n2p4/5P2/4PNP1/3P1N1P/R4BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n1Rp4/5P2/4PNP1/3P1N1P/R4BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/2R2P2/4PNP1/3P1N1P/R4BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/2R1PNP1/3P1N1P/R4BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/2RP1N1P/R4BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P1N1P/R3RBK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P1N1P/R2R1BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P1N1P/RR3BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/R2p4/5P2/4PNP1/3P1N1P/2R2BK1 b - - 0 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/R4P2/4PNP1/3P1N1P/2R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/R3PNP1/3P1N1P/2R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/R2P1N1P/2R2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNP1/3P1N1P/1RR2BK1 b - - 7 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p1P2/8/4PNP1/3P1N1P/R1R2BK1 b - - 0 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5PP1/4PN2/3P1N1P/R1R2BK1 b - - 0 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/4PP2/5NP1/3P1N1P/R1R2BK1 b - - 0 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/4PNPP/3P1N2/R1R2BK1 b - - 0 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P2/3PPNP1/5N1P/R1R2BK1 b - - 0 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/5P1P/4PNP1/3P1N2/R1R2BK1 b - - 0 20".to_string(),
                "r1b2rk1/1p4pp/pb3p2/n2p4/3P1P2/4PNP1/5N1P/R1R2BK1 b - - 0 20".to_string(),
            ],
        ));
        scenarios.push((
            "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PN1PPP/R2QK2R w KQkq - 6 6".to_string(),
            vec![
                "rBb1k2r/pp3ppp/4pn2/q1pp4/3Pn3/P3PB2/P1PN1PPP/R2QK2R b KQkq - 7 6".to_string(),
                "r1b1k2r/ppB2ppp/4pn2/q1pp4/3Pn3/P3PB2/P1PN1PPP/R2QK2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn1B/q1pp4/3Pn3/P3PB2/P1PN1PPP/R2QK2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/3Bpn2/q1pp4/3Pn3/P3PB2/P1PN1PPP/R2QK2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp2B1/3Pn3/P3PB2/P1PN1PPP/R2QK2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1ppB3/3Pn3/P3PB2/P1PN1PPP/R2QK2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3Pn3/P3PBB1/P1PN1PPP/R2QK2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp3B/3PnB2/P3P3/P1PN1PPP/R2QK2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnBB1/P3P3/P1PN1PPP/R2QK2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PBB2/P3P3/P1PN1PPP/R2QK2R b KQkq - 0 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3P3/P1PNBPPP/R2QK2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PN1PPP/R2QK1R1 b Qkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PN1PPP/R2QKR2 b Qkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PNKPPP/R2Q3R b kq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PN1PPP/R2Q1K1R b kq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PNQPPP/R3K2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PN1PPP/R1Q1K2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PN1PPP/RQ2K2R b KQkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PN1PPP/2RQK2R b Kkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PN1PPP/1R1QK2R b Kkq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB2/P1PN1PPP/R2Q1RK1 b kq - 7 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1Pp4/4nB2/P3PB2/P1PN1PPP/R2QK2R b KQkq - 0 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/P2PnB2/4PB2/P1PN1PPP/R2QK2R b KQkq - 0 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PB1P/P1PN1PP1/R2QK2R b KQkq - 0 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P3PBP1/P1PN1P1P/R2QK2R b KQkq - 0 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB2/P1P1PB2/P2N1PPP/R2QK2R b KQkq - 0 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnB1P/P3PB2/P1PN1PP1/R2QK2R b KQkq - 0 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/3PnBP1/P3PB2/P1PN1P1P/R2QK2R b KQkq - 0 6".to_string(),
                "r1b1k2r/pp3ppp/4pn2/q1pp4/2PPnB2/P3PB2/P2N1PPP/R2QK2R b KQkq - 0 6".to_string(),
            ],
        ));
        scenarios.push((
            "8/8/8/2n5/pKp5/2P2k2/8/8 b - - 5 65".to_string(),
            vec![
                "8/3n4/8/8/pKp5/2P2k2/8/8 w - - 6 66".to_string(),
                "8/1n6/8/8/pKp5/2P2k2/8/8 w - - 6 66".to_string(),
                "8/8/4n3/8/pKp5/2P2k2/8/8 w - - 6 66".to_string(),
                "8/8/n7/8/pKp5/2P2k2/8/8 w - - 6 66".to_string(),
                "8/8/8/8/pKp1n3/2P2k2/8/8 w - - 6 66".to_string(),
                "8/8/8/8/pKp5/2Pn1k2/8/8 w - - 6 66".to_string(),
                "8/8/8/8/pKp5/1nP2k2/8/8 w - - 6 66".to_string(),
                "8/8/8/2n5/pKp3k1/2P5/8/8 w - - 6 66".to_string(),
                "8/8/8/2n5/pKp2k2/2P5/8/8 w - - 6 66".to_string(),
                "8/8/8/2n5/pKp1k3/2P5/8/8 w - - 6 66".to_string(),
                "8/8/8/2n5/pKp5/2P3k1/8/8 w - - 6 66".to_string(),
                "8/8/8/2n5/pKp5/2P1k3/8/8 w - - 6 66".to_string(),
                "8/8/8/2n5/pKp5/2P5/6k1/8 w - - 6 66".to_string(),
                "8/8/8/2n5/pKp5/2P5/5k2/8 w - - 6 66".to_string(),
                "8/8/8/2n5/pKp5/2P5/4k3/8 w - - 6 66".to_string(),
                "8/8/8/2n5/1Kp5/p1P2k2/8/8 w - - 0 66".to_string(),
            ],
        ));
        scenarios.push((
            "8/7b/8/3NN3/8/2K5/k7/8 b - - 87 120".to_string(),
            vec![
                "6b1/8/8/3NN3/8/2K5/k7/8 w - - 88 121".to_string(),
                "8/8/6b1/3NN3/8/2K5/k7/8 w - - 88 121".to_string(),
                "8/8/8/3NNb2/8/2K5/k7/8 w - - 88 121".to_string(),
                "8/8/8/3NN3/4b3/2K5/k7/8 w - - 88 121".to_string(),
                "8/8/8/3NN3/8/2Kb4/k7/8 w - - 88 121".to_string(),
                "8/8/8/3NN3/8/2K5/k1b5/8 w - - 88 121".to_string(),
                "8/8/8/3NN3/8/2K5/k7/1b6 w - - 88 121".to_string(),
                "8/7b/8/3NN3/8/k1K5/8/8 w - - 88 121".to_string(),
                "8/7b/8/3NN3/8/2K5/8/1k6 w - - 88 121".to_string(),
                "8/7b/8/3NN3/8/2K5/8/k7 w - - 88 121".to_string(),
            ],
        ));
        scenarios.push((
            "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B1N1P1P/R2Q1BRK w - - 0 22".to_string(),
            vec![
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1N1/1P2PnP1/2P5/1B1N1P1P/R2Q1BRK b - - 0 22".to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1N1p1/1P2PnP1/2P5/1B1N1P1P/R2Q1BRK b - - 0 22".to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnPN/2P5/1B1N1P1P/R2Q1BRK b - - 1 22".to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P1NPnP1/2P5/1B1N1P1P/R2Q1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P5/1B1N1P1P/R2QNBRK b - - 1 22".to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1PN1PnP1/2P2N2/1B3P1P/R2Q1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/1NP2N2/1B3P1P/R2Q1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B3P1P/RN1Q1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/B1P2N2/3N1P1P/R2Q1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/3N1P1P/R1BQ1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2NR1/1B1N1P1P/R2Q1B1K b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B1N1PRP/R2Q1B1K b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1Bp1p1p1/1P2PnP1/2P2N2/1B1N1P1P/R2Q2RK b - - 0 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1PB1PnP1/2P2N2/1B1N1P1P/R2Q2RK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N1B/1B1N1P1P/R2Q2RK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2PB1N2/1B1N1P1P/R2Q2RK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B1N1PBP/R2Q2RK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B1NBP1P/R2Q2RK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/QP2PnP1/2P2N2/1B1N1P1P/R4BRK b - - 1 22".to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/1QP2N2/1B1N1P1P/R4BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B1NQP1P/R4BRK b - - 1 22".to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1BQN1P1P/R4BRK b - - 1 22".to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B1N1P1P/R3QBRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B1N1P1P/R1Q2BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B1N1P1P/RQ3BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/Rpp1p1p1/1P2PnP1/2P2N2/1B1N1P1P/3Q1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/RP2PnP1/2P2N2/1B1N1P1P/3Q1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/R1P2N2/1B1N1P1P/3Q1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/RB1N1P1P/3Q1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B1N1P1P/2RQ1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N2/1B1N1P1P/1R1Q1BRK b - - 1 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pP1p1p1/4PnP1/2P2N2/1B1N1P1P/R2Q1BRK b - - 0 22".to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1PP1PnP1/5N2/1B1N1P1P/R2Q1BRK b - - 0 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnP1/2P2N1P/1B1N1P2/R2Q1BRK b - - 0 22"
                    .to_string(),
                "r1q2r1k/p2bb2p/P2pn3/1pp1p1p1/1P2PnPP/2P2N2/1B1N1P2/R2Q1BRK b - - 0 22"
                    .to_string(),
            ],
        ));
        scenarios.push((
            "8/8/5k2/p3b3/1p2B2P/1P3K2/P7/8 b - - 56 76".to_string(),
            vec![
                "8/6k1/8/p3b3/1p2B2P/1P3K2/P7/8 w - - 57 77".to_string(),
                "8/5k2/8/p3b3/1p2B2P/1P3K2/P7/8 w - - 57 77".to_string(),
                "8/4k3/8/p3b3/1p2B2P/1P3K2/P7/8 w - - 57 77".to_string(),
                "8/8/4k3/p3b3/1p2B2P/1P3K2/P7/8 w - - 57 77".to_string(),
                "1b6/8/5k2/p7/1p2B2P/1P3K2/P7/8 w - - 57 77".to_string(),
                "8/2b5/5k2/p7/1p2B2P/1P3K2/P7/8 w - - 57 77".to_string(),
                "8/8/3b1k2/p7/1p2B2P/1P3K2/P7/8 w - - 57 77".to_string(),
                "8/8/5k2/p7/1p2Bb1P/1P3K2/P7/8 w - - 57 77".to_string(),
                "8/8/5k2/p7/1p1bB2P/1P3K2/P7/8 w - - 57 77".to_string(),
                "8/8/5k2/p7/1p2B2P/1P3Kb1/P7/8 w - - 57 77".to_string(),
                "8/8/5k2/p7/1p2B2P/1Pb2K2/P7/8 w - - 57 77".to_string(),
                "8/8/5k2/p7/1p2B2P/1P3K2/P6b/8 w - - 57 77".to_string(),
                "8/8/5k2/p7/1p2B2P/1P3K2/Pb6/8 w - - 57 77".to_string(),
                "8/8/5k2/p7/1p2B2P/1P3K2/P7/b7 w - - 57 77".to_string(),
                "8/8/5k2/4b3/pp2B2P/1P3K2/P7/8 w - - 0 77".to_string(),
            ],
        ));
        scenarios.push((
            "3rnrk1/p5bp/1p1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 b - - 1 26".to_string(),
            vec![
                "3rnr1k/p5bp/1p1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnr2/p4kbp/1p1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rn1k1/p4rbp/1p1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rn1k1/p5bp/1p1p1r2/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rn1k1/p5bp/1p1p4/3Ppr1q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rn1k1/p5bp/1p1p4/3Pp2q/2P1Nrn1/PP2N3/R7/2BQR1K1 w - - 0 27".to_string(),
                "3r1rk1/p1n3bp/1p1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3r1rk1/p5bp/1p1p1n2/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "2r1nrk1/p5bp/1p1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "1r2nrk1/p5bp/1p1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "r3nrk1/p5bp/1p1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "4nrk1/p2r2bp/1p1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrkb/p6p/1p1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p6p/1p1p3b/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p6p/1p1p1b2/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p4qbp/1p1p4/3Pp3/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p3q/3Pp3/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p2q1/3Pp3/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p4/3Pp1q1/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p4/3Ppq2/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p4/3Pp3/2P1NPnq/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p4/3Pp3/2P1NPn1/PP2N2q/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p4/3Pp3/2P1NPn1/PP2N3/R6q/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p4/3Pp3/2P1NPn1/PP2N3/R7/2BQR1Kq w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p3n/3Pp2q/2P1NP2/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p1n2/3Pp2q/2P1NP2/PP2N3/R7/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p4/3Pp2q/2P1NP2/PP2n3/R7/2BQR1K1 w - - 0 27".to_string(),
                "3rnrk1/p5bp/1p1p4/3Pp2q/2P1NP2/PP2N3/R6n/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p4/3Pp2q/2P1NP2/PP2N3/R4n2/2BQR1K1 w - - 2 27".to_string(),
                "3rnrk1/p5bp/1p1p4/3P3q/2P1Npn1/PP2N3/R7/2BQR1K1 w - - 0 27".to_string(),
                "3rnrk1/p5b1/1p1p3p/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 0 27".to_string(),
                "3rnrk1/6bp/pp1p4/3Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 0 27".to_string(),
                "3rnrk1/p5bp/3p4/1p1Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 0 27".to_string(),
                "3rnrk1/6bp/1p1p4/p2Pp2q/2P1NPn1/PP2N3/R7/2BQR1K1 w - - 0 27".to_string(),
            ],
        ));
        scenarios.push((
            "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BB1P1/R2R2K1 w - - 1 16".to_string(),
            vec![
                "r2r2k1/1p2bppp/p1n1pnb1/6N1/1P3P2/P1N1P2P/3BB1P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/4N3/1P3P2/P1N1P2P/3BB1P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P1N/P1N1P2P/3BB1P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P1N1P2/P1N1P2P/3BB1P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1P2P/3BB1PN/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1P2P/3BB1P1/R2RN1K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/3N4/1P3P2/P3PN1P/3BB1P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/1N6/1P3P2/P3PN1P/3BB1P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P2NP2/P3PN1P/3BB1P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/NP3P2/P3PN1P/3BB1P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P3PN1P/N2BB1P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P3PN1P/3BB1P1/RN1R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/B1n1pnb1/8/1P3P2/P1N1PN1P/3B2P1/R2R2K1 b - - 0 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/1B6/1P3P2/P1N1PN1P/3B2P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1PB2P2/P1N1PN1P/3B2P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1NBPN1P/3B2P1/R2R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3B2P1/R2R1BK1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/4B1P1/R2RB1K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/4B1P1/R1BR2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BB1PK/R2R4 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BBKP1/R2R4 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BB1P1/R2R3K b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BB1P1/R2R1K2 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BB1P1/R4RK1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BB1P1/R3R1K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BB1P1/R1R3K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BB1P1/RR4K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/R2BB1P1/3R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BB1P1/2RR2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PN1P/3BB1P1/1R1R2K1 b - - 2 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/5P2/1P6/P1N1PN1P/3BB1P1/R2R2K1 b - - 0 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/1P6/5P2/P1N1PN1P/3BB1P1/R2R2K1 b - - 0 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P1P/P1N1PN2/3BB1P1/R2R2K1 b - - 0 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P2PP2/P1N2N1P/3BB1P1/R2R2K1 b - - 0 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/PP3P2/2N1PN1P/3BB1P1/R2R2K1 b - - 0 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3P2/P1N1PNPP/3BB3/R2R2K1 b - - 0 16".to_string(),
                "r2r2k1/1p2bppp/p1n1pnb1/8/1P3PP1/P1N1PN1P/3BB3/R2R2K1 b - - 0 16".to_string(),
            ],
        ));

        scenarios
    }
}
