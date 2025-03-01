use svg::node::element::{
    Circle, Definitions, Group, Line, Path, Rectangle, Title, Use, SVG,
};
use svg::node::Text;

use crate::kind_v_to_usize;
use crate::{Input, VisData};

const W: f64 = 800.0;
const H: f64 = 800.0;
const PADDING: f64 = 10.0;

pub fn new_svg() -> SVG {
    svg::Document::new()
        .set("id", "vis")
        .set(
            "viewBox",
            (-PADDING, -PADDING, W + 2.0 * PADDING, H + 2.0 * PADDING),
        )
        .set("width", W + 2.0 * PADDING)
        .set("height", H + 2.0 * PADDING)
}

pub fn draw_grid(input: &Input, mut doc: SVG) -> SVG {
    doc = doc.add(
        Rectangle::new()
            .set("x", -PADDING)
            .set("y", -PADDING)
            .set("width", W + 2.0 * PADDING)
            .set("height", H + 2.0 * PADDING)
            .set("fill", "white")
            .set("stroke-width", 0.0),
    );

    for i in 0..=input.n {
        let y = i as f64 * H / input.n as f64;
        doc = doc.add(
            Line::new()
                .set("x1", 0)
                .set("y1", y)
                .set("x2", W)
                .set("y2", y)
                .set("stroke", "lightgray")
                .set("stroke-width", 0.5),
        );
    }

    for i in 0..=input.n {
        let x = i as f64 * W / input.n as f64;
        doc = doc.add(
            Line::new()
                .set("x1", x)
                .set("y1", 0)
                .set("x2", x)
                .set("y2", H)
                .set("stroke", "lightgray")
                .set("stroke-width", 0.5),
        );
    }

    doc
}

fn draw_rail(rail_type: &str, theta: f64, x: f64, y: f64, ratio: f64, doc: SVG) -> SVG {
    doc.add(
        Use::new()
            .set("href", rail_type)
            .set("stroke", "silver")
            .set(
                "transform",
                format!(
                    "rotate({},{},{})",
                    theta,
                    x + 15.0 * ratio,
                    y + 15.0 * ratio
                ),
            )
            .set("x", x)
            .set("y", y),
    )
}

pub fn draw_rails(vis_data: &VisData, input: &Input, mut doc: SVG) -> SVG {
    let ratio = H as f64 / input.n as f64 / 30.0;
    for i in 0..input.n {
        for j in 0..input.n {
            let x = j as f64 * W / input.n as f64;
            let y = i as f64 * H / input.n as f64;
            let kind = kind_v_to_usize(&vis_data.state.grid_state[i][j]);
            match kind {
                usize::MAX => {
                    continue;
                }
                0 => {
                    // station
                    doc = draw_rail("#rail3", 0.0, x, y, ratio, doc);
                    doc = draw_rail("#rail3", 90.0, x, y, ratio, doc);
                }
                1 => doc = draw_rail("#rail1", 90.0, x, y, ratio, doc),
                2 => doc = draw_rail("#rail1", 0.0, x, y, ratio, doc),
                3 => doc = draw_rail("#rail2", 270.0, x, y, ratio, doc),
                4 => doc = draw_rail("#rail2", 0.0, x, y, ratio, doc),
                5 => doc = draw_rail("#rail2", 90.0, x, y, ratio, doc),
                6 => doc = draw_rail("#rail2", 180.0, x, y, ratio, doc),
                _ => {
                    unreachable!("kind: {}", kind);
                }
            }
        }
    }

    doc
}

pub fn draw_station_range(vis_data: &VisData, input: &Input, mut doc: SVG) -> SVG {
    for r in 0..input.n {
        for c in 0..input.n {
            if kind_v_to_usize(&vis_data.state.grid_state[r][c]) == 0 {
                for dr in -2..=2i64 {
                    for dc in -2..=2i64 {
                        if dr.abs() + dc.abs() > 2 {
                            continue;
                        }
                        let nr = r as i64 + dr;
                        let nc = c as i64 + dc;
                        if nr < 0 || nr >= input.n as i64 || nc < 0 || nc >= input.n as i64 {
                            continue;
                        }
                        let x = nc as f64 * W / input.n as f64;
                        let y = nr as f64 * H / input.n as f64;
                        doc = doc.add(
                            Rectangle::new()
                                .set("x", x)
                                .set("y", y)
                                .set("width", W / input.n as f64)
                                .set("height", H / input.n as f64)
                                .set("fill", "palegreen")
                                .set("fill-opacity", 0.5),
                        );
                    }
                }
            }
        }
    }
    doc
}

pub fn draw_source(input: &Input, mut doc: SVG) -> SVG {
    let ratio = H as f64 / input.n as f64 / 30.0;
    for i in 0..input.m {
        let x = input.src[i].1 as f64 * H / input.n as f64;
        let y = input.src[i].0 as f64 * W / input.n as f64;
        doc = doc.add(
            Circle::new()
                .set("cx", x + 10.0 * ratio)
                .set("cy", y + 10.0 * ratio)
                .set("r", 5.0 * ratio)
                .set("fill", "orangered"),
        );
    }

    doc
}

