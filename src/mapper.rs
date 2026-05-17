// mapper.rs

/// Trait for mapping between 1D index and 2D coordinates on a grid.
pub trait OneToTwoMapper {
    /// Grid dimensions (width, height).
    fn dimensions(&self) -> (u32, u32);

    /// Convert 1D index `d` to (x,y). Returns None if `d` is out of range.
    fn d2xy(&self, d: u64) -> Option<(u32, u32)>;

    /// Convert (x,y) to 1D index `d`. Returns None if (x,y) is out of range.
    fn xy2d(&self, x: i64, y: i64) -> Option<u64>;
}

//
// Hilbert mapper (square grid: N=2^order)
// Reference: https://en.wikipedia.org/wiki/Hilbert_curve
//
pub struct HilbertMapper {
    order: u32,
    n: u32, // width == height == n
    max_index: u64,
}

impl HilbertMapper {
    pub fn new(order: u32) -> Self {
        let capped = if order > 31 { 31 } else { order };
        let n = 1u32.checked_shl(capped).unwrap_or(u32::MAX);
        let max_index = (n as u64).saturating_mul(n as u64).saturating_sub(1);
        HilbertMapper {
            order: capped,
            n,
            max_index,
        }
    }
}

/// standard Hilbert d -> (x,y) from "A. Hamilton, J. Skilling" style algorithms
fn rot(n: u32, x: &mut u32, y: &mut u32, rx: u32, ry: u32) {
    if ry == 0 {
        if rx == 1 {
            *x = n - 1 - *x;
            *y = n - 1 - *y;
        }
        // swap
        let t = *x;
        *x = *y;
        *y = t;
    }
}

impl OneToTwoMapper for HilbertMapper {
    fn dimensions(&self) -> (u32, u32) {
        (self.n, self.n)
    }

    fn d2xy(&self, d: u64) -> Option<(u32, u32)> {
        if d > self.max_index {
            return None;
        }

        let mut x: u32 = 0;
        let mut y: u32 = 0;
        let mut t = d;
        let mut s = 1u32;

        while s < self.n {
            // Determine quadrant: rx/ry are bits representing the 4 sub-squares
            let rx = 1 & (t / 2) as u32;
            let ry = 1 & (t as u32 ^ rx);

            // Rotate and flip quadrants based on standard Hilbert curve rules
            rot(s, &mut x, &mut y, rx, ry);

            x += s * rx;
            y += s * ry;
            t >>= 2; // Process next 2 bits for the next scale
            s <<= 1;
        }

        Some((x, y))
    }

    fn xy2d(&self, x: i64, y: i64) -> Option<u64> {
        let max = (self.n as i64) - 1;
        if x < 0 || x > max || y < 0 || y > max {
            return None;
        }

        let mut xi = x as u32;
        let mut yi = y as u32;
        let mut d: u64 = 0;
        let mut s = self.n / 2;

        while s > 0 {
            // Identify which quadrant (x, y) falls into at the current scale
            let rx = ((xi & s) > 0) as u32;
            let ry = ((yi & s) > 0) as u32;

            // Map (rx, ry) to the 0-3 index in the Hilbert sequence (using Gray code)
            d += (s as u64) * (s as u64) * (((3 * rx) ^ ry) as u64);

            // Clear current quadrant bits and rotate/flip for the next iteration
            xi &= !s;
            yi &= !s;
            rot(s, &mut xi, &mut yi, rx, ry);
            s >>= 1;
        }

        Some(d)
    }
}

//
// Centered spiral mapper (constructor enforces odd square size)
//
pub struct CenterSpiralMapper {
    n: u32,         // width == height == n (odd)
    max_index: u64, // n*n - 1
    clockwise: bool,
}

impl CenterSpiralMapper {
    pub fn new(mut n: u32, clockwise: bool) -> Self {
        if n == 0 {
            n = 1;
        }
        // Removed forced-odd constraint
        let max_index = (n as u64).saturating_mul(n as u64).saturating_sub(1);
        CenterSpiralMapper {
            n,
            max_index,
            clockwise,
        }
    }
}

impl OneToTwoMapper for CenterSpiralMapper {
    fn dimensions(&self) -> (u32, u32) {
        (self.n, self.n)
    }

