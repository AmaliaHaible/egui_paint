use eframe;
use eframe::egui::{self, Color32, Pos2};
use rand::{Rng, rng};
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

#[derive(Default, Clone)]
struct Line {
    points: Vec<egui::Pos2>,
    stroke: egui::Stroke,
    id: u64,
}
impl Line {
    fn from_point(p: egui::Pos2, s: egui::Stroke) -> Line {
        Self {
            // just to make it easy, every line is at least 2 points:
            points: vec![p, p],
            stroke: s,
            id: rng().random(),
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
                .any(|i| Self::segment_contains_point(s, e, w, c + (d - c) * i as f32 / 30.0))
        })
    }
}

struct DrawAction {
    drawn_line_id: Option<u64>,
    erased_lines: Vec<Line>,
}
struct RedoAction {
    undone_line: Option<Line>,
    unerased_lines: Vec<u64>,
}

struct MyEguiApp {
    lines: Vec<Line>,
    append_to_last_line: bool,
    current_stroke: egui::Stroke,
    current_zoomlevel: f32,
    current_position: egui::Pos2,
    last_mousepos: egui::Pos2,
    undo_stack: Vec<DrawAction>,
    redo_stack: Vec<RedoAction>,
    last_frametime: Instant,
    color_picker_open: bool,
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
            undo_stack: Vec::default(),
            redo_stack: Vec::default(),
            last_frametime: Instant::now(),
            color_picker_open: false,
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
        /* ui.painter()
        .add(eframe::epaint::QuadraticBezierShape::from_points_stroke(
            [
                Pos2::new(50.0, 50.0),
                Pos2::new(50.0, 100.0),
                Pos2::new(100.0, 100.0),
            ],
            false,
            Color32::from_black_alpha(0),
            egui::Stroke::new(32.0, Color32::BLUE),
        )); */
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
                ui.painter().line_segment(
                    [self.world_to_screen(s), self.world_to_screen(e)],
                    egui::Stroke::new(
                        line.stroke.width * self.current_zoomlevel,
                        line.stroke.color,
                    ),
                );
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

    fn erase_lines(&mut self, p: egui::Pos2) {
        let point = self.screen_to_world(p);
        let lastpoint = self.screen_to_world(self.last_mousepos);
        // self.lines.retain(|line| !line.contains_point(point));
        //self.lines.retain(|line| !line.overlaps_line(point, lastpoint));
        self.lines.retain(|line| {
            let overlaps = line.overlaps_line(point, lastpoint);
            if overlaps {
                if self.undo_stack.is_empty()
                    || self.undo_stack.last().unwrap().erased_lines.is_empty()
                {
                    self.undo_stack.push(DrawAction {
                        drawn_line_id: None,
                        erased_lines: vec![line.clone()],
                    });
                } else {
                    if let Some(lastaction) = self.undo_stack.last_mut() {
                        if !lastaction.erased_lines.is_empty() {
                            lastaction.erased_lines.push(line.clone());
                        }
                    }
                }
                false
            } else {
                true
            }
        });
    }
    fn undo(&mut self) {
        if let Some(mut lastaction) = self.undo_stack.pop() {
            assert!(lastaction.drawn_line_id.is_none() || lastaction.erased_lines.is_empty());
            if let Some(drawnline) = lastaction.drawn_line_id {
                self.lines.retain(|line| {
                    let keep = line.id != drawnline;
                    if keep {
                        true
                    } else {
                        self.redo_stack.push(RedoAction {
                            undone_line: Some(line.clone()),
                            unerased_lines: Vec::default(),
                        });
                        false
                    }
                });
            }
            if !lastaction.erased_lines.is_empty() {
                self.redo_stack.push(RedoAction {
                    undone_line: None,
                    unerased_lines: lastaction.erased_lines.iter().map(|l| l.id).collect(),
                });
                self.lines.append(&mut lastaction.erased_lines);
            }
        }
    }
    fn redo(&mut self) {
        if let Some(lastaction) = self.redo_stack.pop() {
            if let Some(line) = lastaction.undone_line {
                let id = line.id;
                self.lines.push(line);
                self.undo_stack.push(DrawAction {
                    drawn_line_id: Some(id),
                    erased_lines: Vec::default(),
                });
            }
            if !lastaction.unerased_lines.is_empty() {
                let mut to_erase = Vec::default();
                self.lines.retain(|line| {
                    !lastaction.unerased_lines.iter().any(|line_to_erase| {
                        let keep = *line_to_erase != line.id;
                        if keep {
                            false
                        } else {
                            to_erase.push(line.clone());
                            true
                        }
                    })
                });
                self.undo_stack.push(DrawAction {
                    drawn_line_id: None,
                    erased_lines: to_erase,
                });
            }
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context, handle_mouse: bool) {
        ctx.input_mut(|i| {
            // Drawing
            if i.pointer.primary_down() && handle_mouse {
                if let Some(p) = i.pointer.latest_pos() {
                    self.add_point(p);
                }
                self.append_to_last_line = true;
            } else {
                if self.append_to_last_line {
                    if let Some(last) = self.lines.last() {
                        self.undo_stack.push(DrawAction {
                            drawn_line_id: Some(last.id),
                            erased_lines: Vec::default(),
                        });
                    }
                }
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
            if i.pointer.secondary_down() && handle_mouse {
                if let Some(p) = i.pointer.latest_pos() {
                    self.erase_lines(p);
                }
            }
            let undokey = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Z);
            let redokey = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Y);
            let resetkey = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::R);
            if i.consume_shortcut(&undokey) {
                self.undo();
            }
            if i.consume_shortcut(&redokey) {
                self.redo();
            }
            if i.consume_shortcut(&resetkey) {
                self.lines = Vec::default();
                self.redo_stack = Vec::default();
                self.undo_stack = Vec::default();
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
        let mut markfordraw = false;
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let now = Instant::now();
                let deltatime = now.duration_since(self.last_frametime).as_secs_f32();
                self.last_frametime = now;
                let mut fps = 0.0;
                if deltatime > 0.0 {
                    fps = 1.0 / deltatime;
                }
                if fps < 10.0 && cfg!(debug_assertions) {
                    println!("FPS low: {:.3}, Timestamp: {:?}", fps, now);
                }

                ui.heading("Stroke Width: ");
                let _stroke_slider = ui.add(egui::Slider::new(
                    &mut self.current_stroke.width,
                    1.0..=50.0,
                ));
                let colorpicker = egui::widgets::color_picker::color_edit_button_srgba(
                    ui,
                    &mut self.current_stroke.color,
                    egui::widgets::color_picker::Alpha::Opaque,
                );
                if colorpicker.clicked() {
                    self.color_picker_open = !self.color_picker_open;
                }
                if self.color_picker_open && colorpicker.clicked_elsewhere() {
                    markfordraw = true;
                }

                if self.color_picker_open && cfg!(debug_assertions) {
                    ui.heading("AAA");
                }
                ui.heading(format!(
                    "Lines: {:?}, FPS: {:.2}, Undo: {:?}, Redo: {:?}",
                    self.lines.len(),
                    fps,
                    self.undo_stack.len(),
                    self.redo_stack.len(),
                ));
            });
        });
        /* egui::TopBottomPanel::bottom("bottom").show(ctx, |ui|{

        }); */

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw(ui);
            self.handle_input(ctx, ui.ui_contains_pointer() && !self.color_picker_open);
        });
        if markfordraw {
            self.color_picker_open = false;
        }
        ctx.request_repaint();
    }
}
