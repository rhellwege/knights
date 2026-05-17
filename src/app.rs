use crate::board::SimulationState;
use eframe::egui;
use std::io::Write;

impl From<crate::common::Color> for egui::Color32 {
    fn from(value: crate::common::Color) -> Self {
        let rgba = value.as_u32();
        let r = ((rgba >> 24) & 0xff) as u8;
        let g = ((rgba >> 16) & 0xff) as u8;
        let b = ((rgba >> 8) & 0xff) as u8;
        let a = (rgba & 0xff) as u8;
        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
    }
}

/// A trait that combines interaction logic and visual rendering.
pub trait SimulationApp {
    fn run<S>(&mut self, simulation: S) -> std::io::Result<()>
    where
        S: SimulationState + 'static;
}

pub struct TerminalApp;

impl SimulationApp for TerminalApp {
    fn run<S>(&mut self, mut simulation: S) -> std::io::Result<()>
    where
        S: SimulationState + 'static,
    {
        use std::io::{stdin, stdout};
        use termion::event::Key;
        use termion::input::TermRead;
        use termion::raw::IntoRawMode;

        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;
        let (w, h) = simulation.mapper_dimensions();

        write!(
            stdout,
            "{}{}Knight's Tour ({}x{}). Press SPACE to step, Q to quit.\r\n",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            w,
            h
        )?;

        self.render(&simulation, &mut stdout)?;

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char(' ') => {
                    if !simulation.is_finished() {
                        simulation.step();
                        self.render(&simulation, &mut stdout)?;
                    }
                }
                Key::Char('q') | Key::Esc => break,
                _ => {}
            }
            stdout.flush()?;
        }

        write!(stdout, "\r\nExiting...\r\n")?;
        stdout.flush()?;
        Ok(())
    }
}

impl TerminalApp {
    fn render<S, W: Write>(&self, simulation: &S, writer: &mut W) -> std::io::Result<()>
    where
        S: SimulationState,
    {
        write!(writer, "{}", termion::cursor::Goto(1, 2))?;
        let (w, h) = simulation.mapper_dimensions();

        let mut max_val = 0;
        for y in 0..h {
            for x in 0..w {
                max_val = max_val.max(simulation.get_value(x, y).unwrap_or(0));
            }
        }
        let width_padding = if max_val > 99 {
            3
        } else if max_val > 9 {
            2
        } else {
            1
        };

        for y in 0..h {
            let mut line = String::new();
            for x in 0..w {
                match simulation.get_value(x, y) {
                    Some(val) => {
                        line.push_str(&format!("{:>width$} ", val, width = width_padding));
                    }
                    None => {
                        line.push_str(&format!("{:>width$} ", ".", width = width_padding));
                    }
                }
            }
            write!(writer, "{}\r\n", line)?;
        }
        Ok(())
    }
}

pub struct EguiApp;

impl SimulationApp for EguiApp {
    fn run<S>(&mut self, simulation: S) -> std::io::Result<()>
    where
        S: SimulationState + 'static,
    {
        let options = eframe::NativeOptions::default();
        eframe::run_native(
            "Knight's Tour",
            options,
            Box::new(|_cc| Ok(Box::new(EguiSimulator::new(simulation)))),
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

struct EguiSimulator<S>
where
    S: SimulationState,
{
    simulation: S,
    /// Cache for d-values (mapping labels)
    d_mapping_cache: Vec<u64>,
    /// Cache for square rectangles to avoid recalculating coordinate math every frame.
    cached_square_rects: Vec<egui::Rect>,
    /// The bounding rect used to generate the current cache.
    cached_grid_rect: egui::Rect,
    /// Whether to show the trail of moves
    show_trail: bool,
}

impl<S> EguiSimulator<S>
where
    S: SimulationState,
{
    fn new(simulation: S) -> Self {
        let (w, h) = simulation.mapper_dimensions();
        let mut d_mapping_cache = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                d_mapping_cache.push(simulation.get_d(x, y).unwrap_or(0));
            }
        }

        Self {
            simulation,
            d_mapping_cache,
            cached_square_rects: Vec::new(),
            cached_grid_rect: egui::Rect::NOTHING,
            show_trail: false,
        }
    }

    /// Rebuilds the square rectangle cache if the board size or position has changed.
    fn update_cache(&mut self, rect: egui::Rect) {
        if self.cached_grid_rect == rect {
            return;
        }

        let (w, h) = self.simulation.mapper_dimensions();
        let square_size = rect.width() / w as f32;
        self.cached_square_rects.clear();

        for y in 0..h {
            for x in 0..w {
                let square_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(x as f32 * square_size, y as f32 * square_size),
                    egui::vec2(square_size, square_size),
                );
                self.cached_square_rects.push(square_rect);
            }
        }
        self.cached_grid_rect = rect;
    }
}

