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

struct Cross {
    color: Color,
}

impl Cross {
    fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Piece for Cross {
    fn name(&self) -> &str {
        "Cross"
    }

    fn color(&self) -> Color {
        self.color
    }

    fn move_offsets(&self) -> &[(i32, i32)] {
        &[(-1, 0), (1, 0), (0, -1), (0, 1)]
    }
}

fn main() -> std::io::Result<()> {
    // let mapper = CenterSpiralMapper::new(1000, false);
    // let mapper = HilbertMapper::new(8);
    let mapper = CenterSpiralMapper::new(400, false);
    // let piece = Knight::new(Color::from_u32(0x00ff00ff));
    let pieces: Vec<Box<dyn Piece>> = vec![
        // Box::new(Knight::new(Color::from_u32(0x0000ffff))),
        Box::new(Cross::new(Color::from_u32(0xff0000ff))),
        Box::new(Knight::new(Color::from_u32(0x00ff00ff))),
        Box::new(Knight::new(Color::from_u32(0xffff00ff))),
        Box::new(Knight::new(Color::from_u32(0x0000ffff))),
        // Box::new(Knight::new(Color::from_u32(0xffff00ff))),
    ];

    let tour = DuelingKnights::new(pieces, mapper);

    // Switch between TerminalApp and EguiApp here
    // let mut app = EguiApp;
    // let mut app = TerminalApp;
    let mut app = PngApp::new("test.png".into());

    app.run(tour)?;

    Ok(())
}