pub fn draw_destination(input: &Input, mut doc: SVG) -> SVG {
    let ratio = H as f64 / input.n as f64 / 30.0;
    for i in 0..input.m {
        let x = input.dst[i].1 as f64 * H / input.n as f64;
        let y = input.dst[i].0 as f64 * W / input.n as f64;
        doc = doc.add(
            Circle::new()
                .set("cx", x + 20.0 * ratio)
                .set("cy", y + 20.0 * ratio)
                .set("r", 5.0 * ratio)
                .set("fill", "deepskyblue"),
        );
    }

    doc
}

pub fn draw_tooltips(vis_data: &VisData, input: &Input, mut doc: SVG) -> SVG {
    let mut src_mp = std::collections::HashMap::new();
    let mut dst_mp = std::collections::HashMap::new();
    for i in 0..input.m {
        let tmp_v = src_mp.entry(input.src[i]).or_insert(vec![]);
        tmp_v.push(i);
        let tmp_v = dst_mp.entry(input.dst[i]).or_insert(vec![]);
        tmp_v.push(i);
    }

    for r in 0..input.n {
        for c in 0..input.n {
            let mut statement = vec![format!("({}, {})", r, c)];
            let kind = kind_v_to_usize(&vis_data.state.grid_state[r][c]);
            match kind{
                0 => {
                    statement.push("Station".to_string());
                }
                1 | 2 | 3 | 4 | 5 | 6 => {
                    statement.push(format!("Rail {}", kind))
                }
                usize::MAX => {
                    statement.push("Empty".to_string());
                }
                _ => unreachable!(),
            }

            let mut src_v = vec![];
            let mut dst_v = vec![];
            if let Some(v) = src_mp.get(&(r, c)) {
                for &i in v {
                    src_v.push(i.to_string());
                }
            }
            if let Some(v) = dst_mp.get(&(r, c)) {
                for &i in v {
                    dst_v.push(i.to_string());
                }
            }
            if !src_v.is_empty() {
                statement.push(format!("home: {}", src_v.join(",")));
            }
            if !dst_v.is_empty() {
                statement.push(format!("workplace: {}", dst_v.join(",")));
            }
            let mut all_v = src_v.clone();
            all_v.extend(dst_v.clone());

            let mut rect = create_rectangle(
                c as i64 * W as i64 / input.n as i64,
                r as i64 * H as i64 / input.n as i64,
                W / input.n as f64,
                H / input.n as f64,
                "gray",
                0.0,
                Some("black"),
                0.0,
                Some(statement.join("\n")),
            );
            rect = rect.set("onmouseover", format!("showHighlight([{}])", all_v.join(",")));
            rect = rect.set("onmouseleave", format!("clearHighlight([{}])", all_v.join(",")));
            rect = rect.set("class", format!("{}", all_v.iter().map(|v| "p_".to_string() + v ).collect::<Vec<_>>().join(" ")));
            doc = doc.add(rect);
        }
    }

    doc
}

fn create_rectangle(
    x: i64,
    y: i64,
    width: f64,
    height: f64,
    fill: &str,
    fill_opacity: f64,
    stroke: Option<&str>,
    stroke_width: f64,
    title: Option<String>,
) -> Rectangle {
    let mut rect = Rectangle::new()
        .set("x", x as f64)
        .set("y", y as f64)
        .set("width", width)
        .set("height", height)
        .set("fill", fill)
        .set("fill-opacity", fill_opacity);
    if let Some(stroke) = stroke {
        rect = rect.set("stroke", stroke).set("stroke-width", stroke_width);
    }

    if let Some(title) = title {
        rect = rect.add(Title::new("animals").add(Text::new(title)));
    }

    rect
}

fn add_line(
    g: Group,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    stroke_width: f64,
    stroke: &str,
) -> Group {
    g.add(
        Line::new()
            .set("x1", x1)
            .set("x2", y1)
            .set("y1", x2)
            .set("y2", y2)
            .set("stroke-width", stroke_width)
            .set("stroke", stroke),
    )
}

