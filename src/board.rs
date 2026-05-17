use crate::mapper::OneToTwoMapper;
use crate::piece::Piece;

pub struct BoardState<P, M>
where
    P: Piece,
    M: OneToTwoMapper,
{
    pub(crate) piece: P,
    pub(crate) mapper: M,
    pub(crate) state: Vec<u8>,
    pub(crate) current_d: u64,
    pub(crate) step: u8,
    pub(crate) is_finished: bool,
}

impl<P, M> BoardState<P, M>
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

        BoardState {
            piece,
            mapper,
            state,
            current_d: 0,
            step: 1,
            is_finished: false,
        }
    }

    /// Advances the simulation by one step. Returns true if a step was taken.
    pub fn step(&mut self) -> bool {
        if self.is_finished {
            return false;
        }

        let (w, h) = self.mapper.dimensions();
        let (x, y) = self.mapper.d2xy(self.current_d).unwrap();
        let candidates = self.piece.candidate_moves(x as i32, y as i32);

        let mut next_pos = None;
        for (nx, ny) in candidates {
            if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                if let Some(nd) = self.mapper.xy2d(nx as i64, ny as i64) {
                    if self.state[nd as usize] == 0 {
                        next_pos = Some(nd);
                        break;
                    }
                }
            }
        }

        if let Some(nd) = next_pos {
            self.step = self.step.saturating_add(1);
            self.state[nd as usize] = self.step;
            self.current_d = nd;
            if self.step == 255 {
                self.is_finished = true;
            }
            true
        } else {
            self.is_finished = true;
            false
        }
    }

    pub fn is_finished(&self) -> bool {
        self.is_finished
    }

    pub fn mapper_dimensions(&self) -> (u32, u32) {
        self.mapper.dimensions()
    }

    pub fn get_value(&self, x: u32, y: u32) -> u8 {
        if let Some(d) = self.get_d(x, y) {
            self.state[d as usize]
        } else {
            0
        }
    }

    pub fn get_d(&self, x: u32, y: u32) -> Option<u64> {
        self.mapper.xy2d(x as i64, y as i64)
    }

    pub fn current_pos(&self) -> Option<(u32, u32)> {
        self.mapper.d2xy(self.current_d)
    }
}
