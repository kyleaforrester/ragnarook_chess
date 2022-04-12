use crate::board;
use crate::misc;
use crate::move_gen;
use crate::search::Ending;
use std::cmp;

const initiative: i32 = 30;

const mg_pawn_table: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20, -14,
    13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3, 33, -12,
    -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
];
const eg_pawn_table: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 178, 173, 158, 134, 147, 132, 165, 187, 94, 100, 85, 67, 56, 53, 82,
    84, 32, 24, 13, 5, -2, 4, 17, 17, 13, 9, -3, -7, -7, -8, 3, -1, 4, 7, -6, 1, 0, -5, -1, -8, 13,
    8, 8, 10, 13, 0, 2, -7, 0, 0, 0, 0, 0, 0, 0, 0,
];

const mg_knight_table: [i32; 64] = [
    -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65, 84,
    129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9, 12, 10,
    19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17, -28, -19, -23,
];
const eg_knight_table: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20, 10, 9,
    -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4, -18, -23, -3, -1,
    15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51, -23, -15, -22, -18, -50,
    -64,
];

const mg_bishop_table: [i32; 64] = [
    -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35, 50,
    37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14, 27, 18,
    10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
];
const eg_bishop_table: [i32; 64] = [
    -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1, -2, 6, 0,
    4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10, 13, 3, -7, -15,
    -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
];

const mg_rook_table: [i32; 64] = [
    32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61, 16,
    -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17, 3, 0,
    -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26,
];
const eg_rook_table: [i32; 64] = [
    13, 10, 18, 15, 12, 12, 8, 5, 11, 13, 13, 11, -3, 3, 8, 3, 7, 7, 7, 5, 4, -3, -5, -3, 4, 3, 13,
    1, 2, 1, -1, 2, 3, 5, 8, 4, -5, -6, -8, -11, -4, 0, -5, -1, -7, -12, -8, -16, -6, -6, 0, 2, -9,
    -9, -11, -3, -9, 2, 3, -1, -5, -13, 4, -20,
];

const mg_queen_table: [i32; 64] = [
    -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56, 47,
    57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11, -2, -5, 2,
    14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
];
const eg_queen_table: [i32; 64] = [
    -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35, 19, 9,
    3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6, 9, 17, 10, 5,
    -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20, -41,
];

const mg_king_table: [i32; 64] = [
    -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20, 6,
    22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51, -14,
    -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8, -28, 24,
    14,
];
const eg_king_table: [i32; 64] = [
    -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20, 45,
    44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11, 21, 23,
    16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
];

const MG_P_VAL: u32 = 82;
const EG_P_VAL: u32 = 94;
const MG_N_VAL: u32 = 337;
const EG_N_VAL: u32 = 281;
const MG_B_VAL: u32 = 365;
const EG_B_VAL: u32 = 297;
const MG_R_VAL: u32 = 477;
const EG_R_VAL: u32 = 512;
const MG_Q_VAL: u32 = 1025;
const EG_Q_VAL: u32 = 936;

