#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Exponent,
    OpenParanthesis,
    CloseParanthesis,
    OpenBrace,
    CloseBrace,
    None,
}

impl From<&str> for Operator {
    fn from(operator: &str) -> Self {
        match operator {
            "+" => Self::Addition,
            "-" => Self::Subtraction,
            "*" => Self::Multiplication,
            "/" => Self::Division,
            "**" => Self::Exponent,
            "(" => Self::OpenParanthesis,
            ")" => Self::CloseParanthesis,
            "{" => Self::OpenBrace,
            "}" => Self::CloseBrace,
            _ => Self::None,
        }
    }
}

impl Operator {
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Addition => 1,
            Self::Subtraction => 1,
            Self::Multiplication => 2,
            Self::Division => 2,
            Self::Exponent => 3,
            _ => 0,
        }
    }
}
