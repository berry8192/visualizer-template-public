#![allow(non_snake_case)]

use super::{compute_score, parse_input, parse_output};

pub fn score(input: String, output: String) -> (i64, String) {
    let input = parse_input(&input);
    let out = parse_output(&input, &output);
    let (score, err) = match out {
        Ok(out) => compute_score(&input, &out),
        Err(err) => (0, err),
    };
    (score, err)
}