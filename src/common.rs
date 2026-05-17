#[derive(Debug, Copy, Clone)]
pub struct Color(u32);

impl Color {
    pub fn from_u32(val: u32) -> Self {
        Color(val)
    }
}
