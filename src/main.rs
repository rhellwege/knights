mod app;
mod board;
mod mapper;
mod piece;

use app::{EguiApp, SimulationApp, TerminalApp};
use board::BoardState;
use mapper::CenterSpiralMapper;
use piece::Knight;

fn main() -> std::io::Result<()> {
    let n = 5;
    let mapper = CenterSpiralMapper::new(n, true);
    let piece = Knight;
    let board = BoardState::new(piece, mapper);

    // Switch between TerminalApp and EguiApp here
    let mut app = EguiApp;
    // let mut app = TerminalApp;

    app.run(board)?;

    Ok(())
}
