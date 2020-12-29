use std::sync::{Arc, RwLockWriteGuard};

use crate::search::{self, Node};
use crate::board::{self, Board, PieceType};

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

pub fn bloom(leaf: &Arc<Node>, children: RwLockWriteGuard<Vec<Arc<Node>>>) {

    let w_pieces = leaf.board.w_p_bb | leaf.board.w_n_bb | leaf.board.w_b_bb | leaf.board.w_r_bb | leaf.board.w_q_bb | leaf.board.w_k_bb;
    let b_pieces = leaf.board.b_p_bb | leaf.board.b_n_bb | leaf.board.b_b_bb | leaf.board.b_r_bb | leaf.board.b_q_bb | leaf.board.b_k_bb;

    children.extend(gen_pawn_moves(leaf).iter(), w_pieces, b_pieces);
    children.extend(gen_knight_moves(leaf).iter(), w_pieces, b_pieces);
    children.extend(gen_bishop_moves(leaf).iter(), w_pieces, b_pieces);
    children.extend(gen_rook_moves(leaf).iter(), w_pieces, b_pieces);
    children.extend(gen_queen_moves(leaf).iter(), w_pieces, b_pieces);
    children.extend(gen_king_moves(leaf).iter(), w_pieces, b_pieces);
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

fn gen_pawn_moves(leaf: &Arc<Node>, w_pieces: u64, b_pieces: u64) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.is_w_move {
        let mut p_bb = board.w_p_bb;
        while p_bb.count_ones() > 0 {
            // Reset board
            board = leaf.board;

            // Gets bitboard with only lsb set
            let lsb_p_bb = p_bb & (~p_bb + 1);

            // Promotion possibilities
            let promotions = if lsb_p_bb & rank_7_bb > 0 {
                &[PieceType::WN, PieceType::WB, PieceType::WR, PieceType::WQ]
            } else {
                &[PieceType::WP]
            };

            let one_ahead_bb = lsb_p_bb << 8;
            let capture_bbs = if lsb_p_bb & a_file_bb > 0 {
                &[lsb_p_bb << 9]
            } else if lsb_p_bb & h_file_bb > 0 {
                &[lsb_p_bb << 7]
            } else {
                &[lsb_p_bb << 7, lsb_p_bb << 9]
            };
            for promotion in promotions.iter() {
                // Move ahead one square
                board = leaf.board;
                if one_ahead_bb & all_pieces == 0 {
                    board.w_p_bb &= ~lsb_p_bb;
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
                        children.push(Arc::new(Node::spawn(leaf, board)));
                    }
                }

                // Move ahead two squares
                board = leaf.board;
                if lsb_p_bb & rank_2_bb > 0 && one_ahead_bb & all_pieces == 0 && (lsb_p_bb << 16) & all_pieces == 0 {
                    board.w_p_bb &= ~lsb_p_bb;
                    board.w_p_bb |= lsb_p_bb << 16;
                    board.halfmove_clock = 0;
                    board.en_passent = None;

                    //King cannot be in check
                    if !is_attacked(&board, false, board.w_k_bb) {
                        children.push(Arc::new(Node::spawn(leaf, board)));
                    }
                }

                // Check for captures
                for capture_bb in capture_bbs.iter() {
                    let captured_piece = get_piecetype(board, capture_bb & b_pieces);
                    if captured_piece.is_some() {
                        board = leaf.board;
                        board.w_p_bb &= ~lsb_p_bb;
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
                            PieceType::BP => board.b_p_bb &= ~capture_bb,
                            PieceType::BN => board.b_n_bb &= ~capture_bb,
                            PieceType::BB => board.b_b_bb &= ~capture_bb,
                            PieceType::BR => board.b_r_bb &= ~capture_bb,
                            PieceType::BQ => board.b_q_bb &= ~capture_bb,
                            _ => panic("Invalid capture PieceType for WP"),
                        }

                        board.halfmove_clock = 0;
                        board.en_passent = None;

                        //King cannot be in check
                        if !is_attacked(&board, false, board.w_k_bb) {
                            children.push(Arc::new(Node::spawn(leaf, board)));
                        }
                    }
                }

                // Check for en-passent
                board = leaf.board;
                match board.en_passent {
                    Some(ep_bb) => {
                        for capture_bb in capture_bbs.iter() {
                            if capture_bb & ep_bb > 0 {
                                board.w_p_bb &= ~lsb_p_bb;
                                board.w_p_bb |= ep_bb;
                                board.b_p_bb &= ~(ep_bb >> 8);
                                board.en_passent = None;
                                board.halfmove_clock = 0;

                                //King cannot be in check
                                if !is_attacked(&board, false, board.w_k_bb) {
                                    children.push(Arc::new(Node::spawn(leaf, board)));
                                }
                            }
                        }
                    },
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
            // Reset board
            board = leaf.board;

            // Gets bitboard with only lsb set
            let lsb_p_bb = p_bb & (~p_bb + 1);

            // Promotion possibilities
            let promotions = if lsb_p_bb & rank_2_bb > 0 {
                &[PieceType::BN, PieceType::BB, PieceType::BR, PieceType::BQ]
            } else {
                &[PieceType::BP]
            };

            let one_ahead_bb = lsb_p_bb >> 8;
            let capture_bbs = if lsb_p_bb & a_file_bb > 0 {
                &[lsb_p_bb >> 7]
            } else if lsb_p_bb & h_file_bb > 0 {
                &[lsb_p_bb >> 9]
            } else {
                &[lsb_p_bb >> 7, lsb_p_bb >> 9]
            };
            for promotion in promotions.iter() {
                // Move ahead one square
                board = leaf.board;
                if one_ahead_bb & all_pieces == 0 {
                    board.b_p_bb &= ~lsb_p_bb;
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
                        children.push(Arc::new(Node::spawn(leaf, board)));
                    }
                }

                // Move ahead two squares
                board = leaf.board;
                if lsb_p_bb & rank_7_bb > 0 && one_ahead_bb & all_pieces == 0 && (lsb_p_bb >> 16) & all_pieces == 0 {
                    board.b_p_bb &= ~lsb_p_bb;
                    board.b_p_bb |= lsb_p_bb >> 16;
                    board.halfmove_clock = 0;
                    board.en_passent = None;

                    //King cannot be in check
                    if !is_attacked(&board, true, board.b_k_bb) {
                        children.push(Arc::new(Node::spawn(leaf, board)));
                    }
                }

                // Check for captures
                for capture_bb in capture_bbs.iter() {
                    let captured_piece = get_piecetype(board, capture_bb & b_pieces);
                    if captured_piece.is_some() {
                        board = leaf.board;
                        board.b_p_bb &= ~lsb_p_bb;
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
                            PieceType::WP => board.w_p_bb &= ~capture_bb,
                            PieceType::WN => board.w_n_bb &= ~capture_bb,
                            PieceType::WB => board.w_b_bb &= ~capture_bb,
                            PieceType::WR => board.w_r_bb &= ~capture_bb,
                            PieceType::WQ => board.w_q_bb &= ~capture_bb,
                            _ => panic("Invalid capture PieceType for BP"),
                        }

                        board.halfmove_clock = 0;
                        board.en_passent = None;

                        //King cannot be in check
                        if !is_attacked(&board, true, board.b_k_bb) {
                            children.push(Arc::new(Node::spawn(leaf, board)));
                        }
                    }
                }

                // Check for en-passent
                board = leaf.board;
                match board.en_passent {
                    Some(ep_bb) => {
                        for capture_bb in capture_bbs.iter() {
                            if capture_bb & ep_bb > 0 {
                                board.b_p_bb &= ~lsb_p_bb;
                                board.b_p_bb |= ep_bb;
                                board.w_p_bb &= ~(ep_bb << 8);
                                board.en_passent = None;
                                board.halfmove_clock = 0;

                                //King cannot be in check
                                if !is_attacked(&board, true, board.b_k_bb) {
                                    children.push(Arc::new(Node::spawn(leaf, board)));
                                }
                            }
                        }
                    },
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

    if leaf.is_w_move {
        let mut n_bb = board.w_n_bb;
        while n_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_n_bb = n_bb & (~n_bb + 1);

            let mut solo_n_moves = solo_knight_moves(lsb_n_bb, w_pieces);

            while solo_n_moves > 0 {
                // Reset board
                board = leaf.board;

                // Look at next knight move
                let lsb_solo_n_moves = solo_n_moves & (~solo_n_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_n_moves);
                match captured_pt {
                    Some(pt) => {
                        match pt {
                            PieceType::BP => board.b_p_bb &= ~lsb_solo_n_moves,
                            PieceType::BN => board.b_n_bb &= ~lsb_solo_n_moves,
                            PieceType::BB => board.b_b_bb &= ~lsb_solo_n_moves,
                            PieceType::BR => board.b_r_bb &= ~lsb_solo_n_moves,
                            PieceType::BQ => board.b_q_bb &= ~lsb_solo_n_moves,
                            _ => panic!(format!("Internal error: white knight cannot capture {}!", pt)),
                        }
                    },
                    None => (),
                }

                // Move the knight
                board.w_n_bb |= lsb_solo_n_moves;
                board.w_n_bb &= ~lsb_n_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, false, board.w_k_bb) {
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Remove LSB on bitboard
                solo_n_moves &= ~lsb_solo_n_moves;
            }

            // Remove LSB on bitboard
            n_bb &= ~lsb_n_bb;
        }
    } else {
        // We are black
        let mut n_bb = board.b_n_bb;
        while n_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_n_bb = n_bb & (~n_bb + 1);

            let mut solo_n_moves = solo_knight_moves(lsb_n_bb, b_pieces);

            while solo_n_moves > 0 {
                // Reset board
                board = leaf.board;

                // Look at next knight move
                let lsb_solo_n_moves = solo_n_moves & (~solo_n_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_n_moves);
                match captured_pt {
                    Some(pt) => {
                        match pt {
                            PieceType::WP => board.w_p_bb &= ~lsb_solo_n_moves,
                            PieceType::WN => board.w_n_bb &= ~lsb_solo_n_moves,
                            PieceType::WB => board.w_b_bb &= ~lsb_solo_n_moves,
                            PieceType::WR => board.w_r_bb &= ~lsb_solo_n_moves,
                            PieceType::WQ => board.w_q_bb &= ~lsb_solo_n_moves,
                            _ => panic!(format!("Internal error: black knight cannot capture {}!", pt)),
                        }
                    },
                    None => (),
                }

                // Move the knight
                board.b_n_bb |= lsb_solo_n_moves;
                board.b_n_bb &= ~lsb_n_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, true, board.b_k_bb) {
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Remove LSB on bitboard
                solo_n_moves &= ~lsb_solo_n_moves;
            }

            // Remove LSB on bitboard
            n_bb &= ~lsb_n_bb;
        }
    }

    children
}

