use wasm_bindgen::prelude::*;
use anyhow::{anyhow, bail, Context, Result};
use rand::prelude::Distribution;
use rand::Rng;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use rand_distr::Normal;
use std::{fmt, str};
use svg::node::element::SVG;

mod lib_vis;
use lib_vis::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use once_cell::sync::Lazy;

#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

const N: usize = 50;
const M_LB: usize = 50;
const M_UB: usize = 1600;
const K_LB: i64 = 11000;
const K_UB: i64 = 20000;
const T: usize = 800;
const STATION_COST: i64 = 5000;
const RAIL_COST: i64 = 100;

fn read<T: Copy + PartialOrd + std::fmt::Display + std::str::FromStr>(
    token: Option<&str>,
    lb: T,
    ub: T,
) -> Result<T> {
    if let Some(v) = token {
        if let Ok(v) = v.parse::<T>() {
            if v < lb || ub < v {
                bail!("Out of range: {}", v);
            } else {
                Ok(v)
            }
        } else {
            bail!("Parse error: {}", v);
        }
    } else {
        bail!("Unexpected EOF");
    }
}

#[derive(Clone)]
pub struct Input {
    n: usize,
    m: usize,
    k: i64,
    t: usize,
    src: Vec<(usize, usize)>,
    dst: Vec<(usize, usize)>,
}

impl Input {
    pub fn new() -> Self {
        Input {
            n: 0,
            m: 0,
            k: 0,
            t: 0,
            src: vec![],
            dst: vec![],
        }
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} {} {} {}", self.n, self.m, self.k, self.t)?;
        for i in 0..self.m {
            writeln!(
                f,
                "{} {} {} {}",
                self.src[i].0, self.src[i].1, self.dst[i].0, self.dst[i].1
            )?;
        }
        Ok(())
    }
}

fn parse_input(s: &str) -> Result<Input> {
    let mut tokens = s.split_whitespace();

    let n = read(tokens.next(), N, N).context("N")?;
    let m = read(tokens.next(), M_LB, M_UB).context("M")?;
    let k = read(tokens.next(), K_LB, K_UB).context("K")?;
    let t = read(tokens.next(), T, T).context("T")?;

    let mut src = vec![];
    let mut dst = vec![];
    for c in 0..m {
        let r_src = read(tokens.next(), 0, N - 1).context(format!("home: i_{},s", c))?;
        let c_src = read(tokens.next(), 0, N - 1).context(format!("home: j_{},s", c))?;
        let r_dst = read(tokens.next(), 0, N - 1).context(format!("workplace: i_{},t", c))?;
        let c_dst = read(tokens.next(), 0, N - 1).context(format!("workplace: j_{},t", c))?;
        src.push((r_src, c_src));
        dst.push((r_dst, c_dst));
    }

    Ok(Input {
        n,
        m,
        k,
        t,
        src,
        dst,
    })
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    Left,
    Right,
    Up,
    Down,
}

fn kind_v_to_usize(kind_v: &Vec<Kind>) -> usize {
    let mut dir = [false; 4]; // [Left, Right, Up, Down]
    if kind_v.contains(&Kind::Left) {
        dir[0] = true;
    }
    if kind_v.contains(&Kind::Right) {
        dir[1] = true;
    }
    if kind_v.contains(&Kind::Up) {
        dir[2] = true;
    }
    if kind_v.contains(&Kind::Down) {
        dir[3] = true;
    }
    match dir {
        [false, false, false, false] => usize::MAX,
        [true, true, true, true] => 0,
        [true, true, false, false] => 1,
        [false, false, true, true] => 2,
        [true, false, false, true] => 3,
        [true, false, true, false] => 4,
        [false, true, true, false] => 5,
        [false, true, false, true] => 6,
        _ => unreachable!(),
    }
}

#[derive(Clone)]
pub struct Rail {
    // 1: Left, Right
    // 2: Up, Down
    // 3: Left, Down
    // 4: Left, Up
    // 5: Right, Up
    // 6: Right, Down
    pub kind_v: Vec<Kind>,
    pub r: usize,
    pub c: usize,
}

