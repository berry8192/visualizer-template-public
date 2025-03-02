#![allow(non_snake_case, unused_macros)]

use noise::{NoiseFn, Perlin};
use proconio::{input, marker::Chars};
use rand::prelude::*;
use std::ops::RangeBounds;
pub mod score;

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

#[macro_export]
macro_rules! mat {
	($($e:expr),*) => { Vec::from(vec![$($e),*]) };
	($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
	($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
	($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
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

pub fn generate(seed: u64, problem: &str) -> Input {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
    match problem {
        "A" => {
            let N = 20;
            let M = 1;
            let mut cs = mat!['.'; N; N];
            let mut ps = vec![];
            for i in 0..N {
                for j in 0..N {
                    ps.push((i, j));
                }
            }
            ps.shuffle(&mut rng);
            let (i, j) = ps.pop().unwrap();
            cs[i][j] = 'A';
            for _ in 0..2 * N {
                let (i, j) = ps.pop().unwrap();
                cs[i][j] = 'a';
            }
            for _ in 0..2 * N {
                let (i, j) = ps.pop().unwrap();
                cs[i][j] = '@';
            }
            Input { N, M, cs }
        }
        "B" => {
            let N = 20;
            let M = 3;
            loop {
                let mut cs = mat!['.'; N; N];
                let mut ps = vec![];
                for i in 0..N {
                    for j in 0..N {
                        ps.push((i, j));
                    }
                }
                ps.shuffle(&mut rng);
                let mut ss = vec![];
                for k in 0..3 {
                    let (i, j) = ps.pop().unwrap();
                    cs[i][j] = (b'A' + k as u8) as char;
                    ss.push((i, j));
                }
                for k in 0..3 {
                    for _ in 0..N {
                        let (i, j) = ps.pop().unwrap();
                        cs[i][j] = (b'a' + k as u8) as char;
                    }
                }
                let mut ok = true;
                for k in 0..3 {
                    let t = (b'a' + k as u8) as char;
                    let s = ss[k];
                    let mut visited = mat![false; N; N];
                    let mut stack = vec![s];
                    visited[s.0][s.1] = true;
                    while let Some((i, j)) = stack.pop() {
                        for d in 0..4 {
                            let (di, dj) = DIJ[d];
                            let i = i + di;
                            let j = j + dj;
                            if i < N
                                && j < N
                                && !visited[i][j]
                                && (cs[i][j] == '.' || cs[i][j] == t)
                            {
                                visited[i][j] = true;
                                stack.push((i, j));
                            }
                        }
                    }
                    for i in 0..N {
                        for j in 0..N {
                            if cs[i][j] == t && !visited[i][j] {
                                ok = false;
                            }
                        }
                    }
                }
                if ok {
                    return Input { N, M, cs };
                }
            }
        }
        "C" => {
            let N = 20;
            let M = 1;
            let perlin = Perlin::new(rng.r#gen());
            let D = 10.0;
            let mut ps = vec![];
            for i in 0..N {
                for j in 0..N {
                    let x = i as f64 / D;
                    let y = j as f64 / D;
                    ps.push((perlin.get([x, y]), i, j));
                }
            }
            ps.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let mut cs = mat!['.'; N; N];
            for _ in 0..N * N / 2 {
                let (_, i, j) = ps.pop().unwrap();
                cs[i][j] = '@';
            }
            ps.shuffle(&mut rng);
            let (_, i, j) = ps.pop().unwrap();
            cs[i][j] = 'A';
            for _ in 0..2 * N {
                let (_, i, j) = ps.pop().unwrap();
                cs[i][j] = 'a';
            }
            Input { N, M, cs }
        }
        _ => {
            panic!("Unknown problem: {}", problem);
        }
    }
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
