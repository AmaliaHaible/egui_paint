use eframe;
use eframe::egui::{self, Color32, Pos2};
use std::time::Instant;
fn main() {
    let nativeoptions = eframe::NativeOptions::default();
    eframe::run_native(
        "Egui Paint",
        nativeoptions,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
    .unwrap()
}

#[derive(Default)]
struct Line {
    points: Vec<egui::Pos2>,
    stroke: egui::Stroke,
}
impl Line {
    fn from_point(p: egui::Pos2, s: egui::Stroke) -> Line {
        Self {
            // just to make it easy, every line is at least 2 points:
            points: vec![p, p],
            stroke: s,
        }
    }

    fn segment_contains_point(s: Pos2, e: Pos2, w: f32, p: Pos2) -> bool {
        // Check if in reach of end points for rounded lines
        if s.distance_sq(p) < w * w / 4.0 || e.distance_sq(p) < w * w / 4.0 {
            return true;
        }

        // Compute the vector representing the line segment.
        let line_vec = Pos2 {
            x: e.x - s.x,
            y: e.y - s.y,
        };

        let line_len_sq = line_vec.x * line_vec.x + line_vec.y * line_vec.y;
        // If the line segment is a point, check if p is within w/2 of s.
        if line_len_sq == 0.0 {
            return (p.x - s.x).abs() <= w / 2.0 && (p.y - s.y).abs() <= w / 2.0;
        }
        // Compute the projection scalar (t) of p onto the line.
        let t = ((p.x - s.x) * line_vec.x + (p.y - s.y) * line_vec.y) / line_len_sq;
        // If t is not between 0 and 1, then the projection of p lies outside the segment.
        if t < 0.0 || t > 1.0 {
            return false;
        }
        // Compute the projection point on the line.
        let proj = Pos2 {
            x: s.x + line_vec.x * t,
            y: s.y + line_vec.y * t,
        };
        // Compute the distance from p to its projection on the line.
        let dx = p.x - proj.x;
        let dy = p.y - proj.y;
        let distance = (dx * dx + dy * dy).sqrt();
        // Check if the distance is within half the width.
        distance <= w / 2.0
    }

    fn overlaps_line(&self, c: egui::Pos2, d: egui::Pos2) -> bool {
        // Stroke width independent
        // TODO: Good or bad?
        self.points.windows(2).any(|window| {
            let s = window[0];
            let e = window[1];
            let w = self.stroke.width;
            // if Line::segment_contains_point(s, e, w, c){
            //     return true;
            // }
            //Self::segment_intersects_segment(s, e, c, d)
            // fuck this
            (0..=30)
                .into_iter()
                .any(|i| Self::segment_contains_point(s, e, w, c + (d - c) * i as f32 / 10.0))
        })
    }
}
struct MyEguiApp {
    lines: Vec<Line>,
    append_to_last_line: bool,
    current_stroke: egui::Stroke,
    current_zoomlevel: f32,
    current_position: egui::Pos2,
    last_mousepos: egui::Pos2,
    last_frametime: Instant,
}

impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            lines: Vec::default(),
            append_to_last_line: false,
            current_stroke: egui::Stroke::default(),
            current_zoomlevel: 1.0,
            current_position: egui::Pos2::default(),
            last_mousepos: egui::Pos2::default(),
            last_frametime: Instant::now(),
        }
    }
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_zoomlevel: 1.0,
            current_stroke: egui::Stroke::new(10.0, Color32::WHITE),
            last_frametime: Instant::now(),
            ..Self::default()
        }
    }

    fn world_to_screen(&self, world_point: egui::Pos2) -> egui::Pos2 {
        ((world_point - self.current_position) * self.current_zoomlevel).to_pos2()
    }

    fn screen_to_world(&self, screen_point: egui::Pos2) -> egui::Pos2 {
        screen_point / self.current_zoomlevel + self.current_position.to_vec2()
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        // for line in &self.lines {
        //     ui.painter().line(line.points.clone(), line.stroke);
        // }
        self.lines.iter().for_each(|line| {
            /* ui.painter().line(
                line.points
                    .iter()
                    .map(|point| self.world_to_screen(*point))
                    .collect(),
                egui::Stroke::new(
                    line.stroke.width * self.current_zoomlevel,
                    line.stroke.color,
                ),
            );
            ui.painter().circle_filled(
                self.world_to_screen(line.points[0]),
                line.stroke.width * self.current_zoomlevel / 2.0,
                line.stroke.color,
            );
            ui.painter().circle_filled(
                self.world_to_screen(*line.points.last().unwrap()),
                line.stroke.width * self.current_zoomlevel / 2.0,
                line.stroke.color,
            ); */

            line.points.iter().for_each(|point| {
                ui.painter().circle_filled(
                    self.world_to_screen(*point),
                    line.stroke.width * self.current_zoomlevel / 2.0,
                    line.stroke.color,
                );
            });
            line.points.windows(2).for_each(|window| {
                let s = window[0];
                let e = window[1];
                ui.painter().line_segment([self.world_to_screen(s),self.world_to_screen(e)], egui::Stroke::new(line.stroke.width*self.current_zoomlevel, line.stroke.color));
            })
        });
    }

    fn add_point(&mut self, p: egui::Pos2) {
        let point = self.screen_to_world(p);
        if self.append_to_last_line {
            // lastmut should only be error when its empty, and we should never append_to_last_line
            // if we havent even started
            let d = self
                .lines
                .last()
                .unwrap()
                .points
                .last()
                .unwrap()
                .distance_sq(point);
            if d > 0.1 {
                self.lines.last_mut().unwrap().points.push(point);
            }
        } else {
            self.lines
                .push(Line::from_point(point, self.current_stroke));
        }
    }

    fn erase_line(&mut self, p: egui::Pos2) {
        let point = self.screen_to_world(p);
        let lastpoint = self.screen_to_world(self.last_mousepos);
        // self.lines.retain(|line| !line.contains_point(point));
        self.lines
            .retain(|line| !line.overlaps_line(point, lastpoint));
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Drawing
            if i.pointer.primary_down() {
                if let Some(p) = i.pointer.latest_pos() {
                    self.add_point(p);
                }
                self.append_to_last_line = true;
            } else {
                self.append_to_last_line = false;
            }
            // Change Stroke size
            // let scroll_data = i.raw_scroll_delta;
            // self.current_stroke.width += scroll_data.x+scroll_data.y;
            self.current_stroke.width += (i.zoom_delta() - 1.0) * 10.0;
            if i.key_pressed(egui::Key::Plus) {
                self.current_stroke.width = (self.current_stroke.width + 1.0).ceil();
            }
            if i.key_pressed(egui::Key::Minus) {
                self.current_stroke.width = (self.current_stroke.width - 1.0).floor();
            }
            self.current_stroke.width = self.current_stroke.width.max(1.0);

            // erase
            if i.pointer.secondary_down() {
                if let Some(p) = i.pointer.latest_pos() {
                    self.erase_line(p);
                }
            }

            // zoom?
            let factor = i.smooth_scroll_delta.y;
            let f = if factor < 0.0 {
                0.99
            } else if factor > 0.0 {
                1.01
            } else {
                1.0
            };
            if f != 1.0 {
                if let Some(p) = i.pointer.latest_pos() {
                    self.current_position = (p.to_vec2() / self.current_zoomlevel
                        + self.current_position.to_vec2()
                        - p.to_vec2() / (self.current_zoomlevel * f))
                        .to_pos2();
                }
            }
            self.current_zoomlevel *= f;

            if let Some(p) = i.pointer.latest_pos() {
                self.last_mousepos = p;
            }
        });
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            let now = Instant::now();
            let deltatime = now.duration_since(self.last_frametime).as_secs_f32();
            self.last_frametime = now;
            let mut fps = 0.0;
            if deltatime > 0.0 {
                fps = 1.0 / deltatime;
            }
            ui.heading(format!(
                "Lines: {:?}, Stroke Width: {:.2}, FPS: {:.2}",
                self.lines.len(),
                self.current_stroke.width,
                fps,
            ));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw(ui);
        });
        ctx.request_repaint();
    }
}