fn gen_bishop_moves(leaf: &Arc<Node>) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.is_w_move {
        let mut b_bb = board.w_b_bb;
        while b_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_b_bb = b_bb & (~b_bb + 1);

            let mut solo_b_moves = solo_bishop_moves(lsb_b_bb, w_pieces, all_pieces);

            while solo_b_moves > 0 {
                // Reset board
                board = leaf.board;

                // Look at next bishop move
                let lsb_solo_b_moves = solo_b_moves & (~solo_b_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_b_moves);
                match captured_pt {
                    Some(pt) => {
                        match pt {
                            PieceType::BP => board.b_p_bb &= ~lsb_solo_b_moves,
                            PieceType::BN => board.b_n_bb &= ~lsb_solo_b_moves,
                            PieceType::BB => board.b_b_bb &= ~lsb_solo_b_moves,
                            PieceType::BR => board.b_r_bb &= ~lsb_solo_b_moves,
                            PieceType::BQ => board.b_q_bb &= ~lsb_solo_b_moves,
                            _ => panic!(format!("Internal error: white bishop cannot capture {}!", pt)),
                        }
                    },
                    None => (),
                }

                // Move the bishop 
                board.w_b_bb |= lsb_solo_b_moves;
                board.w_b_bb &= ~lsb_b_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, false, board.w_k_bb) {
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Remove LSB on bitboard
                solo_b_moves &= ~lsb_solo_b_moves;
            }

            // Remove LSB on bitboard
            b_bb &= ~lsb_b_bb;
        }
    } else {
        // We are black
        let mut b_bb = board.b_b_bb;
        while b_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_b_bb = b_bb & (~b_bb + 1);

            let mut solo_b_moves = solo_bishop_moves(lsb_b_bb, b_pieces, all_pieces);

            while solo_b_moves > 0 {
                // Reset board
                board = leaf.board;

                // Look at next knight move
                let lsb_solo_b_moves = solo_b_moves & (~solo_b_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_b_moves);
                match captured_pt {
                    Some(pt) => {
                        match pt {
                            PieceType::WP => board.w_p_bb &= ~lsb_solo_b_moves,
                            PieceType::WN => board.w_n_bb &= ~lsb_solo_b_moves,
                            PieceType::WB => board.w_b_bb &= ~lsb_solo_b_moves,
                            PieceType::WR => board.w_r_bb &= ~lsb_solo_b_moves,
                            PieceType::WQ => board.w_q_bb &= ~lsb_solo_b_moves,
                            _ => panic!(format!("Internal error: black bishop cannot capture {}!", pt)),
                        }
                    },
                    None => (),
                }

                // Move the knight
                board.b_b_bb |= lsb_solo_b_moves;
                board.b_b_bb &= ~lsb_b_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, true, board.b_k_bb) {
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Remove LSB on bitboard
                solo_b_moves &= ~lsb_solo_b_moves;
            }

            // Remove LSB on bitboard
            b_bb &= ~lsb_b_bb;
        }
    }

    children
}

