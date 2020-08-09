#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Quantity {
    Number(usize),
    Page(usize),
    Line(usize),
    Word(usize),
    All,
}

impl Default for Quantity {
    fn default() -> Quantity {
        Quantity::Number(1)
    }
}
