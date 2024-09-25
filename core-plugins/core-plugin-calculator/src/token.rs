mod operand;
mod operator;

use operand::Operand;
use operator::Operator;

use regex::Regex;

#[derive(Debug)]
pub enum Token<T: num_traits::Num + std::fmt::Debug + Clone> {
    Operand(Operand<T>),
    Operator(Operator),
    None,
}

impl From<&str> for Token<f64> {
    fn from(token_string: &str) -> Self {
        let operand = Operand::from(token_string);
        if let Operand::Number(_) = operand {
            Self::Operand(operand)
        } else {
            let operator = Operator::from(token_string);
            if let Operator::None = operator {
                Self::None
            } else {
                Self::Operator(operator)
            }
        }
    }
}

impl Token<f64> {
    pub fn tokenize(expression: &str) -> Vec<Token<f64>> {
        let pattern =
            Regex::new(r"([-]?\d+(?:\.\d+)?)|([+|\-|/])|(\*{1,2})|([\(|\)|\{|\}])").unwrap();
        pattern
            .captures_iter(expression)
            // .filter(|capture| capture.get(0).is_some())
            .map(|capture| {
                let token = capture
                    .get(0)
                    .expect("Should be valid capture group index")
                    .as_str();
                Token::from(token)
            })
            .collect()
    }
    pub fn convert_infix_to_postfix<T>(tokens: Vec<Token<T>>) -> Vec<Token<T>>
    where
        T: num_traits::Num + std::fmt::Debug + Clone,
    {
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
                        Operator::CloseParanthesis => loop {
                            if let Some(operator_stack_top) = operator_stack.last() {
                                match operator_stack_top {
                                    Operator::OpenParanthesis => {
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
                        Operator::CloseBrace => loop {
                            if let Some(operator_stack_top) = operator_stack.last() {
                                match operator_stack_top {
                                    Operator::OpenBrace => {
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
                                    if let Operator::Exponent = operator_stack_top {
                                        if let Operator::Exponent = operator {
                                            break;
                                        }
                                    }
                                    if operator_stack_top.precedence() >= operator.precedence() {
                                        let top_operator = operator_stack
                                            .pop()
                                            .expect("Should be a valid operator");
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
                _ => {}
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
        postfix_expression
    }
    pub fn evaluate_postfix_expression<T>(postfix_expression: Vec<Token<T>>) -> Result<T, String>
    where
        T: num_traits::Num
            + std::fmt::Debug
            + Clone
            + std::convert::From<f64>
            + num_traits::Pow<T, Output = T>,
    {
        let mut result: Vec<Operand<T>> = Vec::new();
        for token in postfix_expression {
            match token {
                Token::Operand(operand) => result.push(operand.clone()),
                Token::Operator(operator) => {
                    let value_b;
                    if let Some(Operand::Number(value)) = result.pop() {
                        value_b = value
                    } else {
                        break;
                    };
                    let value_a;
                    if let Some(Operand::Number(value)) = result.pop() {
                        value_a = value
                    } else {
                        break;
                    };

                    match operator {
                        Operator::Multiplication => result.push(Operand::Number(value_a * value_b)),
                        Operator::Addition => result.push(Operand::Number(value_a + value_b)),
                        Operator::Division => result.push(Operand::Number(value_a / value_b)),
                        Operator::Subtraction => result.push(Operand::Number(value_a - value_b)),
                        Operator::Exponent => {
                            result.push(Operand::Number(value_a.pow(value_b)));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        if let Some(Operand::Number(value)) = result.pop() {
            Ok(value.clone())
        } else {
            Err("Unable to compute result".to_string())
        }
    }
}
