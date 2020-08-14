
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Motion {
    Up,
    Down,
    Left,
    Right,
    RightEnd,
    First,
    FirstOccupied,
    Last,
    High,
    Middle,
    Low,
    Bracket,
}
