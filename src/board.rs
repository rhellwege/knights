use std::collections::HashSet;

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

        let (x, y) = self.mapper.d2xy(self.current_d).unwrap();
        let candidates = self
            .piece
            .candidate_moves(x as i32, y as i32)
            .into_iter()
            .filter(|(nx, ny)| self.mapper.in_bounds(*nx, *ny));

        // Find all valid moves and pick the one with the lowest d index (cost)
        let mut best_move: Option<u64> = None;

        for (nx, ny) in candidates {
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

pub struct DuelingKnights<M>
where
    M: OneToTwoMapper,
{
    pieces: Vec<Box<dyn Piece>>,
    mapper: M,
    state: Vec<u32>,
    current_d: u64,
    step_count: u32,
    is_finished: bool,
    // -- helpers --
    lowest_free_d: Vec<u32>,
    contested_cache: Vec<HashSet<(u32, u32)>>, // the set of squares piece[i] contests and where piece[i]
}

impl<M> DuelingKnights<M>
where
    M: OneToTwoMapper,
{
    /// Construct new state with the first piece's move already played
    pub fn new(pieces: Vec<Box<dyn Piece>>, mapper: M) -> Self {
        let mut dk = DuelingKnights {
            pieces,
            mapper,
            state: Vec::new(),
            current_d: 0,
            step_count: 0,
            is_finished: false,
            lowest_free_d: Vec::new(),
            contested_cache: Vec::new(),
        };
        dk.reset();
        dk
    }

    fn current_piece(&self) -> usize {
        self.step_count as usize % self.pieces.len()
    }

    fn update_contested(&mut self, x: u32, y: u32) {
        let piece_index = self.current_piece();
        let candidates = self.pieces[piece_index]
            .candidate_moves(x as i32, y as i32)
            .into_iter()
            .filter(|(nx, ny)| self.mapper.in_bounds(*nx, *ny));

        for (cx, cy) in candidates {
            self.contested_cache[piece_index].insert((cx as u32, cy as u32));
        }
    }

    fn is_valid_move(&self, nd: u32) -> bool {
        // other piece on square
        if self.state[nd as usize] > 0 {
            return false;
        }

        let piece_index = self.current_piece();

        if let Some(pos) = self.mapper.d2xy(nd as u64) {
            for i in 0..self.pieces.len() {
                if i == piece_index {
                    continue;
                }
                if self.contested_cache[i].contains(&pos) {
                    // other piece attacks this square
                    return false;
                }
            }
        } else {
            // out of bounds
            return false;
        };
        true
    }
}

impl<M> SimulationState for DuelingKnights<M>
where
    M: OneToTwoMapper,
{
    fn step(&mut self) -> bool {
        if self.is_finished {
            return false;
        }

        let piece_index = self.current_piece();
        let mut nd = self.lowest_free_d[piece_index];
        while (nd as usize) < self.state.len() {
            if self.is_valid_move(nd) {
                // make the move
                let (x, y) = self.mapper.d2xy(nd as u64).unwrap();
                self.update_contested(x, y);
                self.step_count += 1;
                self.state[nd as usize] = self.step_count;
                self.lowest_free_d[piece_index] = nd + 1;
                self.current_d = nd as u64;
                return true;
            }
            nd += 1;
        }

        self.is_finished = true;
        return false;
    }

    fn reset(&mut self) {
        let (w, h) = self.mapper.dimensions();
        let size = (w as usize) * (h as usize);
        let num_pieces = self.pieces.len();
        self.current_d = 0;
        self.step_count = 0;
        self.is_finished = false;
        self.lowest_free_d = vec![1; num_pieces];
        self.state = vec![0; size];
        // Mark the start at d=0
        self.state[0] = 1;
        let (x_0, y_0) = self.mapper.d2xy(0).unwrap(); // we must surely have a starting point
        self.contested_cache = vec![HashSet::new(); num_pieces];
        self.update_contested(x_0, y_0);
        self.step_count = 1;
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
        self.get_value(x, y)
            .map(|v| self.pieces[v as usize % self.pieces.len()].color())
    }

    fn get_d(&self, x: u32, y: u32) -> Option<u64> {
        self.mapper.xy2d(x as i64, y as i64)
    }

    fn current_pos(&self) -> Option<(u32, u32)> {
        self.mapper.d2xy(self.current_d)
    }
}
