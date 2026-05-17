/// Trait for pieces with static movement patterns.
pub trait Piece {
    /// Returns the name of the piece.
    fn name(&self) -> &str;

    /// Returns the relative (dx, dy) offsets this piece can move.
    fn move_offsets(&self) -> &[(i32, i32)];

    /// Generates absolute candidate coordinates from a starting point.
    /// Note: Bounds checking should be handled by the board/engine layer.
    fn candidate_moves(&self, from_x: i32, from_y: i32) -> Vec<(i32, i32)> {
        self.move_offsets()
            .iter()
            .map(|&(dx, dy)| (from_x + dx, from_y + dy))
            .collect()
    }
}

pub struct Knight;

impl Piece for Knight {
    fn name(&self) -> &str {
        "Knight"
    }

    fn move_offsets(&self) -> &[(i32, i32)] {
        static OFFSETS: [(i32, i32); 8] = [
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ];
        &OFFSETS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knight_moves() {
        let knight = Knight;
        let moves = knight.candidate_moves(0, 0);
        assert_eq!(moves.len(), 8);
        assert!(moves.contains(&(1, 2)));
        assert!(moves.contains(&(-2, 1)));
    }
}
