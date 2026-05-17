mod app;
mod board;
mod mapper;
mod piece;

use app::{EguiApp, SimulationApp, TerminalApp};
use board::{GreedyKnightsTour, SimulationState};
use mapper::CenterSpiralMapper;
use piece::Knight;

use crate::mapper::HilbertMapper;

fn main() -> std::io::Result<()> {
    let n = 4;
    // let mapper = CenterSpiralMapper::new(n, true);
    let mapper = HilbertMapper::new(n);
    let piece = Knight;
    let tour = GreedyKnightsTour::new(piece, mapper);

    // Switch between TerminalApp and EguiApp here
    let mut app = EguiApp;
    // let mut app = TerminalApp;

    app.run(tour)?;

    Ok(())
}