fn gen_rook_moves(leaf: &Arc<Node>) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.is_w_move {
        let mut r_bb = board.w_r_bb;
        while r_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_r_bb = r_bb & (~r_bb + 1);

            let mut solo_r_moves = solo_rook_moves(lsb_r_bb, w_pieces, all_pieces);

            while solo_r_moves > 0 {
                // Reset board
                board = leaf.board;

                // Look at next rook move
                let lsb_solo_r_moves = solo_r_moves & (~solo_r_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_r_moves);
                match captured_pt {
                    Some(pt) => {
                        match pt {
                            PieceType::BP => board.b_p_bb &= ~lsb_solo_r_moves,
                            PieceType::BN => board.b_n_bb &= ~lsb_solo_r_moves,
                            PieceType::BB => board.b_b_bb &= ~lsb_solo_r_moves,
                            PieceType::BR => board.b_r_bb &= ~lsb_solo_r_moves,
                            PieceType::BQ => board.b_q_bb &= ~lsb_solo_r_moves,
                            _ => panic!(format!("Internal error: white rook cannot capture {}!", pt)),
                        }
                    },
                    None => (),
                }

                // Move the rook 
                board.w_r_bb |= lsb_solo_r_moves;
                board.w_r_bb &= ~lsb_r_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }
                if lsb_r_bb & 0x1 > 0 {
                    board.is_w_q_castle = false;
                }
                else if lsb_r_bb & 0x80 > 0 {
                    board.is_w_castle = false;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, false, board.w_k_bb) {
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Remove LSB on bitboard
                solo_r_moves &= ~lsb_solo_r_moves;
            }

            // Remove LSB on bitboard
            r_bb &= ~lsb_r_bb;
        }
    } else {
        // We are black
        let mut r_bb = board.b_r_bb;
        while r_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_r_bb = r_bb & (~r_bb + 1);

            let mut solo_r_moves = solo_rook_moves(lsb_r_bb, b_pieces, all_pieces);

            while solo_r_moves > 0 {
                // Reset board
                board = leaf.board;

                // Look at next rook move
                let lsb_solo_r_moves = solo_r_moves & (~solo_r_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_r_moves);
                match captured_pt {
                    Some(pt) => {
                        match pt {
                            PieceType::WP => board.w_p_bb &= ~lsb_solo_r_moves,
                            PieceType::WN => board.w_n_bb &= ~lsb_solo_r_moves,
                            PieceType::WB => board.w_b_bb &= ~lsb_solo_r_moves,
                            PieceType::WR => board.w_r_bb &= ~lsb_solo_r_moves,
                            PieceType::WQ => board.w_q_bb &= ~lsb_solo_r_moves,
                            _ => panic!(format!("Internal error: black rook cannot capture {}!", pt)),
                        }
                    },
                    None => (),
                }

                // Move the rook
                board.b_r_bb |= lsb_solo_r_moves;
                board.b_r_bb &= ~lsb_r_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }
                if lsb_r_bb & 0x100000000000000 > 0 {
                    board.is_b_q_castle = false;
                }
                else if lsb_r_bb & 0x8000000000000000 > 0 {
                    board.is_b_castle = false;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, true, board.b_k_bb) {
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Remove LSB on bitboard
                solo_r_moves &= ~lsb_solo_r_moves;
            }

            // Remove LSB on bitboard
            r_bb &= ~lsb_r_bb;
        }
    }

    children
}

