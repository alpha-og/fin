mod token;

use plugin_api::{Plugin, PluginState};
use token::Token;

#[derive(Default)]
pub struct CalculatorState {
    result: f64,
    history: Vec<f64>,
}

impl PluginState for CalculatorState {
    fn init() -> Box<dyn PluginState> {
        Box::new(Self::default())
    }
    fn get(&self) -> &dyn PluginState {
        self
    }
}

pub struct CalculatorPlugin {
    pub state: Box<dyn PluginState>,
}

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
