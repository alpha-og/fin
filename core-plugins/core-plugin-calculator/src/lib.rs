mod token;

use plugin_api::{Event, EventType, Metadata, Plugin, Response};
use token::Token;

pub struct CalculatorPlugin {
    result: Option<f64>,
    history: Vec<f64>,
}

impl Default for CalculatorPlugin {
    fn default() -> Self {
        Self {
            result: None,
            history: Vec::new(),
        }
    }
}

impl Plugin for CalculatorPlugin {
    fn get_metadata(&self) -> Metadata {
        Metadata {
            name: "Calculator".to_string(),
            description: "A simple calculator".to_string(),
            icon: None,
            url: None,
        }
    }

    fn get_registered_events(&self) -> Vec<EventType> {
        let mut events = Vec::new();
        events.push(EventType::UpdateSearchQuery);
        events
    }
    fn get_response(&self) -> Option<Response> {
        if let Some(result) = self.result {
            Some(Response::F64(result))
        } else {
            None
        }
    }

    fn listen(&mut self, event: &Event) {
        if let Some(event_data) = &event.data {
            if let Ok(result) = Self::calculate(&event_data) {
                self.result = Some(result);
                self.history.push(result);
            }
        }
    }
}

impl CalculatorPlugin {
    pub fn calculate(input: &str) -> Result<f64, String> {
        let tokens = Token::tokenize(input);
        let postfix = Token::convert_infix_to_postfix(tokens);
        Token::evaluate_postfix_expression(postfix)
    }
}
