use std::sync::{Arc, RwLockWriteGuard};

use crate::search::{self, Node};
use create::board::{self, Board, PieceType};

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
    let mut board = Node.board;

    if leaf.is_w_move {
        let mut p_bb = board.w_p_bb;
        while p_bb.count_ones() > 0 {
            // Reset board
            board = Node.board;

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
                board = Node.board;
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
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Move ahead two squares
                board = Node.board;
                if lsb_p_bb & rank_2_bb > 0 && one_ahead_bb & all_pieces == 0 && (lsb_p_bb << 16) & all_pieces == 0 {
                    board.w_p_bb &= ~lsb_p_bb;
                    board.w_p_bb |= lsb_p_bb << 16;
                    board.halfmove_clock = 0;
                    board.en_passent = None;
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Check for captures
                for capture_bb in capture_bbs.iter() {
                    let captured_piece = get_piecetype(board, capture_bb & b_pieces);
                    if captured_piece.is_some() {
                        board = Node.board;
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
                        children.push(Arc::new(Node::spawn(leaf, board)));
                    }
                }

                // Check for en-passent
                board = Node.board;
                match board.en_passent {
                    Some(ep_bb) => {
                        for capture_bb in capture_bbs.iter() {
                            if capture_bb & ep_bb > 0 {
                                board.w_p_bb &= ~lsb_p_bb;
                                board.w_p_bb |= ep_bb;
                                board.b_p_bb &= ~(ep_bb >> 8);
                                board.en_passent = None;
                                board.halfmove_clock = 0;
                                children.push(Arc::new(Node::spawn(leaf, board)));
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
            board = Node.board;

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
                board = Node.board;
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
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Move ahead two squares
                board = Node.board;
                if lsb_p_bb & rank_7_bb > 0 && one_ahead_bb & all_pieces == 0 && (lsb_p_bb >> 16) & all_pieces == 0 {
                    board.b_p_bb &= ~lsb_p_bb;
                    board.b_p_bb |= lsb_p_bb >> 16;
                    board.halfmove_clock = 0;
                    board.en_passent = None;
                    children.push(Arc::new(Node::spawn(leaf, board)));
                }

                // Check for captures
                for capture_bb in capture_bbs.iter() {
                    let captured_piece = get_piecetype(board, capture_bb & b_pieces);
                    if captured_piece.is_some() {
                        board = Node.board;
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
                        children.push(Arc::new(Node::spawn(leaf, board)));
                    }
                }

                // Check for en-passent
                board = Node.board;
                match board.en_passent {
                    Some(ep_bb) => {
                        for capture_bb in capture_bbs.iter() {
                            if capture_bb & ep_bb > 0 {
                                board.b_p_bb &= ~lsb_p_bb;
                                board.b_p_bb |= ep_bb;
                                board.w_p_bb &= ~(ep_bb << 8);
                                board.en_passent = None;
                                board.halfmove_clock = 0;
                                children.push(Arc::new(Node::spawn(leaf, board)));
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

fn gen_knight_moves(leaf: &Arc<Node>) -> Vec<Arc<Node>> {
    Vec::new()
}

fn gen_bishop_moves(leaf: &Arc<Node>) -> Vec<Arc<Node>> {
    Vec::new()
}

fn gen_rook_moves(leaf: &Arc<Node>) -> Vec<Arc<Node>> {
    Vec::new()
}

fn gen_queen_moves(leaf: &Arc<Node>) -> Vec<Arc<Node>> {
    Vec::new()
}

fn gen_king_moves(leaf: &Arc<Node>) -> Vec<Arc<Node>> {
    Vec::new()
}

fn is_attacked(board: Board, by_white: bool, bb: u64) -> bool {
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
        if solo_pawn_checks(bb, enemy_pieces, true) & board.b_p_bb > 0 {
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
        if solo_pawn_checks(bb, enemy_pieces, false) & board.w_p_bb > 0 {
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
    magic::bishop_magic_move_sets[pos][magic_ind.0] & ~ally_pieces
}

fn solo_rook_moves(bb: u64, ally_pieces: u64, all_pieces: u64) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    let occupied_coll = magic::rook_collisions[pos] & all_pieces;
    let magic_ind = Wrapping(magic::rook_magic_numbers[pos]) * Wrapping(occupied_coll) >> 52;
    magic::rook_magic_move_sets[pos][magic_ind.0] & ~ally_pieces
}

fn solo_king_moves(bb: u64, ally_pieces: u64) -> u64 {
    let pos = bb.trailing_zeros() as usize;
    magic::king_collisions[pos] & ~ally_pieces
}
