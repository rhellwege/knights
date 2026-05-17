/// Trait for mapping between 1D index and 2D coordinates on an N×N grid.
pub trait OneToTwoMapper {
    /// Grid size (N).
    fn size(&self) -> u32;

    /// Convert 1D index `d` to (x,y). Returns None if `d` is out of range.
    fn d2xy(&self, d: u64) -> Option<(u32, u32)>;

    /// Convert (x,y) to 1D index `d`. Returns None if (x,y) is out of range.
    fn xy2d(&self, x: i64, y: i64) -> Option<u64>;
}

//
// Hilbert mapper
//
pub struct HilbertMapper {
    order: u32,
    n: u32,
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

impl OneToTwoMapper for HilbertMapper {
    fn size(&self) -> u32 {
        self.n
    }

    fn d2xy(&self, mut d: u64) -> Option<(u32, u32)> {
        if d > self.max_index {
            return None;
        }

        let mut x: u32 = 0;
        let mut y: u32 = 0;
        let mut s: u32 = 1;

        while s < self.n {
            let t_mod4 = (d & 3) as u32;
            let rx = (t_mod4 >> 1) & 1;
            let ry = (t_mod4 ^ rx) & 1;

            if ry == 0 {
                if rx == 1 {
                    x = s - 1 - x;
                    y = s - 1 - y;
                }
                let tmp = x;
                x = y;
                y = tmp;
            }

            x = x.wrapping_add(s.wrapping_mul(rx));
            y = y.wrapping_add(s.wrapping_mul(ry));

            d >>= 2;
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
        let mut s = self.n >> 1;

        while s > 0 {
            let rx = ((xi & s) > 0) as u32;
            let ry = ((yi & s) > 0) as u32;
            let pair = ((rx as u64) << 1) | (ry as u64);
            d = (d << 2) | pair;

            if ry == 0 {
                if rx == 1 {
                    xi = (self.n - 1) - xi;
                    yi = (self.n - 1) - yi;
                }
                let tmp = xi;
                xi = yi;
                yi = tmp;
            }

            s >>= 1;
        }

        Some(d)
    }
}

//
// Centered spiral mapper: grid size N must be odd. If even n provided, constructor makes it odd.
//
pub struct CenterSpiralMapper {
    n: u32,         // grid size (odd)
    max_index: u64, // n*n - 1
    clockwise: bool,
}

impl CenterSpiralMapper {
    pub fn new(mut n: u32, clockwise: bool) -> Self {
        if n == 0 {
            n = 1;
        }
        if n % 2 == 0 {
            n += 1;
        } // ensure odd
        let max_index = (n as u64).saturating_mul(n as u64).saturating_sub(1);
        CenterSpiralMapper {
            n,
            max_index,
            clockwise,
        }
    }
}

impl OneToTwoMapper for CenterSpiralMapper {
    fn size(&self) -> u32 {
        self.n
    }

    fn d2xy(&self, mut d: u64) -> Option<(u32, u32)> {
        if d > self.max_index {
            return None;
        }

        let center = (self.n / 2) as i64;
        if d == 0 {
            return Some((center as u32, center as u32));
        }

        let mut remaining = d.saturating_sub(1);
        let mut layer: u64 = 1;
        while remaining >= 8 * layer {
            remaining -= 8 * layer;
            layer += 1;
        }

        let r = layer as i64;
        let side_len = 2 * r;
        let mut x = center + r;
        let mut y = center + (r - 1);
        let mut offset = remaining as i64;

        let mut step = |dx: i64, dy: i64, steps: i64, off: &mut i64, x: &mut i64, y: &mut i64| {
            let take = if *off < steps { *off } else { steps };
            *x += dx * take;
            *y += dy * take;
            *off -= take;
        };

        if self.clockwise {
            step(0, -1, side_len, &mut offset, &mut x, &mut y);
            if offset > 0 {
                step(-1, 0, side_len, &mut offset, &mut x, &mut y);
            }
            if offset > 0 {
                step(0, 1, side_len, &mut offset, &mut x, &mut y);
            }
            if offset > 0 {
                step(1, 0, side_len, &mut offset, &mut x, &mut y);
            }
        } else {
            step(0, -1, side_len, &mut offset, &mut x, &mut y);
            if offset > 0 {
                step(1, 0, side_len, &mut offset, &mut x, &mut y);
            }
            if offset > 0 {
                step(0, 1, side_len, &mut offset, &mut x, &mut y);
            }
            if offset > 0 {
                step(-1, 0, side_len, &mut offset, &mut x, &mut y);
            }
        }

        Some((x as u32, y as u32))
    }