pub fn define_rails(doc: SVG) -> SVG {
    let ratio = H as f64 / 50.0 / 30.0;
    let mut rail1 = Group::new().set("id", "rail1").set("stroke_width", 2.0);
    let rail1_stroke_width = 0.8;
    let rail1_stroke = "silver";
    rail1 = add_line(
        rail1,
        12.0 * ratio,
        12.0 * ratio,
        0.0 * ratio,
        30.0 * ratio,
        rail1_stroke_width,
        rail1_stroke,
    );
    rail1 = add_line(
        rail1,
        18.0 * ratio,
        18.0 * ratio,
        0.0 * ratio,
        30.0 * ratio,
        rail1_stroke_width,
        rail1_stroke,
    );
    rail1 = add_line(
        rail1,
        12.0 * ratio,
        18.0 * ratio,
        3.0 * ratio,
        3.0 * ratio,
        rail1_stroke_width,
        rail1_stroke,
    );
    rail1 = add_line(
        rail1,
        12.0 * ratio,
        18.0 * ratio,
        9.0 * ratio,
        9.0 * ratio,
        rail1_stroke_width,
        rail1_stroke,
    );
    rail1 = add_line(
        rail1,
        12.0 * ratio,
        18.0 * ratio,
        15.0 * ratio,
        15.0 * ratio,
        rail1_stroke_width,
        rail1_stroke,
    );
    rail1 = add_line(
        rail1,
        12.0 * ratio,
        18.0 * ratio,
        21.0 * ratio,
        21.0 * ratio,
        rail1_stroke_width,
        rail1_stroke,
    );
    rail1 = add_line(
        rail1,
        12.0 * ratio,
        18.0 * ratio,
        27.0 * ratio,
        27.0 * ratio,
        rail1_stroke_width,
        rail1_stroke,
    );

    let mut rail2 = Group::new().set("id", "rail2").set("stroke_width", 2.0);
    let rail2_stroke_width = 0.8;
    let rail2_stroke = "silver";
    let r12 = 12.0 * ratio;
    let r18 = 18.0 * ratio;
    rail2 = rail2.add(
        Path::new()
            .set("d", format!("M{},0 A{},{} 0 0,1 0,{}", r12, r12, r12, r12))
            .set("fill", "none")
            .set("stroke-width", rail2_stroke_width)
            .set("stroke", rail2_stroke),
    );
    rail2 = rail2.add(
        Path::new()
            .set("d", format!("M{},0 A{},{} 0 0,1 0,{}", r18, r18, r18, r18))
            .set("fill", "none")
            .set("stroke-width", rail2_stroke_width)
            .set("stroke", rail2_stroke),
    );
    rail2 = add_line(
        rail2,
        12.0 * ratio,
        18.0 * ratio,
        2.0 * ratio,
        3.0 * ratio,
        rail2_stroke_width,
        rail2_stroke,
    );
    rail2 = add_line(
        rail2,
        11.0 * ratio,
        16.0 * ratio,
        5.0 * ratio,
        8.0 * ratio,
        rail2_stroke_width,
        rail2_stroke,
    );
    rail2 = add_line(
        rail2,
        8.0 * ratio,
        13.0 * ratio,
        8.0 * ratio,
        13.0 * ratio,
        rail2_stroke_width,
        rail2_stroke,
    );
    rail2 = add_line(
        rail2,
        5.0 * ratio,
        8.0 * ratio,
        11.0 * ratio,
        16.0 * ratio,
        rail2_stroke_width,
        rail2_stroke,
    );
    rail2 = add_line(
        rail2,
        2.0 * ratio,
        3.0 * ratio,
        12.0 * ratio,
        18.0 * ratio,
        rail2_stroke_width,
        rail2_stroke,
    );

    let mut rail3 = Group::new().set("id", "rail3").set("stroke_width", 2.0);
    let rail3_stroke_width = 1.3;
    let rail3_stroke = "lightslategray";
    rail3 = add_line(
        rail3,
        12.0 * ratio,
        12.0 * ratio,
        0.0 * ratio,
        30.0 * ratio,
        rail3_stroke_width,
        rail3_stroke,
    );
    rail3 = add_line(
        rail3,
        18.0 * ratio,
        18.0 * ratio,
        0.0 * ratio,
        30.0 * ratio,
        rail3_stroke_width,
        rail3_stroke,
    );
    rail3 = add_line(
        rail3,
        12.0 * ratio,
        18.0 * ratio,
        3.0 * ratio,
        3.0 * ratio,
        rail3_stroke_width,
        rail3_stroke,
    );
    rail3 = add_line(
        rail3,
        12.0 * ratio,
        18.0 * ratio,
        9.0 * ratio,
        9.0 * ratio,
        rail3_stroke_width,
        rail3_stroke,
    );
    rail3 = add_line(
        rail3,
        12.0 * ratio,
        18.0 * ratio,
        15.0 * ratio,
        15.0 * ratio,
        rail3_stroke_width,
        rail3_stroke,
    );
    rail3 = add_line(
        rail3,
        12.0 * ratio,
        18.0 * ratio,
        21.0 * ratio,
        21.0 * ratio,
        rail3_stroke_width,
        rail3_stroke,
    );
    rail3 = add_line(
        rail3,
        12.0 * ratio,
        18.0 * ratio,
        27.0 * ratio,
        27.0 * ratio,
        rail3_stroke_width,
        rail3_stroke,
    );
    doc.add(Definitions::new().add(rail1).add(rail2).add(rail3))
}