fn gen_queen_moves(leaf: &Arc<Node>) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.is_w_move {
        let mut q_bb = board.w_q_bb;
        while q_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_q_bb = q_bb & (~q_bb + 1);

            let mut solo_q_moves = solo_rook_moves(lsb_q_bb, w_pieces, all_pieces) | solo_bishop_moves(lsb_q_bb, w_pieces, all_pieces);

            while solo_q_moves > 0 {
                // Reset board
                board = leaf.board;

                // Look at next queen move
                let lsb_solo_q_moves = solo_q_moves & (~solo_q_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_q_moves);
                match captured_pt {
                    Some(pt) => {
                        match pt {
                            PieceType::BP => board.b_p_bb &= ~lsb_solo_q_moves,
                            PieceType::BN => board.b_n_bb &= ~lsb_solo_q_moves,
                            PieceType::BB => board.b_b_bb &= ~lsb_solo_q_moves,
                            PieceType::BR => board.b_r_bb &= ~lsb_solo_q_moves,
                            PieceType::BQ => board.b_q_bb &= ~lsb_solo_q_moves,
                            _ => panic!(format!("Internal error: white queen cannot capture {}!", pt)),
                        }
                    },
                    None => (),
                }

                // Move the queen 
                board.w_q_bb |= lsb_solo_q_moves;
                board.w_q_bb &= ~lsb_q_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, false, board.w_k_bb) {
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Remove LSB on bitboard
                solo_q_moves &= ~lsb_solo_q_moves;
            }

            // Remove LSB on bitboard
            q_bb &= ~lsb_q_bb;
        }
    } else {
        // We are black
        let mut q_bb = board.b_q_bb;
        while q_bb > 0 {
            // Gets bitboard with only lsb set
            let lsb_q_bb = q_bb & (~q_bb + 1);

            let mut solo_q_moves = solo_rook_moves(lsb_q_bb, b_pieces, all_pieces) | solo_bishop_moves(lsb_q_bb, b_pieces, all_pieces);

            while solo_q_moves > 0 {
                // Reset board
                board = leaf.board;

                // Look at next queen move
                let lsb_solo_q_moves = solo_q_moves & (~solo_q_moves + 1);

                // Strip all enemies away from landing square
                let captured_pt = get_piecetype(&board, lsb_solo_q_moves);
                match captured_pt {
                    Some(pt) => {
                        match pt {
                            PieceType::WP => board.w_p_bb &= ~lsb_solo_q_moves,
                            PieceType::WN => board.w_n_bb &= ~lsb_solo_q_moves,
                            PieceType::WB => board.w_b_bb &= ~lsb_solo_q_moves,
                            PieceType::WR => board.w_r_bb &= ~lsb_solo_q_moves,
                            PieceType::WQ => board.w_q_bb &= ~lsb_solo_q_moves,
                            _ => panic!(format!("Internal error: black queen cannot capture {}!", pt)),
                        }
                    },
                    None => (),
                }

                // Move the queen
                board.b_q_bb |= lsb_solo_q_moves;
                board.b_q_bb &= ~lsb_q_bb;

                // Update other board fields
                board.en_passent = None;
                if captured_pt.is_some() {
                    board.halfmove_clock = 0;
                } else {
                    board.halfmove_clock += 1;
                }

                // If not in check then add to next moves
                if !is_attacked(&board, true, board.b_k_bb) {
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Remove LSB on bitboard
                solo_q_moves &= ~lsb_solo_q_moves;
            }

            // Remove LSB on bitboard
            q_bb &= ~lsb_q_bb;
        }
    }

    children
}

