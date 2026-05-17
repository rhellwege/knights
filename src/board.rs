use crate::common::Color;
use crate::mapper::OneToTwoMapper;
use crate::piece::Piece;

/// Trait for a generic simulation state.
pub trait SimulationState {
    /// Advances the simulation by one step. Returns true if a step was taken.
    fn step(&mut self) -> bool;

    /// Resets the simulation to its initial state.
    fn reset(&mut self);

    /// Returns true if the simulation can no longer proceed.
    fn is_finished(&self) -> bool;

    /// Returns the dimensions of the board.
    fn mapper_dimensions(&self) -> (u32, u32);

    /// Returns the value (visit order) at the given coordinate.
    fn get_value(&self, x: u32, y: u32) -> Option<u32>;

    /// Returns the rgba value as a u32 at the given coordinate.
    fn get_color(&self, x: u32, y: u32) -> Option<Color>;

    /// Returns the 1D mapping index (d) for the given coordinate.
    fn get_d(&self, x: u32, y: u32) -> Option<u64>;

    /// Returns the current position of the piece.
    fn current_pos(&self) -> Option<(u32, u32)>;
}

/// A specific simulation: a greedy knight's tour that prioritizes moves with the lowest d index.
pub struct GreedyKnightsTour<P, M>
where
    P: Piece,
    M: OneToTwoMapper,
{
    piece: P,
    mapper: M,
    state: Vec<u32>,
    current_d: u64,
    step_count: u32,
    is_finished: bool,
}

impl<P, M> GreedyKnightsTour<P, M>
where
    P: Piece,
    M: OneToTwoMapper,
{
    pub fn new(piece: P, mapper: M) -> Self {
        let (w, h) = mapper.dimensions();
        let size = (w as usize) * (h as usize);
        let mut state = vec![0; size];

        // Mark the start at d=0
        state[0] = 1;

        GreedyKnightsTour {
            piece,
            mapper,
            state,
            current_d: 0,
            step_count: 1,
            is_finished: false,
        }
    }
}

impl<P, M> SimulationState for GreedyKnightsTour<P, M>
where
    P: Piece,
    M: OneToTwoMapper,
{
    fn step(&mut self) -> bool {
        if self.is_finished {
            return false;
        }

        let (w, h) = self.mapper.dimensions();
        let (x, y) = self.mapper.d2xy(self.current_d).unwrap();
        let candidates = self.piece.candidate_moves(x as i32, y as i32);

        // Find all valid moves and pick the one with the lowest d index (cost)
        let mut best_move: Option<u64> = None;

        for (nx, ny) in candidates {
            if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                if let Some(nd) = self.mapper.xy2d(nx as i64, ny as i64) {
                    if self.state[nd as usize] == 0 {
                        match best_move {
                            None => best_move = Some(nd),
                            Some(best_d) => {
                                if nd < best_d {
                                    best_move = Some(nd);
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(nd) = best_move {
            self.step_count = self.step_count.saturating_add(1);
            self.state[nd as usize] = self.step_count;
            self.current_d = nd;

            // Check if board is full
            if self.step_count as usize == self.state.len() {
                self.is_finished = true;
            }

            true
        } else {
            self.is_finished = true;
            false
        }
    }

    fn reset(&mut self) {
        // Clear board
        for val in self.state.iter_mut() {
            *val = 0;
        }

        // Reset to initial state
        self.state[0] = 1;
        self.current_d = 0;
        self.step_count = 1;
        self.is_finished = false;
    }

    fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn mapper_dimensions(&self) -> (u32, u32) {
        self.mapper.dimensions()
    }

    fn get_value(&self, x: u32, y: u32) -> Option<u32> {
        let v = self.state[self.get_d(x, y)? as usize];
        if v == 0 { None } else { Some(v as u32 - 1) }
    }

    fn get_color(&self, x: u32, y: u32) -> Option<Color> {
        self.get_value(x, y).map(|_| self.piece.color())
    }

    fn get_d(&self, x: u32, y: u32) -> Option<u64> {
        self.mapper.xy2d(x as i64, y as i64)
    }

    fn current_pos(&self) -> Option<(u32, u32)> {
        self.mapper.d2xy(self.current_d)
    }
}
