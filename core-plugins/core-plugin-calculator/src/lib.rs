mod token;

use plugin_api::{EventBus, Plugin, PluginEventPayload};
use token::Token;

#[derive(Default)]
pub struct CalculatorState {
    result: Option<f64>,
    history: Vec<f64>,
}

pub struct CalculatorPlugin {
    pub state: CalculatorState,
    event_bus: Option<EventBus>,
    active: bool,
}

impl Plugin for CalculatorPlugin {
    fn init(&mut self, event_bus: EventBus) {
        self.event_bus = Some(event_bus);
        self.active = true;
        println!("Initialised");
        loop {
            if !self.active {
                self.destroy();
                break;
            }
            self.listen("query");
            if let Some(result) = self.state.result {
                self.emit("response", PluginEventPayload::Single(result.to_string()));
            }
        }
    }
    fn emit(&mut self, event_name: &str, payload: PluginEventPayload<String>) {
        if let Some(event_bus_arc) = &self.event_bus {
            let mut event_bus = event_bus_arc.lock().expect("Should not be poisoned");
            event_bus.insert(event_name.into(), payload);
        }
    }

    fn listen(&mut self, event_name: &str) {
        if let Some(event_bus_arc) = self.event_bus.clone() {
            let event_bus = event_bus_arc.lock().expect("Should not be poisoned");
            if let Some(payload) = event_bus.get(event_name) {
                let result = self.calculate(payload);
                if let Ok(result) = result {
                    self.state.result = Some(result);
                }
            }
        };
    }

    fn destroy(&self) {
        println!("Destroyed");
    }
}

impl Default for CalculatorPlugin {
    fn default() -> Self {
        Self {
            state: CalculatorState::default(),
            event_bus: None,
            active: false,
        }
    }
}

impl CalculatorPlugin {
    pub fn calculate(&self, payload: &PluginEventPayload<String>) -> Result<f64, String> {
        if let PluginEventPayload::Single(query) = payload {
            let tokens = Token::tokenize(query);
            let postfix = Token::convert_infix_to_postfix(tokens);
            Token::evaluate_postfix_expression(postfix)
        } else {
            Err(String::from("Event payload type 'Single' not found"))
        }
    }
}
