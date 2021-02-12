use crate::board;
use crate::misc;
use crate::move_gen;
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
    let w_mobility = mobility(board, true);

    let b_pawn_score = pawn_score(false, board.b_p_bb);
    let b_mobility = mobility(board, false);

    let w_score = w_pawn_score + w_mobility;
    let b_score = b_pawn_score + b_mobility;
    let cp_eval = (w_material as f32) + w_score - (b_material as f32) - b_score;

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

        let rank = if is_w { pos / 8 } else { 7 - (pos / 8) };

        score += (150.0 / 48.0) * (rank.pow(2) as f32) - (150.0 / 48.0);

        p_bb = p_bb & (p_bb - 1);
    }
    score
}

fn mobility(board: &board::Board, is_white: bool) -> f32 {
    let n_mult = 10.0;
    let b_mult = 10.0;
    let r_mult = 10.0;
    let q_mult = 5.0;
    let mut score = 0.0;

    if is_white {
        let allies =
            board.w_p_bb | board.w_n_bb | board.w_b_bb | board.w_r_bb | board.w_q_bb | board.w_k_bb;
        let enemies =
            board.b_p_bb | board.b_n_bb | board.b_b_bb | board.b_r_bb | board.b_q_bb | board.b_k_bb;

        // Knight mobility
        let mut bb = board.w_n_bb;
        while bb > 0 {
            let lsb = bb & (!bb + 1);
            score += n_mult * (move_gen::solo_knight_moves(bb, allies).count_ones() as f32);
            bb = bb & (bb - 1);
        }

        // Bishop mobility
        let mut bb = board.w_b_bb;
        while bb > 0 {
            let lsb = bb & (!bb + 1);
            score += b_mult
                * (move_gen::solo_bishop_moves(bb, allies, allies | enemies).count_ones() as f32);
            bb = bb & (bb - 1);
        }

        // Rook mobility
        let mut bb = board.w_r_bb;
        while bb > 0 {
            let lsb = bb & (!bb + 1);
            score += r_mult
                * (move_gen::solo_rook_moves(bb, allies, allies | enemies).count_ones() as f32);
            bb = bb & (bb - 1);
        }

        // Queen mobility
        let mut bb = board.w_q_bb;
        while bb > 0 {
            let lsb = bb & (!bb + 1);
            score += q_mult
                * ((move_gen::solo_bishop_moves(bb, allies, allies | enemies).count_ones()
                    + move_gen::solo_rook_moves(bb, allies, allies | enemies).count_ones())
                    as f32);
            bb = bb & (bb - 1);
        }
    } else {
        let allies =
            board.b_p_bb | board.b_n_bb | board.b_b_bb | board.b_r_bb | board.b_q_bb | board.b_k_bb;
        let enemies =
            board.w_p_bb | board.w_n_bb | board.w_b_bb | board.w_r_bb | board.w_q_bb | board.w_k_bb;

        // Knight mobility
        let mut bb = board.b_n_bb;
        while bb > 0 {
            let lsb = bb & (!bb + 1);
            score += n_mult * (move_gen::solo_knight_moves(bb, allies).count_ones() as f32);
            bb = bb & (bb - 1);
        }

        // Bishop mobility
        let mut bb = board.b_b_bb;
        while bb > 0 {
            let lsb = bb & (!bb + 1);
            score += b_mult
                * (move_gen::solo_bishop_moves(bb, allies, allies | enemies).count_ones() as f32);
            bb = bb & (bb - 1);
        }

        // Rook mobility
        let mut bb = board.b_r_bb;
        while bb > 0 {
            let lsb = bb & (!bb + 1);
            score += r_mult
                * (move_gen::solo_rook_moves(bb, allies, allies | enemies).count_ones() as f32);
            bb = bb & (bb - 1);
        }

        // Queen mobility
        let mut bb = board.b_q_bb;
        while bb > 0 {
            let lsb = bb & (!bb + 1);
            score += q_mult
                * ((move_gen::solo_bishop_moves(bb, allies, allies | enemies).count_ones()
                    + move_gen::solo_rook_moves(bb, allies, allies | enemies).count_ones())
                    as f32);
            bb = bb & (bb - 1);
        }
    }
    score
}
