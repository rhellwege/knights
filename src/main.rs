mod app;
mod board;
mod common;
mod mapper;
mod piece;

use app::{EguiApp, SimulationApp};
use board::GreedyKnightsTour;
use mapper::CenterHilbertMapper;
use piece::Knight;

use crate::{common::Color, mapper::CenterSpiralMapper};

fn main() -> std::io::Result<()> {
    let mapper = CenterSpiralMapper::new(7, false);
    let piece = Knight::new(Color::from_u32(0xffff00ff));
    let tour = GreedyKnightsTour::new(piece, mapper);

    // Switch between TerminalApp and EguiApp here
    let mut app = EguiApp;
    // let mut app = TerminalApp;

    app.run(tour)?;

    Ok(())
}
