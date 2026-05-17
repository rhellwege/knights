#[derive(Debug, Copy, Clone)]
pub struct Color(u32);

impl Color {
    pub fn from_u32(val: u32) -> Self {
        Color(val)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}
