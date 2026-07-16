#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayType {
    None = 0,
    CurrentTime = 1,
    WorldTime = 6,
}
impl DisplayType {
    pub fn from_int(v: i32) -> Self {
        match v { 1 => Self::CurrentTime, 6 => Self::WorldTime, _ => Self::None }
    }
    pub fn to_int(self) -> i32 { self as i32 }
}