    fn d2xy(&self, d: u64) -> Option<(u32, u32)> {
        if d > self.max_index {
            return None;
        }
        // Center is (n-1)/2, n/2 to handle even dimensions (bottom-left of center 2x2)
        let cx_eff = (self.n as i64 - 1) / 2;
        let cy_eff = (self.n as i64) / 2;

        if d == 0 {
            return Some((cx_eff as u32, cy_eff as u32));
        }

        let mut rem = d - 1;
        let mut r: i64 = 1;
        while rem >= (8 * r as u64) {
            rem -= 8 * r as u64;
            r += 1;
        }
        let side = 2 * r;
        let mut x: i64;
        let mut y: i64;
        if self.clockwise {
            x = cx_eff + r;
            y = cy_eff + (r - 1);
        } else {
            x = cx_eff + r;
            y = cy_eff - (r - 1);
        }
        let mut step = rem as i64;

        let take = |dx: i64, dy: i64, s_len: i64, steps: &mut i64, x: &mut i64, y: &mut i64| {
            let t = if *steps < s_len { *steps } else { s_len };
            *x += dx * t;
            *y += dy * t;
            *steps -= t;
        };

        if self.clockwise {
            take(0, -1, side - 1, &mut step, &mut x, &mut y); // up
            take(-1, 0, side, &mut step, &mut x, &mut y); // left
            take(0, 1, side, &mut step, &mut x, &mut y); // down
            take(1, 0, side, &mut step, &mut x, &mut y); // right
        } else {
            take(0, 1, side - 1, &mut step, &mut x, &mut y); // down
            take(-1, 0, side, &mut step, &mut x, &mut y); // left
            take(0, -1, side, &mut step, &mut x, &mut y); // up
            take(1, 0, side, &mut step, &mut x, &mut y); // right
        }

        if x < 0 || x >= self.n as i64 || y < 0 || y >= self.n as i64 {
            return None;
        }
        Some((x as u32, y as u32))
    }

