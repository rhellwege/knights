use crate::board::BoardState;
use crate::mapper::OneToTwoMapper;
use crate::piece::Piece;
use eframe::egui;
use std::io::Write;

/// A trait that combines interaction logic and visual rendering.
pub trait SimulationApp {
    fn run<P, M>(&mut self, board: BoardState<P, M>) -> std::io::Result<()>
    where
        P: Piece + 'static,
        M: OneToTwoMapper + 'static;
}

pub struct TerminalApp;

impl SimulationApp for TerminalApp {
    fn run<P, M>(&mut self, mut board: BoardState<P, M>) -> std::io::Result<()>
    where
        P: Piece + 'static,
        M: OneToTwoMapper + 'static,
    {
        use std::io::{stdin, stdout};
        use termion::event::Key;
        use termion::input::TermRead;
        use termion::raw::IntoRawMode;

        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;
        let (w, h) = board.mapper_dimensions();

        write!(
            stdout,
            "{}{}Knight's Tour ({}x{}). Press SPACE to step, Q to quit.\r\n",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            w,
            h
        )?;

        self.render(&board, &mut stdout)?;

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char(' ') => {
                    if !board.is_finished() {
                        board.step();
                        self.render(&board, &mut stdout)?;
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
    fn render<P, M, W: Write>(
        &self,
        board: &BoardState<P, M>,
        writer: &mut W,
    ) -> std::io::Result<()>
    where
        P: Piece,
        M: OneToTwoMapper,
    {
        write!(writer, "{}", termion::cursor::Goto(1, 2))?;
        let (w, h) = board.mapper_dimensions();

        let mut max_val = 0;
        for y in 0..h {
            for x in 0..w {
                max_val = max_val.max(board.get_value(x, y));
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
                let val = board.get_value(x, y);
                if val == 0 {
                    line.push_str(&format!("{:>width$} ", ".", width = width_padding));
                } else {
                    line.push_str(&format!("{:>width$} ", val, width = width_padding));
                }
            }
            write!(writer, "{}\r\n", line)?;
        }
        Ok(())
    }
}

pub struct EguiApp;

impl SimulationApp for EguiApp {
    fn run<P, M>(&mut self, board: BoardState<P, M>) -> std::io::Result<()>
    where
        P: Piece + 'static,
        M: OneToTwoMapper + 'static,
    {
        let options = eframe::NativeOptions::default();
        eframe::run_native(
            "Knight's Tour",
            options,
            Box::new(|_cc| Ok(Box::new(EguiSimulator::new(board)))),
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

struct EguiSimulator<P, M>
where
    P: Piece,
    M: OneToTwoMapper,
{
    board: BoardState<P, M>,
    /// Cache for d-values (mapping labels)
    d_mapping_cache: Vec<u64>,
    /// Cache for square rectangles to avoid recalculating coordinate math every frame.
    cached_square_rects: Vec<egui::Rect>,
    /// The bounding rect used to generate the current cache.
    cached_grid_rect: egui::Rect,
}

impl<P, M> EguiSimulator<P, M>
where
    P: Piece,
    M: OneToTwoMapper,
{
    fn new(board: BoardState<P, M>) -> Self {
        let (w, h) = board.mapper_dimensions();
        let mut d_mapping_cache = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                d_mapping_cache.push(board.get_d(x, y).unwrap_or(0));
            }
        }

        Self {
            board,
            d_mapping_cache,
            cached_square_rects: Vec::new(),
            cached_grid_rect: egui::Rect::NOTHING,
        }
    }

    /// Rebuilds the square rectangle cache if the board size or position has changed.
    fn update_cache(&mut self, rect: egui::Rect) {
        if self.cached_grid_rect == rect {
            return;
        }

        let (w, h) = self.board.mapper_dimensions();
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

impl<P, M> eframe::App for EguiSimulator<P, M>
where
    P: Piece,
    M: OneToTwoMapper,
{
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Handle input
        if ui.input(|i| i.key_pressed(egui::Key::Space)) {
            self.board.step();
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Knight's Tour Simulation");
            ui.label("Press SPACE to step.");

            let (w, h) = self.board.mapper_dimensions();
            let current_pos = self.board.current_pos();

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
                let val = self.board.get_value(x, y);
                if val > 0 {
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

                // Draw the knight if this is the current position
                if let Some((cx, cy)) = current_pos {
                    if cx == x && cy == y {
                        painter.circle_filled(
                            square_rect.center(),
                            square_size * 0.35,
                            egui::Color32::RED,
                        );
                    }
                }
            }

            if self.board.is_finished() {
                ui.label("Simulation Finished!");
            }
        });
    }
}