    fn xy2d(&self, x_in: i64, y_in: i64) -> Option<u64> {
        let max = (self.n as i64) - 1;
        if x_in < 0 || x_in > max || y_in < 0 || y_in > max {
            return None;
        }

        let xi = x_in;
        let yi = y_in;
        let center = (self.n / 2) as i64;
        let dx = xi - center;
        let dy = yi - center;

        if dx == 0 && dy == 0 {
            return Some(0);
        }

        let layer = dx.abs().max(dy.abs()) as u64;
        let mut d = 1u64 + 4 * layer * (layer - 1);
        let r = layer as i64;
        let side_len = 2 * r;
        let mut offset: i64 = 0;
        let cx = center + r;
        let cy = center + (r - 1);

        if self.clockwise {
            // edge 0: up from (cx,cy) -> (cx, cy - t), t in [0, side_len-1]
            if xi == cx && yi <= cy && yi >= cy - (side_len - 1) {
                offset = cy - yi;
            } else {
                // edge1: left from (cx, cy-side_len)
                let ex1x = cx;
                let ex1y = cy - side_len;
                if yi == ex1y && xi <= ex1x && xi >= ex1x - (side_len - 1) {
                    offset = side_len + (ex1x - xi);
                } else {
                    // edge2: down from (cx-side_len, cy-side_len)
                    let ex2x = cx - side_len;
                    let ex2y = cy - side_len;
                    if xi == ex2x && yi >= ex2y && yi <= ex2y + (side_len - 1) {
                        offset = 2 * side_len + (yi - ex2y);
                    } else {
                        // edge3: right
                        offset = 3 * side_len + (xi - (cx - side_len));
                    }
                }
            }
        } else {
            // counter-clockwise: up, right, down, left
            if xi == cx && yi <= cy && yi >= cy - (side_len - 1) {
                offset = cy - yi;
            } else if yi == cy - side_len && xi >= cx && xi <= cx + (side_len - 1) {
                // right edge
                offset = side_len + (xi - cx);
            } else if xi == cx + (side_len - 1)
                && yi >= cy - side_len
                && yi <= cy + 0 + (side_len - 1)
            {
                // down edge
                offset = 2 * side_len + (yi - (cy - side_len));
            } else {
                // left edge
                offset = 3 * side_len + ((cx + side_len - 1) - xi);
            }
        }

        let max_off = (8 * layer) as i64 - 1;
        if offset < 0 {
            offset = 0;
        }
        if offset > max_off {
            offset = max_off;
        }

        d += offset as u64;
        Some(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hilbert_roundtrip() {
        let mapper = HilbertMapper::new(3);
        let n2 = (mapper.n as u64) * (mapper.n as u64);
        for d in 0..n2 {
            let (x, y) = mapper.d2xy(d).unwrap();
            let d2 = mapper.xy2d(x as i64, y as i64).unwrap();
            assert_eq!(d, d2);
        }
    }

    #[test]
    fn spiral_roundtrip_clockwise() {
        let mapper = CenterSpiralMapper::new(7, true);
        let n2 = (mapper.n as u64) * (mapper.n as u64);
        assert_eq!(mapper.d2xy(0).unwrap(), ((mapper.n / 2, mapper.n / 2)));
        for d in 0..n2 {
            let (x, y) = mapper.d2xy(d).unwrap();
            let d2 = mapper.xy2d(x as i64, y as i64).unwrap();
            assert_eq!(d, d2);
        }
    }

    #[test]
    fn spiral_roundtrip_counter() {
        let mapper = CenterSpiralMapper::new(7, false);
        let n2 = (mapper.n as u64) * (mapper.n as u64);
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
