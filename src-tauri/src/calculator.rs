use regex::Regex;

#[derive(Debug, Clone, Copy)]
enum Operator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    OpenParanthesis,
    CloseParanthesis,
    OpenBrace,
    CloseBrace,
    None,
}

#[derive(Debug)]
enum Operand<T> {
    Number(T),
    None,
}

#[derive(Debug)]
enum Token<T> {
    Operand(Operand<T>),
    Operator(Operator),
}

impl From<&str> for Operator {
    fn from(operator: &str) -> Self {
        match operator {
            "+" => Self::Addition,
            "-" => Self::Subtraction,
            "*" => Self::Multiplication,
            "/" => Self::Division,
            "(" => Self::OpenParanthesis,
            ")" => Self::CloseParanthesis,
            "{" => Self::OpenBrace,
            "}" => Self::CloseBrace,
            _ => Self::None,
        }
    }
}

impl Operator {
    fn precedence(&self) -> u8 {
        match self {
            Self::Addition => 1,
            Self::Subtraction => 1,
            Self::Multiplication => 2,
            Self::Division => 2,
            _ => 0,
        }
    }
}

impl From<&str> for Operand<i32> {
    fn from(operand: &str) -> Self {
        operand
            .parse::<i32>()
            .map_or(Operand::None, |operand| Operand::Number(operand))
    }
}

impl From<&str> for Token<i32> {
    fn from(token_string: &str) -> Self {
        let operand = Operand::from(token_string);
        if let Operand::Number(_) = operand {
            Self::Operand(operand)
        } else {
            let operator = Operator::from(token_string);
            if let Operator::None = operator {
                panic!("Invalid token")
            } else {
                Self::Operator(operator)
            }
        }
    }
}
fn tokenize(expression: &str) -> Vec<Token<i32>> {
    let pattern = Regex::new(r"(\d+)|([\+|\-|\*|/]{1,2})|([\(|\)|\{|\}])").unwrap();
    pattern
        .captures_iter(expression)
        .map(|capture| {
            let token = capture
                .get(0)
                .expect("Should be valid capture group index")
                .as_str();
            Token::from(token)
        })
        .collect()
}
fn convert_infix_to_postfix<T: std::convert::From<i32> + std::fmt::Debug>(tokens: Vec<Token<T>>) {
    let mut postfix_expression: Vec<Token<T>> = Vec::new();
    let mut operator_stack: Vec<Operator> = Vec::new();
    for token in tokens {
        match &token {
            Token::Operand(_operand) => postfix_expression.push(token),
            Token::Operator(operator) => {
                match operator {
                    Operator::OpenParanthesis | Operator::OpenBrace => {
                        operator_stack.push(operator.clone())
                    }
                    Operator::CloseParanthesis | Operator::CloseBrace => loop {
                        if let Some(operator_stack_top) = operator_stack.last() {
                            match operator_stack_top {
                                Operator::OpenParanthesis | Operator::OpenBrace => {
                                    operator_stack.pop();
                                    break;
                                }
                                _ => postfix_expression.push(Token::Operator(
                                    operator_stack
                                        .pop()
                                        .expect("Vector stack should be non-empty"),
                                )),
                            }
                        } else {
                            break;
                        }
                    },
                    _ => {
                        loop {
                            if let Some(operator_stack_top) = operator_stack.last() {
                                if operator_stack_top.precedence() >= operator.precedence() {
                                    let top_operator =
                                        operator_stack.pop().expect("Should be a valid operator");
                                    postfix_expression.push(Token::Operator(top_operator));
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        operator_stack.push(operator.clone());
                    }
                };
            }
        }
    }

    loop {
        if let Some(_) = operator_stack.last() {
            let top_operator = operator_stack.pop().expect("Should be a valid operator");
            postfix_expression.push(Token::Operator(top_operator));
        } else {
            break;
        }
    }
}

#[tauri::command]
pub fn calculate(input: String) {
    let tokens = tokenize(&input);
    convert_infix_to_postfix(tokens);
}
