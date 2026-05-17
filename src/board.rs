use crate::mapper::OneToTwoMapper;
use crate::piece::Piece;

/// Trait for a generic simulation state.
pub trait SimulationState {
    /// Advances the simulation by one step. Returns true if a step was taken.
    fn step(&mut self) -> bool;

    /// Returns true if the simulation can no longer proceed.
    fn is_finished(&self) -> bool;

    /// Returns the dimensions of the board.
    fn mapper_dimensions(&self) -> (u32, u32);

    /// Returns the value (visit order) at the given coordinate.
    fn get_value(&self, x: u32, y: u32) -> u8;

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
    state: Vec<u8>,
    current_d: u64,
    step_count: u8,
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
        let mut best_move: Option<(u64, u8)> = None;

        for (nx, ny) in candidates {
            if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                if let Some(nd) = self.mapper.xy2d(nx as i64, ny as i64) {
                    if self.state[nd as usize] == 0 {
                        match best_move {
                            None => best_move = Some((nd, 0)),
                            Some((best_d, _)) => {
                                if nd < best_d {
                                    best_move = Some((nd, 0));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some((nd, _)) = best_move {
            self.step_count = self.step_count.saturating_add(1);
            self.state[nd as usize] = self.step_count;
            self.current_d = nd;
            if self.step_count == 255 {
                self.is_finished = true;
            }
            true
        } else {
            self.is_finished = true;
            false
        }
    }

    fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn mapper_dimensions(&self) -> (u32, u32) {
        self.mapper.dimensions()
    }

    fn get_value(&self, x: u32, y: u32) -> u8 {
        if let Some(d) = self.get_d(x, y) {
            self.state[d as usize]
        } else {
            0
        }
    }

    fn get_d(&self, x: u32, y: u32) -> Option<u64> {
        self.mapper.xy2d(x as i64, y as i64)
    }

    fn current_pos(&self) -> Option<(u32, u32)> {
        self.mapper.d2xy(self.current_d)
    }
}
