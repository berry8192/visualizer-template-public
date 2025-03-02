use wasm_bindgen::prelude::*;
mod tools;

use noise::{NoiseFn, Perlin};
use proconio::{input, marker::Chars};
use rand::prelude::*;
use std::ops::RangeBounds;

pub trait SetMinMax {
    fn setmin(&mut self, v: Self) -> bool;
    fn setmax(&mut self, v: Self) -> bool;
}
impl<T> SetMinMax for T
where
    T: PartialOrd,
{
    fn setmin(&mut self, v: T) -> bool {
        *self > v && {
            *self = v;
            true
        }
    }
    fn setmax(&mut self, v: T) -> bool {
        *self < v && {
            *self = v;
            true
        }
    }
}

#[derive(Clone, Debug)]
pub struct Input {
    N: usize,
    M: usize,
    cs: Vec<Vec<char>>,
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", self.N, self.M)?;
        for i in 0..self.N {
            writeln!(f, "{}", self.cs[i].iter().collect::<String>())?;
        }
        Ok(())
    }
}

pub fn parse_input(f: &str) -> Input {
    let f = proconio::source::once::OnceSource::from(f);
    input! {
        from f,
        N: usize, M: usize,
        cs: [Chars; N],
    }
    Input { N, M, cs }
}

pub fn read<T: Copy + PartialOrd + std::fmt::Display + std::str::FromStr, R: RangeBounds<T>>(
    token: Option<&str>,
    range: R,
) -> Result<T, String> {
    if let Some(v) = token {
        if let Ok(v) = v.parse::<T>() {
            if !range.contains(&v) {
                Err(format!("Out of range: {}", v))
            } else {
                Ok(v)
            }
        } else {
            Err(format!("Parse error: {}", v))
        }
    } else {
        Err("Unexpected EOF".to_owned())
    }
}

#[derive(Clone, Debug, Copy)]
pub enum Action {
    Move(usize),
    Carry(usize),
    Roll(usize),
}

const DIJ: [(usize, usize); 4] = [(!0, 0), (1, 0), (0, !0), (0, 1)];
const DIR: [char; 4] = ['U', 'D', 'L', 'R'];

pub struct Output {
    pub out: Vec<Action>,
}

pub fn parse_output(_input: &Input, f: &str) -> Result<Output, String> {
    let mut out = vec![];
    let mut ss = f.split_whitespace().peekable();
    while ss.peek().is_some() {
        let a = read(ss.next(), 1..=3)?;
        let dir = read(ss.next(), 'A'..='Z')?;
        let Some(d) = DIR.iter().position(|&x| x == dir) else {
            return Err(format!("Invalid direction: {}", dir));
        };
        out.push(match a {
            1 => Action::Move(d),
            2 => Action::Carry(d),
            3 => Action::Roll(d),
            _ => unreachable!(),
        });
    }
    if out.len() > 10000 {
        return Err("Too many actions".to_owned());
    }
    Ok(Output { out })
}

pub fn compute_score(input: &Input, out: &Output) -> (i64, String) {
    let (mut score, err, _) = compute_score_details(input, &out.out);
    if err.len() > 0 {
        score = 0;
    }
    (score, err)
}

pub fn compute_score_details(input: &Input, out: &[Action]) -> (i64, String, ()) {
    let mut cs = input.cs.clone();
    let mut pos = (0, 0);
    let mut K = 0;
    let mut A = 0;
    for i in 0..input.N {
        for j in 0..input.N {
            if cs[i][j] == 'A' {
                pos = (i, j);
            } else if cs[i][j] >= 'a' && cs[i][j] <= 'z' {
                K += 1;
            }
        }
    }
    for t in 0..out.len() {
        match out[t] {
            Action::Move(d) => {
                let (di, dj) = DIJ[d];
                pos.0 += di;
                pos.1 += dj;
                if pos.0 >= input.N || pos.1 >= input.N {
                    return (0, format!("Out of the board (turn {t})"), ());
                }
            }
            Action::Carry(d) => {
                let (di, dj) = DIJ[d];
                if (cs[pos.0][pos.1] < 'a' || cs[pos.0][pos.1] > 'z') && cs[pos.0][pos.1] != '@' {
                    return (0, format!("No item to carry (turn {t})"), ());
                }
                let c = cs[pos.0][pos.1];
                cs[pos.0][pos.1] = '.';
                pos.0 += di;
                pos.1 += dj;
                if pos.0 >= input.N || pos.1 >= input.N {
                    return (0, format!("Out of the board (turn {t})"), ());
                }
                if matches!(cs[pos.0][pos.1], '@' | 'a'..='z') {
                    return (0, format!("Collision (turn {t})"), ());
                } else if matches!(cs[pos.0][pos.1], 'A'..='Z') {
                    if cs[pos.0][pos.1].to_ascii_lowercase() == c {
                        A += 1;
                    }
                } else {
                    assert_eq!(cs[pos.0][pos.1], '.');
                    cs[pos.0][pos.1] = c;
                }
            }
            Action::Roll(d) => {
                let (di, dj) = DIJ[d];
                if (cs[pos.0][pos.1] < 'a' || cs[pos.0][pos.1] > 'z') && cs[pos.0][pos.1] != '@' {
                    return (0, format!("No item to roll (turn {t})"), ());
                }
                let c = cs[pos.0][pos.1];
                cs[pos.0][pos.1] = '.';
                let mut crt = pos;
                loop {
                    let next = (crt.0 + di, crt.1 + dj);
                    if next.0 >= input.N
                        || next.1 >= input.N
                        || matches!(cs[next.0][next.1], '@' | 'a'..='z')
                    {
                        cs[crt.0][crt.1] = c;
                        break;
                    } else if matches!(cs[next.0][next.1], 'A'..='Z') {
                        if cs[next.0][next.1].to_ascii_lowercase() == c {
                            A += 1;
                        }
                        break;
                    } else {
                        crt = next;
                    }
                }
            }
        }
    }
    let score = if A == K {
        (1e6 * (1.0 + (1e4 / out.len() as f64).log2())).round() as i64
    } else {
        (1e6 * A as f64 / K as f64).round() as i64
    };
    (score, String::new(), ())
}

#[wasm_bindgen]
pub fn gen(seed: i32, problemId: String) -> String {
    let input = tools::generate(seed as u64, &problemId);
    input.to_string()
}

#[wasm_bindgen(getter_with_clone)]
pub struct Ret {
    pub score: i64,
    pub err: String,
    pub svg: String,
}

#[wasm_bindgen]
pub fn vis(input: String, output: String, turn: usize) -> Ret {
    let (score, err) = tools::score::score(input, output);
    Ret {
        score,
        err,
        svg: "".to_string(),
    }
}

#[wasm_bindgen]
pub fn get_max_turn(_input: String, _output: String) -> usize {
    10000
}
