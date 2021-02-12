use std::num::Wrapping;

pub fn eval_to_cp(eval: f32) -> i32 {
    if eval > 0.5 {
        ((20_000.0 * eval - 10_000.0) / (1.0 - eval)).sqrt() as i32
    } else if eval < 0.5 {
        let inverse = 1.0 - eval;
        -((20_000.0 * inverse - 10_000.0) / (1.0 - inverse)).sqrt() as i32
    } else {
        0
    }
}

pub fn cp_to_eval(cp: i32) -> f32 {
    if cp > 0 {
        ((cp.pow(2) + 10_000) as f32) / ((cp.pow(2) + 20_000) as f32)
    } else if cp < 0 {
        let inverse = cp * -1;
        let eval = ((inverse.pow(2) + 10_000) as f32) / ((inverse.pow(2) + 20_000) as f32);
        1.0 - eval
    } else {
        0.5
    }
}

pub fn eval_to_movestogo(_eval: f32) -> u32 {
    50
}

//PRNG Algorithm
//Credit:
//https://nullprogram.com/blog/2017/09/21/
pub fn spcg32(state: &u64) -> (u32, u64) {
    let state = Wrapping(state.clone());
    let m = Wrapping(0x9b60933458e17d7du64);
    let a = Wrapping(0xd737232eeccdf7edu64);
    let new_state = state * m + a;
    let shift = 29 - (new_state.0 >> 61);
    ((new_state.0 >> shift) as u32, new_state.0)
}