impl Rail {
    fn new(kind: usize, r: usize, c: usize) -> Self {
        let kind_v = match kind {
            1 => vec![Kind::Left, Kind::Right],
            2 => vec![Kind::Up, Kind::Down],
            3 => vec![Kind::Left, Kind::Down],
            4 => vec![Kind::Left, Kind::Up],
            5 => vec![Kind::Right, Kind::Up],
            6 => vec![Kind::Right, Kind::Down],
            _ => unreachable!(),
        };
        Rail { kind_v, r, c }
    }
}

#[derive(Clone)]
pub struct Station {
    pub kind_v: Vec<Kind>,
    pub r: usize,
    pub c: usize,
}

impl Station {
    fn new(r: usize, c: usize) -> Self {
        Station {
            kind_v: vec![Kind::Left, Kind::Right, Kind::Up, Kind::Down],
            r,
            c,
        }
    }
}

#[derive(Clone)]
enum Op {
    None,
    Rail(Rail),
    Station(Station),
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::None => writeln!(f, "-1")?,
            Op::Rail(r) => writeln!(f, "{} {} {}", kind_v_to_usize(&r.kind_v), r.r, r.c)?,
            Op::Station(t) => writeln!(f, "0 {} {}", t.r, t.c)?,
        }
        Ok(())
    }
}

#[derive(Clone)]
struct CommentedOp {
    op: Op,
    #[allow(dead_code)]
    comments: Vec<String>,
}

#[derive(Clone)]
struct Output {
    commented_ops: Vec<CommentedOp>,
}

fn parse_output(s: &str) -> Result<Output> {
    let mut commented_ops = vec![];
    let mut comments = vec![];

    for (i, line) in s.trim().lines().enumerate() {
        let line = line.trim();

        if !line.is_empty() {
            if line.starts_with("#") {
                let comment = line.strip_prefix("#").unwrap().trim().to_string();
                comments.push(comment);
            } else {
                let op = parse_op(line).context(format!("{}-th line", i + 1))?;
                commented_ops.push(CommentedOp {
                    op,
                    comments: comments.clone(),
                });
                comments.clear();
            }
        }
    }

    Ok(Output { commented_ops })
}

fn parse_op(line: &str) -> Result<Op> {
    let mut tokens = line.split_whitespace();
    let kind: i64 = read(tokens.next(), -1, 6).context("operation type")?;

    let op = match kind {
        -1 => Op::None,
        0 => {
            let r = read(tokens.next(), 0, N - 1).context("i")?;
            let c = read(tokens.next(), 0, N - 1).context("j")?;
            Op::Station(Station::new(r, c))
        }
        1 | 2 | 3 | 4 | 5 | 6 => {
            let r = read(tokens.next(), 0, N - 1).context("i")?;
            let c = read(tokens.next(), 0, N - 1).context("j")?;
            Op::Rail(Rail::new(kind as usize, r, c))
        }
        _ => Err(anyhow!("Invalid value: {}", kind).context("operation type"))?,
    };

    if tokens.next().is_some() {
        bail!("Too many input");
    }

    Ok(op)
}

#[derive(Clone)]
struct State {
    money: i64,
    income: i64,
    action: String,
    op_cnt: usize,
    uf: UnionFind,
    grid_state: Vec<Vec<Vec<Kind>>>,
}

impl State {
    fn new(input: &Input) -> State {
        State {
            money: input.k,
            income: 0,
            action: "".to_string(),
            op_cnt: 0,
            uf: UnionFind::new((input.n * input.n) as usize),
            grid_state: vec![vec![vec![]; input.n as usize]; input.n as usize],
        }
    }