pub fn evaluate(board: &board::Board) -> (Option<Ending>, f32) {
    let end = eval_ending(board);

    let w_mg_mat = (board.w_p_bb.count_ones() * MG_P_VAL
        + board.w_n_bb.count_ones() * MG_N_VAL
        + board.w_b_bb.count_ones() * MG_B_VAL
        + board.w_r_bb.count_ones() * MG_R_VAL
        + board.w_q_bb.count_ones() * MG_Q_VAL) as i32;
    let w_eg_mat = (board.w_p_bb.count_ones() * EG_P_VAL
        + board.w_n_bb.count_ones() * EG_N_VAL
        + board.w_b_bb.count_ones() * EG_B_VAL
        + board.w_r_bb.count_ones() * EG_R_VAL
        + board.w_q_bb.count_ones() * EG_Q_VAL) as i32;
    let b_mg_mat = (board.b_p_bb.count_ones() * MG_P_VAL
        + board.b_n_bb.count_ones() * MG_N_VAL
        + board.b_b_bb.count_ones() * MG_B_VAL
        + board.b_r_bb.count_ones() * MG_R_VAL
        + board.b_q_bb.count_ones() * MG_Q_VAL) as i32;
    let b_eg_mat = (board.b_p_bb.count_ones() * EG_P_VAL
        + board.b_n_bb.count_ones() * EG_N_VAL
        + board.b_b_bb.count_ones() * EG_B_VAL
        + board.b_r_bb.count_ones() * EG_R_VAL
        + board.b_q_bb.count_ones() * EG_Q_VAL) as i32;

    let w_mg_pesto = pesto_score(board, true, true);
    let w_eg_pesto = pesto_score(board, true, false);
    let b_mg_pesto = pesto_score(board, false, true);
    let b_eg_pesto = pesto_score(board, false, false);

    let phase: i32 = cmp::min(
        (board.w_n_bb.count_ones()
            + board.b_n_bb.count_ones()
            + board.w_b_bb.count_ones()
            + board.b_b_bb.count_ones()
            + board.w_r_bb.count_ones() * 2
            + board.b_r_bb.count_ones() * 2
            + board.w_q_bb.count_ones() * 4
            + board.b_q_bb.count_ones() * 4) as i32,
        24,
    );

    let w_mg = w_mg_mat + w_mg_pesto;
    let w_eg = w_eg_mat + w_eg_pesto;
    let b_mg = b_mg_mat + b_mg_pesto;
    let b_eg = b_eg_mat + b_eg_pesto;

    let mut eval = (phase * (w_mg - b_mg) + (24 - phase) * (w_eg - b_eg)) / 24;

    // Add the initiative
    eval += if board.is_w_move {
        initiative
    } else {
        -initiative
    };

    /*
    println!("w_mg_mat: {}", w_mg_mat);
    println!("w_eg_mat: {}", w_eg_mat);
    println!("w_mg_pesto: {}", w_mg_pesto);
    println!("w_eg_pesto: {}", w_eg_pesto);
    println!("b_mg_mat: {}", b_mg_mat);
    println!("b_eg_mat: {}", b_eg_mat);
    println!("b_mg_pesto: {}", b_mg_pesto);
    println!("b_eg_pesto: {}", b_eg_pesto);

    println!("initiative: {}", initiative);
    println!("phase: {}", phase);
    println!("eval: {}", eval);
    */
    //println!("cp_eval: {}", eval);
    //println!("cp_eval_2: {}", misc::eval_to_cp(misc::cp_to_eval(eval)));

    (end, misc::cp_to_eval(eval))
}

fn eval_ending(board: &board::Board) -> Option<Ending> {
    // Check drawing conditions
    if board.halfmove_clock >= 50 {
        Some(Ending::Draw)
    } else {
        None
    }
}

fn pesto_score(board: &board::Board, is_w_move: bool, is_mg_phase: bool) -> i32 {
    let bbs = if is_w_move {
        [
            board.w_p_bb,
            board.w_n_bb,
            board.w_b_bb,
            board.w_r_bb,
            board.w_q_bb,
            board.w_k_bb,
        ]
    } else {
        [
            board.b_p_bb,
            board.b_n_bb,
            board.b_b_bb,
            board.b_r_bb,
            board.b_q_bb,
            board.b_k_bb,
        ]
    };

    let pestos = if is_mg_phase {
        [
            mg_pawn_table,
            mg_knight_table,
            mg_bishop_table,
            mg_rook_table,
            mg_queen_table,
            mg_king_table,
        ]
    } else {
        [
            eg_pawn_table,
            eg_knight_table,
            eg_bishop_table,
            eg_rook_table,
            eg_queen_table,
            eg_king_table,
        ]
    };

    let mut score = 0;
    for i in 0..6 {
        let mut bb = bbs[i];
        while bb > 0 {
            let lsb = bb & (!bb + 1);
            let pos = lsb.trailing_zeros() as usize;

            let idx = if is_w_move {
                let row = 7 - (pos / 8);
                let col = pos % 8;
                row * 8 + col
            } else {
                let row = pos / 8;
                let col = pos % 8;
                row * 8 + col
            };

            score += pestos[i][idx];

            //println!("is_w_move: {}, is_mg_phase: {}, i: {}, idx: {}, score: {}", is_w_move, is_mg_phase, i, idx, pestos[i][idx]);

            // Strip the least significant bit
            bb &= bb - 1;
        }
    }
    score
}