impl<S> eframe::App for EguiSimulator<S>
where
    S: SimulationState,
{
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Handle input (Ui derefs to Context in 0.34)
        if ui.input(|i| i.key_pressed(egui::Key::Space)) {
            self.simulation.step();
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Knight's Tour Simulation");
            ui.horizontal(|ui| {
                ui.label("Press SPACE to step.");
                if ui.button("Complete").clicked() {
                    while !self.simulation.is_finished() {
                        self.simulation.step();
                    }
                }
                if ui.button("Reset").clicked() {
                    self.simulation.reset();
                }
                ui.checkbox(&mut self.show_trail, "Show Trail");
            });

            let (w, h) = self.simulation.mapper_dimensions();
            let current_pos = self.simulation.current_pos();

            // Reserve space for the board
            let available_size = ui.available_size();
            let board_side = available_size.x.min(available_size.y) * 0.9;
            let (rect, _response) =
                ui.allocate_exact_size(egui::vec2(board_side, board_side), egui::Sense::hover());

            // Sync cache with current allocation
            self.update_cache(rect);

            let painter = ui.painter();
            let square_size = board_side / w as f32;

            for i in 0..(w * h) as usize {
                let x = i as u32 % w;
                let y = i as u32 / w;
                let square_rect = self.cached_square_rects[i];

                // Alternating colors for chessboard
                let is_dark = (x + y) % 2 == 1;
                let fill_color = if is_dark {
                    egui::Color32::from_gray(80)
                } else {
                    egui::Color32::from_gray(200)
                };

                painter.rect_filled(square_rect, 0.0, fill_color);

                // Draw mapping label (d index) in the top-left corner
                let d_val = self.d_mapping_cache[i];
                let label_color = if is_dark {
                    egui::Color32::LIGHT_GRAY
                } else {
                    egui::Color32::DARK_GRAY
                };
                painter.text(
                    square_rect.min + egui::vec2(2.0, 2.0),
                    egui::Align2::LEFT_TOP,
                    format!("d:{}", d_val),
                    egui::FontId::monospace(square_size * 0.15),
                    label_color,
                );

                // Draw visit order index (step) in the center
                let value = self.simulation.get_value(x, y);
                if let Some(val) = value {
                    let text_color = if is_dark {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::BLACK
                    };
                    painter.text(
                        square_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        val.to_string(),
                        egui::FontId::monospace(square_size * 0.4),
                        text_color,
                    );
                }

                // Draw the knight or trail
                let is_knight = if let Some((cx, cy)) = current_pos {
                    cx == x && cy == y
                } else {
                    false
                };

                if is_knight || (self.show_trail && value.is_some()) {
                    painter.circle_filled(
                        square_rect.center(),
                        square_size * 0.35,
                        self.simulation.get_color(x, y).unwrap(), // get_color will always return Some if value is some
                    );
                }
            }

            if self.simulation.is_finished() {
                ui.label("Simulation Finished!");
            }
        });
    }
}
