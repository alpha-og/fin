mod token;

use token::Token;

use plugin_api::Plugin;

pub struct CalculatorPlugin {}

impl Plugin for CalculatorPlugin {
    fn init(&self) {
        println!("Initialised");
    }
    fn execute(
        &self,
        sender: std::sync::mpsc::Sender<Result<f64, String>>,
        _fn_name: &str,
        args: Vec<String>,
    ) {
        let _ = sender.send(Self::calculate(&args[0]));
    }
    fn destroy(&self) {
        println!("Destroyed");
    }
}

impl CalculatorPlugin {
    pub fn calculate(input: &str) -> Result<f64, String> {
        let tokens = Token::tokenize(input);
        let postfix = Token::convert_infix_to_postfix(tokens);
        Token::evaluate_postfix_expression(postfix)
    }
}
