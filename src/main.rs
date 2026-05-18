mod app;
mod board;
mod common;
mod mapper;
mod piece;

use app::{EguiApp, SimulationApp};
use board::GreedyKnightsTour;
use mapper::CenterHilbertMapper;
use piece::Knight;

use crate::{
    board::DuelingKnights,
    common::Color,
    mapper::{CenterSpiralMapper, HilbertMapper},
    piece::Piece,
};

fn main() -> std::io::Result<()> {
    let mapper = CenterSpiralMapper::new(333, false);
    // let mapper = HilbertMapper::new(5);
    // let piece = Knight::new(Color::from_u32(0x00ff00ff));
    let pieces: Vec<Box<dyn Piece>> = vec![
        Box::new(Knight::new(Color::from_u32(0x000000ff))),
        Box::new(Knight::new(Color::from_u32(0xff0000ff))),
        Box::new(Knight::new(Color::from_u32(0x00ff00ff))),
    ];

    let tour = DuelingKnights::new(pieces, mapper);

    // Switch between TerminalApp and EguiApp here
    let mut app = EguiApp;
    // let mut app = TerminalApp;

    app.run(tour)?;

    Ok(())
}
