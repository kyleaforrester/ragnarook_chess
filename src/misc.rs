use std::num::Wrapping;

pub fn eval_to_cp(eval: f32) -> i32 {
    if eval > 0.5 {
        ((20_000.0*eval - 10_000.0) / (1.0 - eval)).sqrt() as i32
    }
    else if eval < 0.5 {
        let inverse = 1.0 - eval;
        ((20_000.0*inverse - 10_000.0) / (1.0 - inverse)).sqrt() as i32
    }
    else {
        0
    }
}

pub fn eval_to_movestogo(eval: f32) -> u32 {
    50
}

//PRNG Algorithm
//Credit:
//https://nullprogram.com/blog/2017/09/21/
pub fn spcg32(mut state: &u64) -> (u32, u64) {
    let state = Wrapping(state.clone());
    let m = Wrapping(0x9b60933458e17d7du64);
    let a = Wrapping(0xd737232eeccdf7edu64);
    let new_state = state * m + a;
    let shift = 29 - (new_state.0 >> 61);
    ((new_state.0 >> shift) as u32, new_state.0)
}

