
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Motion {
    Up,
    Down,
    Left,
    Right,
    First,
    Last,
    FirstOccupied,
    High,
    Middle,
    Low,
    SemanticLeft,
    SemanticRight,
    SemanticRightEnd,
    WordLeft,
    WordRight,
    WordRightEnd,
    Bracket,
}