    fn rail(&mut self, input: &Input, rail: Rail) -> Result<()> {
        if self.money < RAIL_COST {
            bail!("Not enough money");
        }
        self.money -= RAIL_COST;
        if self.grid_state[rail.r][rail.c] != vec![] {
            bail!("Rail or Station already exists");
        }
        self.grid_state[rail.r][rail.c] = rail.kind_v.clone();

        // Connect rails
        // L
        if self.grid_state[rail.r][rail.c].contains(&Kind::Left)
            && rail.c > 0
            && self.grid_state[rail.r][rail.c - 1].contains(&Kind::Right)
        {
            self.uf
                .merge(rail.r * input.n + rail.c, rail.r * input.n + rail.c - 1);
        }

        // R
        if self.grid_state[rail.r][rail.c].contains(&Kind::Right)
            && rail.c + 1 < input.n
            && self.grid_state[rail.r][rail.c + 1].contains(&Kind::Left)
        {
            self.uf
                .merge(rail.r * input.n + rail.c, rail.r * input.n + rail.c + 1);
        }

        // U
        if self.grid_state[rail.r][rail.c].contains(&Kind::Up)
            && rail.r > 0
            && self.grid_state[rail.r - 1][rail.c].contains(&Kind::Down)
        {
            self.uf
                .merge(rail.r * input.n + rail.c, (rail.r - 1) * input.n + rail.c);
        }

        // D
        if self.grid_state[rail.r][rail.c].contains(&Kind::Down)
            && rail.r + 1 < input.n
            && self.grid_state[rail.r + 1][rail.c].contains(&Kind::Up)
        {
            self.uf
                .merge(rail.r * input.n + rail.c, (rail.r + 1) * input.n + rail.c);
        }

        Ok(())
    }

    fn station(&mut self, input: &Input, station: Station) -> Result<()> {
        if self.money < STATION_COST {
            bail!("Not enough money");
        }
        self.money -= STATION_COST;

        if self.grid_state[station.r][station.c].len() == 4 {
            bail!("Station already exists");
        }
        self.grid_state[station.r][station.c] = station.kind_v.clone();

        // Connect rails
        // L
        if station.c > 0 && self.grid_state[station.r][station.c - 1].contains(&Kind::Right) {
            self.uf.merge(
                station.r * input.n + station.c,
                station.r * input.n + station.c - 1,
            );
        }
        // R
        if station.c + 1 < input.n
            && self.grid_state[station.r][station.c + 1].contains(&Kind::Left)
        {
            self.uf.merge(
                station.r * input.n + station.c,
                station.r * input.n + station.c + 1,
            );
        }
        // U
        if station.r > 0 && self.grid_state[station.r - 1][station.c].contains(&Kind::Down) {
            self.uf.merge(
                station.r * input.n + station.c,
                (station.r - 1) * input.n + station.c,
            );
        }
        // D
        if station.r + 1 < input.n && self.grid_state[station.r + 1][station.c].contains(&Kind::Up)
        {
            self.uf.merge(
                station.r * input.n + station.c,
                (station.r + 1) * input.n + station.c,
            );
        }

        Ok(())
    }

