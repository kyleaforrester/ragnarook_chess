use crate::board;
use crate::misc;
use crate::search::Ending;

pub fn evaluate(board: &board::Board) -> (Option<Ending>, f32) {
    let end = eval_ending(board);

    let w_material = 100 * board.w_p_bb.count_ones()
        + 300 * board.w_n_bb.count_ones()
        + 300 * board.w_b_bb.count_ones()
        + 500 * board.w_r_bb.count_ones()
        + 900 * board.w_q_bb.count_ones();
    let b_material = 100 * board.b_p_bb.count_ones()
        + 300 * board.b_n_bb.count_ones()
        + 300 * board.b_b_bb.count_ones()
        + 500 * board.b_r_bb.count_ones()
        + 900 * board.b_q_bb.count_ones();

    let w_pawn_score = pawn_score(true, board.w_p_bb);
    let b_pawn_score = pawn_score(false, board.b_p_bb);
    let cp_eval = w_material as f32 + w_pawn_score - b_material as f32 - b_pawn_score;

    (end, misc::cp_to_eval(cp_eval as i32))
}

fn eval_ending(board: &board::Board) -> Option<Ending> {
    // Check drawing conditions
    if board.halfmove_clock >= 50 {
        Some(Ending::Draw)
    } else {
        None
    }
}

fn pawn_score(is_w: bool, bb: u64) -> f32 {
    let mut p_bb = bb;
    let mut score = 0.0;
    while p_bb > 0 {
        let lsb = p_bb & (!p_bb + 1);
        let pos = lsb.trailing_zeros();

        let rank = if is_w { pos / 8 } else { 8 - (pos / 8) };

        score += (300.0 / 48.0) * (rank.pow(2) as f32) - (300.0 / 48.0);

        p_bb = p_bb & (p_bb - 1);
    }
    score
}
