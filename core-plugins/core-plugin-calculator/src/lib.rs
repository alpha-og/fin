use std::ffi::{CStr, CString};

use regex::Regex;

#[derive(Debug, Clone, Copy)]
enum Operator {
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

#[derive(Debug, Clone, Copy)]
enum Operand<T: num_traits::Num + std::fmt::Debug + Clone> {
    Number(T),
    None,
}

#[derive(Debug)]
enum Token<T: num_traits::Num + std::fmt::Debug + Clone> {
    Operand(Operand<T>),
    Operator(Operator),
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
    fn precedence(&self) -> u8 {
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

impl From<&str> for Operand<f64> {
    fn from(operand: &str) -> Self {
        operand
            .parse::<f64>()
            .map_or(Operand::None, |operand| Operand::Number(operand))
    }
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
fn tokenize(expression: &str) -> Vec<Token<f64>> {
    let pattern = Regex::new(r"([-]?\d+(?:\.\d+)?)|([+|\-|/])|(\*{1,2})|([\(|\)|\{|\}])").unwrap();
    pattern
        .captures_iter(expression)
        // .filter(|capture| capture.get(0).is_some())
        .map(|capture| {
            let token = capture
                .get(0)
                .expect("Should be valid capture group index")
                .as_str();
            dbg!(&token);
            Token::from(token)
        })
        .collect()
}
fn convert_infix_to_postfix<T>(tokens: Vec<Token<T>>) -> Vec<Token<T>>
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
fn evaluate_postfix_expression<T>(postfix_expression: Vec<Token<T>>) -> Result<T, String>
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
                let mut value_b: T = 0.0.into();
                if let Some(Operand::Number(value)) = result.pop() {
                    value_b = value
                } else {
                    break;
                };
                let mut value_a: T = 0.0.into();
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

#[repr(C)]
pub struct ResultWrapper {
    pub ok: bool,
    pub value: f64,
    pub error: *const libc::c_char,
}

#[no_mangle]
pub extern "C" fn calculate(input: *const libc::c_char) -> ResultWrapper {
    let input = unsafe { CStr::from_ptr(input) }.to_string_lossy();
    let tokens = tokenize(&String::from(input));
    let postfix = convert_infix_to_postfix(tokens);
    match evaluate_postfix_expression(postfix) {
        Ok(value) => ResultWrapper {
            ok: true,
            value,
            error: std::ptr::null(),
        },
        Err(err) => ResultWrapper {
            ok: false,
            value: 0.0,
            error: CString::new(err).unwrap().into_raw(),
        },
    }
}
#[no_mangle]
pub extern "C" fn free_error_string(error: *const libc::c_char) {
    if error.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(error as *mut libc::c_char);
    }
}