    fn commute(&mut self, input: &Input) -> Result<()> {
        self.income = 0;
        for i in 0..input.m {
            let (r_src, c_src) = input.src[i];
            let (r_dst, c_dst) = input.dst[i];
            let r_src = r_src as i64;
            let c_src = c_src as i64;
            let r_dst = r_dst as i64;
            let c_dst = c_dst as i64;

            'station_candidate_loop: for dr_src in -2..=2 {
                let rx = r_src + dr_src;
                if !(0 <= rx && rx < input.n as i64) {
                    continue;
                }
                for dc_src in -2..=2 {
                    let cx = c_src + dc_src;
                    if !(0 <= cx && cx < input.n as i64)
                        || (dr_src).abs() + (dc_src).abs() > 2
                        || self.grid_state[rx as usize][cx as usize].len() != 4
                    {
                        continue;
                    }
                    for dr_dst in -2..=2 {
                        let ry = r_dst + dr_dst;
                        if !(0 <= ry && ry < input.n as i64) {
                            continue;
                        }
                        for dc_dst in -2..=2 {
                            let cy = c_dst + dc_dst;
                            if !(0 <= cy && cy < input.n as i64)
                                || (dr_dst).abs() + (dc_dst).abs() > 2
                                || self.grid_state[ry as usize][cy as usize].len() != 4
                            {
                                continue;
                            }
                            let rx = rx as usize;
                            let cx = cx as usize;
                            let ry = ry as usize;
                            let cy = cy as usize;
                            if self.uf.same(rx * input.n + cx, ry * input.n + cy) {
                                let val = (r_src - r_dst).abs() + (c_src - c_dst).abs();
                                self.money += val;
                                self.income += val;
                                break 'station_candidate_loop;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct VisData {
    state: State,
    #[allow(dead_code)]
    commented_op: Option<CommentedOp>,
}

pub struct JudgeResult {
    pub score: i64,
}

pub fn judge(
    input_s: &str,
    output_s: &str,
    vis_data_vec: &mut Vec<VisData>,
    input_res: &mut Input,
) -> Result<JudgeResult> {
    let input = parse_input(input_s).context("input")?;
    *input_res = input.clone();
    let output = parse_output(output_s).context("output")?;

    let mut state = State::new(&input);
    vis_data_vec.push(VisData {
        state: state.clone(),
        commented_op: None,
    });

    for (i, commented_op) in output.commented_ops.clone().into_iter().enumerate() {
        if state.op_cnt >= input.t {
            bail!("Too many operations");
        }
        state.op_cnt += 1;
        let res = match commented_op.op.clone() {
            Op::None => {
                state.action = "Wait".to_string();
                Ok(())
            }
            Op::Rail(rail) => {
                state.action = format!("Build rail {} at ({}, {})", kind_v_to_usize(&rail.kind_v), rail.r, rail.c);
                state.rail(&input, rail)
            }
            Op::Station(station) => {
                state.action = format!("Build station at ({}, {})", station.r, station.c);
                state.station(&input, station)
            }
        };
        res.context(format!("{}-th build", i))?;

        let res = state.commute(&input);
        res.context(format!("{}-th commute", i))?;

        vis_data_vec.push(VisData {
            state: state.clone(),
            commented_op: Some(commented_op),
        });
    }

    if state.op_cnt < input.t {
        bail!("The number of operations is less than T");
    }

    Ok(JudgeResult { score: state.money })
}

struct Dist {
    n: usize,
    normal_r_v: Vec<Normal<f64>>,
    normal_c_v: Vec<Normal<f64>>,
    w_v: Vec<f64>,
    w_sum: f64,
}

impl Dist {
    fn new(rng: &mut ChaCha20Rng) -> Self {
        let n = rng.gen_range(5 as i64..=15 as i64) as usize;
        let mut normal_r_v = vec![];
        let mut normal_c_v = vec![];
        let mut w_v = vec![];
        let mut w_sum = 0.0;
        for _ in 0..n {
            let r = rng.gen_range(0.0..(N - 1) as f64);
            let c = rng.gen_range(0.0..(N - 1) as f64);
            let sigma = rng.gen_range(2.0..15.0);

            let w = rng.gen_range(0.0..1.0);
            w_v.push(w);
            w_sum += w;

            // normal_r
            let normal_r = Normal::new(r, sigma).unwrap();
            normal_r_v.push(normal_r);

            // normal_c
            let normal_c = Normal::new(c, sigma).unwrap();
            normal_c_v.push(normal_c);
        }

        Self {
            n,
            normal_r_v,
            normal_c_v,
            w_v,
            w_sum,
        }
    }

    fn sample(&self, rng: &mut ChaCha20Rng) -> (usize, usize) {
        // select the distribution to use according to w_v
        let mut w = rng.gen_range(0.0..self.w_sum);
        let mut i = 0;
        while w > self.w_v[i] && i < self.n - 1 {
            w -= self.w_v[i];
            i += 1;
        }

        let r = self.normal_r_v[i].sample(rng);
        let r = r.round() as isize as usize;  // negative values are rounded to zero if cast directly to usize
        let c = self.normal_c_v[i].sample(rng);
        let c = c.round() as isize as usize;
        (r, c)
    }
}

#[wasm_bindgen]
pub fn gen(seed: i32, problemId: String) -> String {
    let mut rng = ChaCha20Rng::seed_from_u64((seed as u64) ^ 94);

    let m_tmp = rng.gen_range(0.0..5.0);
    let m =(50.0 * 2.0f64.powf(m_tmp)).round() as usize;

    let dist_src = Dist::new(&mut rng);
    let dist_dst = Dist::new(&mut rng);
    let mut src = vec![];
    let mut dst = vec![];
    let mut min_distance = usize::MAX;
    let mut i = 0;
    while i < m {
        let (r_src, c_src) = dist_src.sample(&mut rng);
        if !(r_src < N && c_src < N) {
            continue;
        }
        let (r_dst, c_dst) = dist_dst.sample(&mut rng);
        if !(r_dst < N && c_dst < N) {
            continue;
        }

        let distance = r_src.abs_diff(r_dst) + c_src.abs_diff(c_dst);
        if distance <= 4 {
            continue;
        }
        min_distance = min_distance.min(distance);

        src.push((r_src, c_src));
        dst.push((r_dst, c_dst));
        i += 1;
    }

    let k = rng.gen_range((min_distance as i64).max(10) * RAIL_COST..=2 * N as i64 * RAIL_COST) + 10000;

    format!(
        "{} {} {} {}\n{}",
        N,
        m,
        k,
        T,
        src.iter()
            .zip(dst.iter())
            .map(|((r_src, c_src), (r_dst, c_dst))| {
                format!("{} {} {} {}", r_src, c_src, r_dst, c_dst)
            })
            .collect::<Vec<_>>()
            .join("\n")
    )
}

pub fn draw_background_svg(input: &Input) -> SVG {
    let svg = new_svg();
    let svg = draw_grid(input, svg);
    let svg = draw_source(input, svg);
    let svg = draw_destination(input, svg);
    svg
}

pub fn draw_svg(vis_data: &VisData, input: &Input, enable_tooltip: bool, draw_background: bool, svg: Option<SVG>) -> SVG {
    let mut svg = match svg {
        Some(svg) => svg,
        None => new_svg(),
    };
    if draw_background {
        svg = draw_grid(input, svg);
        svg = draw_source(input, svg);
        svg = draw_destination(input, svg);
    }
    svg = draw_station_range(vis_data, input, svg);
    svg = define_rails(svg);
    svg = draw_rails(vis_data, input, svg);
    svg = if enable_tooltip {
        draw_tooltips(vis_data, input, svg)
    } else {
        svg
    };
    svg
}

#[wasm_bindgen(getter_with_clone)]
pub struct Ret {
    pub score: i64,
    pub err: String,
    pub svg: String,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(getter_with_clone)]
pub struct SolInfo {
    pub error: Option<String>,
    pub score: i64,
    pub max_turn: usize,
    pub svg: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct VisCache {
    error: Option<String>,
    vis_data_vec: Vec<VisData>,
    input: Input,
}

#[cfg(target_arch = "wasm32")]
static VIS_CACHE: Lazy<Mutex<Option<VisCache>>> = Lazy::new(|| Mutex::new(None));

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(getter_with_clone)]
pub struct VisResult {
    pub svg: String,
    pub money: i64,
    pub income: i64,
    pub action: String,
    pub comments: Vec<String>,
}

#[wasm_bindgen]
pub fn vis(_input: String, _output: String, turn: usize) -> Ret {
    let mut vis_data_vec = vec![];
    let mut input_res = Input::new();

    let res = judge(_input.as_str(), _output.as_str(), &mut vis_data_vec, &mut input_res);
    let background_svg = draw_background_svg(&input_res).to_string();

    let vis_data = &vis_data_vec[turn];

    let comments = match &vis_data.commented_op {
        Some(commented_op) => commented_op.comments.clone(),
        None => vec![],
    };

    Ret {
        score: res.unwrap().score,
        err: "".to_string(),
        svg: draw_svg(&vis_data, &input_res, true, true, None).to_string(),
    }
}

#[wasm_bindgen]
pub fn get_max_turn(_input: String, _output: String) -> usize {
    T
}

#[derive(Clone)]
pub struct UnionFind {
    par: Vec<usize>,
    sz: Vec<usize>,
}

impl UnionFind {
    pub fn new(n: usize) -> UnionFind {
        let mut par = Vec::new();
        for i in 0..n {
            par.push(i);
        }
        let sz = vec![1; n];
        UnionFind { par, sz }
    }

    pub fn root(&mut self, x: usize) -> usize {
        if self.par[x] == x {
            x
        } else {
            self.par[x] = self.root(self.par[x]);
            self.par[x]
        }
    }

    pub fn merge(&mut self, x: usize, y: usize) {
        let mut x = self.root(x);
        let mut y = self.root(y);
        if x == y {
            return;
        }

        if self.sz[x] > self.sz[y] {
            std::mem::swap(&mut x, &mut y);
        }

        self.sz[y] = self.sz[x] + self.sz[y];
        self.sz[x] = self.sz[y];
        self.par[x] = y;
    }

    pub fn same(&mut self, x: usize, y: usize) -> bool {
        self.root(x) == self.root(y)
    }
}
