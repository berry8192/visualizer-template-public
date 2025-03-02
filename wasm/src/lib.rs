use wasm_bindgen::prelude::*;
mod tools;

use svg::node::element::{Rectangle, Circle, Text, SVG};
use svg::Document;
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

fn get_color(c: char) -> &'static str {
    match c {
        '@' => "#000000", // 黒
        '.' => "#FFFFFF", // 白
        'A' => "#FF0000",
        'a' => "#FFFFFF",
        'B' => "#00FF00",
        'b' => "#FFFFFF",
        'C' => "#0000FF",
        'c' => "#FFFFFF",
        _ => "#888888",   // デフォルト灰色
    }
}

/// 小文字のキャラに対応する丸の色を取得
fn get_circle_color(c: char) -> &'static str {
    match c {
        'a' => "#FF8888", // 薄い赤
        'b' => "#88FF88", // 薄い緑
        'c' => "#8888FF", // 薄い青
        _ => "#000000",   // 通常は黒
    }
}

/// 20×20 の盤面を表す SVG を生成
fn generate_svg(cs: Vec<Vec<char>>, x: usize, y: usize) -> String {
    let cell_size = 20; // 各マスのサイズ
    let circle_radius = 8;
    let mut document = Document::new()
        .set("viewBox", (0, 0, 400, 400))
        .set("width", "400px")  // 最大幅
        .set("height", "400px") // 最大高さ
        .set("preserveAspectRatio", "xMidYMid meet"); // 縮小時に中央寄せ

    for (row_idx, row) in cs.iter().enumerate() {
        for (col_idx, &c) in row.iter().enumerate() {
            let x_pos = col_idx as i32 * cell_size;
            let y_pos = row_idx as i32 * cell_size;

            // マスの背景を描画
            let rect = Rectangle::new()
                .set("x", x_pos)
                .set("y", y_pos)
                .set("width", cell_size)
                .set("height", cell_size)
                .set("fill", get_color(c))
                .set("stroke", "#000")  // 枠線を黒に
                .set("stroke-width", 1);

            document = document.add(rect);

            // 小文字 (a, b, c) の場合、それぞれの色の小さな丸を描画
            if let Some(circle_color) = match c {
                'a' | 'b' | 'c' => Some(get_circle_color(c)),
                _ => None,
            } {
                let circle = Circle::new()
                    .set("cx", x_pos + cell_size / 2)
                    .set("cy", y_pos + cell_size / 2)
                    .set("r", circle_radius)
                    .set("fill", circle_color); // 各文字の対応する色
                document = document.add(circle);
            }
        }
    }

    // 主人公の位置にキャラを描画 (例: 黄色い円)
    let player_circle = Circle::new()
        .set("cx", x as i32 * cell_size + cell_size / 2)
        .set("cy", y as i32 * cell_size + cell_size / 2)
        .set("r", 6)
        .set("fill", "yellow") // 主人公を黄色の円で表す
        .set("stroke", "black")
        .set("stroke-width", 2);

    document = document.add(player_circle);

    document.to_string() // SVG を文字列として返す
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
    let input = tools::parse_input(&input);
    let output = tools::parse_output(&input, &output);
    let (score, err, svg) = match output {
        Ok(out) => {
            let actions = &out.out[..turn];
            let (score, err) = tools::compute_score(&input, &out);
            let (cs, (x, y)) = tools::get_grid(&input, &actions);
            (score, err, generate_svg(cs, y, x))
        }
        Err(err) => {
            let (cs, (x, y)) = tools::get_grid(&input, &[]);
            (0, err, generate_svg(cs, y, x))
        }
    };

    Ret { score, err, svg }
}

#[wasm_bindgen]
pub fn get_max_turn(input: String, output: String) -> usize {
    let input = tools::parse_input(&input);
    let output = tools::parse_output(&input, &output);
    match output {
        Ok(out) => out.out.len(),
        Err(_) => 0,
    }
}
