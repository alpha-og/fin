#[derive(Debug, Clone, Copy)]
pub enum Operand<T: num_traits::Num + std::fmt::Debug + Clone> {
    Number(T),
    None,
}

impl From<&str> for Operand<f64> {
    fn from(operand: &str) -> Self {
        operand
            .parse::<f64>()
            .map_or(Operand::None, |operand| Operand::Number(operand))
    }
}