fn gen_king_moves(leaf: &Arc<Node>, w_pieces: u64, b_pieces: u64) -> Vec<Arc<Node>> {
    let all_pieces = w_pieces | b_pieces;
    let mut children = Vec::new();
    let mut board = leaf.board;

    if leaf.is_w_move {
        // Castling moves
        // Kingside
        if board.is_w_castle && w_pieces & 0x60 == 0 && !is_attacked(&board, false, board.w_k_bb) && !is_attacked(&board, false, 0x20) && !is_attacked(&board, false, 0x40) {
            // Move rook
            board.w_r_bb &= ~0x80;
            board.w_r_bb |= 0x20;

            // Move king
            board.w_k_bb = 0x40;

            // Other board changes
            board.is_w_castle = false;
            board.is_w_q_castle = false;
            board.en_passent = None;
            board.halfmove_clock += 1;

            children.push(Arc::new(Node::spawn(leaf, board)));
        }
        // Queenside
        board = leaf.board;
        if board.is_w_q_castle && w_pieces & 0xe == 0 && !is_attacked(&board, false, board.w_k_bb) && !is_attacked(&board, false, 0x4) && !is_attacked(&board, false, 0x8) {
            // Move rook
            board.w_r_bb &= ~0x1;
            board.w_r_bb |= 0x8;

            // Move king
            board.w_k_bb = 0x4;

            // Other board changes
            board.is_w_castle = false;
            board.is_w_q_castle = false;
            board.en_passent = None;
            board.halfmove_clock += 1;

            children.push(Arc::new(Node::spawn(leaf, board)));
        }

        // Standard moves
        board = leaf.board;
        let k_bb = board.w_k_bb;

        let mut solo_k_moves = solo_king_moves(k_bb, w_pieces);

        while solo_k_moves > 0 {
            // Reset board
            board = leaf.board;

            // Look at next king move
            let lsb_solo_k_moves = solo_k_moves & (~solo_k_moves + 1);

            // Strip all enemies away from landing square
            let captured_pt = get_piecetype(&board, lsb_solo_k_moves);
            match captured_pt {
                Some(pt) => {
                    match pt {
                        PieceType::BP => board.b_p_bb &= ~lsb_solo_k_moves,
                        PieceType::BN => board.b_n_bb &= ~lsb_solo_k_moves,
                        PieceType::BB => board.b_b_bb &= ~lsb_solo_k_moves,
                        PieceType::BR => board.b_r_bb &= ~lsb_solo_k_moves,
                        PieceType::BQ => board.b_q_bb &= ~lsb_solo_k_moves,
                        _ => panic!(format!("Internal error: white king cannot capture {}!", pt)),
                    }
                },
                None => (),
            }

            // Move the king
            board.w_k_bb |= lsb_solo_k_moves;
            board.w_k_bb &= k_bb;

            // Update other board fields
            board.en_passent = None;
            if captured_pt.is_some() {
                board.halfmove_clock = 0;
            } else {
                board.halfmove_clock += 1;
            }

            // If not in check then add to next moves
            if !is_attacked(&board, false, board.w_k_bb) {
                children.push(Arc::new(Node::spawn(leaf, board)));
            }

            // Remove LSB on bitboard
            solo_k_moves &= ~lsb_solo_k_moves;
        }
    } else {
        // We are black
        // Castling moves
        // Kingside
        if board.is_b_castle && b_pieces & 0x6000000000000000 == 0 && !is_attacked(&board, true, board.b_k_bb) && !is_attacked(&board, true, 0x2000000000000000) && !is_attacked(&board, true, 0x4000000000000000) {
            // Move rook
            board.b_r_bb &= ~0x8000000000000000;
            board.b_r_bb |= 0x2000000000000000;

            // Move king
            board.b_k_bb = 0x4000000000000000;

            // Other board changes
            board.is_b_castle = false;
            board.is_b_q_castle = false;
            board.en_passent = None;
            board.halfmove_clock += 1;

            children.push(Arc::new(Node::spawn(leaf, board)));
        }
        // Queenside
        board = leaf.board;
        if board.is_b_q_castle && b_pieces & 0xe00000000000000 == 0 && !is_attacked(&board, true, board.b_k_bb) && !is_attacked(&board, true, 0x400000000000000) && !is_attacked(&board, true, 0x800000000000000) {
            // Move rook
            board.b_r_bb &= ~0x100000000000000;
            board.b_r_bb |= 0x800000000000000;

            // Move king
            board.b_k_bb = 0x400000000000000;

            // Other board changes
            board.is_b_castle = false;
            board.is_b_q_castle = false;
            board.en_passent = None;
            board.halfmove_clock += 1;

            children.push(Arc::new(Node::spawn(leaf, board)));
        }

        // Standard moves
        board = leaf.board;
        let k_bb = board.b_k_bb;

        let mut solo_k_moves = solo_king_moves(k_bb, b_pieces);

        while solo_k_moves > 0 {
            // Reset board
            board = leaf.board;

            // Look at next king move
            let lsb_solo_k_moves = solo_k_moves & (~solo_k_moves + 1);

            // Strip all enemies away from landing square
            let captured_pt = get_piecetype(&board, lsb_solo_k_moves);
            match captured_pt {
                Some(pt) => {
                    match pt {
                        PieceType::WP => board.w_p_bb &= ~lsb_solo_k_moves,
                        PieceType::WN => board.w_n_bb &= ~lsb_solo_k_moves,
                        PieceType::WB => board.w_b_bb &= ~lsb_solo_k_moves,
                        PieceType::WR => board.w_r_bb &= ~lsb_solo_k_moves,
                        PieceType::WQ => board.w_q_bb &= ~lsb_solo_k_moves,
                        _ => panic!(format!("Internal error: black king cannot capture {}!", pt)),
                    }
                },
                None => (),
            }

            // Move the king
            board.b_k_bb |= lsb_solo_k_moves;
            board.b_k_bb &= k_bb;

            // Update other board fields
            board.en_passent = None;
            if captured_pt.is_some() {
                board.halfmove_clock = 0;
            } else {
                board.halfmove_clock += 1;
            }

            // If not in check then add to next moves
            if !is_attacked(&board, false, board.b_k_bb) {
                children.push(Arc::new(Node::spawn(leaf, board)));
            }

            // Remove LSB on bitboard
            solo_k_moves &= ~lsb_solo_k_moves;
        }
    }

    children
}