    fn xy2d(&self, x_in: i64, y_in: i64) -> Option<u64> {
        let max = (self.n as i64) - 1;
        if x_in < 0 || x_in > max || y_in < 0 || y_in > max {
            return None;
        }
        let cx_eff = (self.n as i64 - 1) / 2;
        let cy_eff = (self.n as i64) / 2;
        let dx = x_in - cx_eff;
        let dy = y_in - cy_eff;
        if dx == 0 && dy == 0 {
            return Some(0);
        }

        let layer = dx.abs().max(dy.abs()) as u64;
        let before = 1u64 + 4 * (layer - 1) * layer;
        let r = layer as i64;
        let offset: i64;

        if self.clockwise {
            if dx == r && dy < r && dy >= -r {
                offset = (r - 1) - dy;
            } else if dy == -r && dx < r && dx >= -r {
                offset = (2 * r - 1) + (r - dx);
            } else if dx == -r && dy > -r && dy <= r {
                offset = (4 * r - 1) + (dy + r);
            } else {
                offset = (6 * r - 1) + (dx + r);
            }
        } else {
            if dx == r && dy > -r && dy <= r {
                offset = dy + r - 1;
            } else if dy == r && dx < r && dx >= -r {
                offset = (2 * r - 1) + (r - dx);
            } else if dx == -r && dy < r && dy >= -r {
                offset = (4 * r - 1) + (r - dy);
            } else {
                offset = (6 * r - 1) + (dx + r);
            }
        }

        let d = before + (offset as u64);
        if d > self.max_index {
            return None;
        }
        Some(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spiral_even_grid() {
        let mapper = CenterSpiralMapper::new(4, true);
        let (w, h) = mapper.dimensions();
        let n2 = (w as u64) * (h as u64);

        // Ensure every index in 0..15 maps to a unique, valid coordinate
        // and that it roundtrips perfectly.
        let mut seen = std::collections::HashSet::new();
        for d in 0..n2 {
            let (x, y) = mapper
                .d2xy(d)
                .expect("Should be within bounds for n^2 points");
            assert!(
                x < w && y < h,
                "Coordinate ({}, {}) out of bounds for {}x{}",
                x,
                y,
                w,
                h
            );
            assert!(
                seen.insert((x, y)),
                "Duplicate coordinate ({}, {}) found for d={}",
                x,
                y,
                d
            );

            let d2 = mapper.xy2d(x as i64, y as i64).unwrap();
            assert_eq!(d, d2, "Roundtrip failed for d={}", d);
        }
    }

    #[test]
    fn hilbert_specific_points() {
        // Order 1: 2x2 grid
        // 0: (0,0), 1: (0,1), 2: (1,1), 3: (1,0)
        let h1 = HilbertMapper::new(1);
        assert_eq!(h1.d2xy(0).unwrap(), (0, 0));
        assert_eq!(h1.d2xy(1).unwrap(), (0, 1));
        assert_eq!(h1.d2xy(2).unwrap(), (1, 1));
        assert_eq!(h1.d2xy(3).unwrap(), (1, 0));

        // Order 2: 4x4 grid
        let h2 = HilbertMapper::new(2);
        assert_eq!(h2.d2xy(0).unwrap(), (0, 0));
        assert_eq!(h2.d2xy(5).unwrap(), (0, 3));
        assert_eq!(h2.d2xy(10).unwrap(), (3, 3));
        assert_eq!(h2.d2xy(15).unwrap(), (3, 0));
    }

    #[test]
    fn spiral_specific_points() {
        // 3x3 spiral (clockwise)
        // Center (0) is (1,1)
        let s = CenterSpiralMapper::new(3, true);
        assert_eq!(s.d2xy(0).unwrap(), (1, 1)); // Center
        assert_eq!(s.d2xy(1).unwrap(), (2, 1)); // Start of Ring 1
        assert_eq!(s.d2xy(2).unwrap(), (2, 0)); // Move Up
        assert_eq!(s.d2xy(3).unwrap(), (1, 0)); // Move Left
        assert_eq!(s.d2xy(4).unwrap(), (0, 0)); // Move Left
        assert_eq!(s.d2xy(5).unwrap(), (0, 1)); // Move Down
        assert_eq!(s.d2xy(6).unwrap(), (0, 2)); // Move Down
        assert_eq!(s.d2xy(7).unwrap(), (1, 2)); // Move Right
        assert_eq!(s.d2xy(8).unwrap(), (2, 2)); // Move Right
    }

    #[test]
    fn spiral_ccw_specific_points() {
        // 3x3 spiral (counter-clockwise)
        // Center (0) is (1,1)
        let s = CenterSpiralMapper::new(3, false);
        assert_eq!(s.d2xy(0).unwrap(), (1, 1));
        assert_eq!(s.d2xy(1).unwrap(), (2, 1)); // Start of Ring 1
        assert_eq!(s.d2xy(2).unwrap(), (2, 2)); // Move Down
        assert_eq!(s.d2xy(3).unwrap(), (1, 2)); // Move Left
        assert_eq!(s.d2xy(4).unwrap(), (0, 2)); // Move Left
        assert_eq!(s.d2xy(5).unwrap(), (0, 1)); // Move Up
        assert_eq!(s.d2xy(6).unwrap(), (0, 0)); // Move Up
        assert_eq!(s.d2xy(7).unwrap(), (1, 0)); // Move Right
        assert_eq!(s.d2xy(8).unwrap(), (2, 0)); // Move Right
    }
    #[test]
    fn hilbert_roundtrip() {
        let mapper = HilbertMapper::new(3);
        let (w, h) = mapper.dimensions();
        let n2 = (w as u64) * (h as u64);
        for d in 0..n2 {
            let (x, y) = mapper.d2xy(d).unwrap();
            let d2 = mapper.xy2d(x as i64, y as i64).unwrap();
            assert_eq!(d, d2);
        }
    }

    #[test]
    fn spiral_roundtrip_clockwise() {
        let mapper = CenterSpiralMapper::new(7, true);
        let (w, h) = mapper.dimensions();
        let n2 = (w as u64) * (h as u64);
        assert_eq!(mapper.d2xy(0).unwrap(), (w / 2, h / 2));
        for d in 0..n2 {
            let (x, y) = mapper.d2xy(d).unwrap();
            let d2 = mapper.xy2d(x as i64, y as i64).unwrap();
            assert_eq!(d, d2);
        }
    }

    #[test]
    fn spiral_roundtrip_counter() {
        let mapper = CenterSpiralMapper::new(7, false);
        let (w, h) = mapper.dimensions();
        let n2 = (w as u64) * (h as u64);
        for d in 0..n2 {
            let (x, y) = mapper.d2xy(d).unwrap();
            let d2 = mapper.xy2d(x as i64, y as i64).unwrap();
            assert_eq!(d, d2);
        }
    }

    #[test]
    fn out_of_bounds() {
        let h = HilbertMapper::new(3);
        assert!(h.d2xy(h.max_index + 1).is_none());
        assert!(h.xy2d(-1, 0).is_none());

        let s = CenterSpiralMapper::new(5, true);
        assert!(s.d2xy(s.max_index + 1).is_none());
        assert!(s.xy2d(10, 10).is_none());
    }
}
