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
    app::PngApp,
    board::DuelingKnights,
    common::Color,
    mapper::{CenterSpiralMapper, HilbertMapper},
    piece::Piece,
};

fn main() -> std::io::Result<()> {
    let mapper = CenterSpiralMapper::new(10000, false);
    // let mapper = HilbertMapper::new(8);
    // let mapper = CenterHilbertMapper::new(7);
    // let piece = Knight::new(Color::from_u32(0x00ff00ff));
    let pieces: Vec<Box<dyn Piece>> = vec![
        Box::new(Knight::new(Color::from_u32(0x000000ff))),
        Box::new(Knight::new(Color::from_u32(0xff0000ff))),
        // Box::new(Knight::new(Color::from_u32(0x00ff00ff))),
        // Box::new(Knight::new(Color::from_u32(0x0000ffff))),
    ];

    let tour = DuelingKnights::new(pieces, mapper);

    // Switch between TerminalApp and EguiApp here
    // let mut app = EguiApp;
    // let mut app = TerminalApp;
    let mut app = PngApp::new("test.png".into());

    app.run(tour)?;

    Ok(())
}