fn is_attacked(board: &Board, by_white: bool, bb: u64) -> bool {
    if !by_white {
        // We are white
        let ally_pieces = board.w_p_bb | board.w_n_bb | board.w_b_bb | board.w_r_bb | board.w_q_bb | board.w_k_bb;
        let enemy_pieces = board.b_p_bb | board.b_n_bb | board.b_b_bb | board.b_r_bb | board.b_q_bb | board.b_k_bb;
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
    }
    else {
        // We are black
        let ally_pieces = board.b_p_bb | board.b_n_bb | board.b_b_bb | board.b_r_bb | board.b_q_bb | board.b_k_bb;
        let enemy_pieces = board.w_p_bb | board.w_n_bb | board.w_b_bb | board.w_r_bb | board.w_q_bb | board.w_k_bb;
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
}

fn solo_knight_moves(bb: u64, ally_pieces: u64) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    magic::knight_collisions[pos] & ~ally_pieces
}

fn solo_bishop_moves(bb: u64, ally_pieces: u64, all_pieces: u64) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    let occupied_coll = magic::bishop_collisions[pos] & all_pieces;
    let magic_ind = Wrapping(magic::bishop_magic_numbers[pos]) * Wrapping(occupied_coll) >> 55;
    magic::bishop_magic_move_sets[pos][magic_ind.0 as usize] & ~ally_pieces
}

fn solo_rook_moves(bb: u64, ally_pieces: u64, all_pieces: u64) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    let occupied_coll = magic::rook_collisions[pos] & all_pieces;
    let magic_ind = Wrapping(magic::rook_magic_numbers[pos]) * Wrapping(occupied_coll) >> 52;
    magic::rook_magic_move_sets[pos][magic_ind.0 as usize] & ~ally_pieces
}

fn solo_king_moves(bb: u64, ally_pieces: u64) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    magic::king_collisions[pos] & ~ally_pieces
}

fn solo_pawn_attacks(bb: u64, enemy_pawns: u64, is_white: bool) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    if is_white {
        magic::w_pawn_attack_collisions[pos] & enemy_pieces
    } else {
        magic::b_pawn_attack_collisions[pos] & enemy_pieces
    }
}


#[cfg(test)]
mod tests {
    use super::*;


