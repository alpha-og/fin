mod token;

use std::sync::{Arc, Mutex};

use plugin_api::Plugin;
use token::Token;

#[derive(Clone)]
pub struct CalculatorPlugin {
    result: Option<f64>,
    history: Vec<f64>,
    client_state: Arc<Mutex<plugin_api::ClientState>>,
}

#[derive(Clone, Debug, serde::Serialize)]
struct CalculatorResult {
    result: f64,
}

impl plugin_api::SearchResultType for CalculatorResult {
    fn clone_box(&self) -> Box<dyn plugin_api::SearchResultType> {
        Box::new(self.clone())
    }
}

impl Default for CalculatorPlugin {
    fn default() -> Self {
        Self {
            result: None,
            history: Vec::new(),
            client_state: Arc::new(Mutex::new(plugin_api::ClientState::default())),
        }
    }
}

impl Plugin for CalculatorPlugin {
    fn init(&mut self, client_state_arc: Arc<Mutex<plugin_api::ClientState>>) {
        self.client_state = client_state_arc;
        println!("Calculator plugin initialized!");
    }
    fn start(&mut self) {
        let mut client_state = self
            .client_state
            .lock()
            .expect("Failed to lock client state");

        let query = client_state.get_search_query();
        let result = CalculatorPlugin::calculate(query);
        if let Ok(result) = result {
            if let Some(existing_result) = self.result {
                if existing_result == result {
                    return;
                }
            }
            self.result = Some(result);
        } else {
            self.result = None;
        }
        if let Some(result) = self.result {
            let existing_results = client_state.get_search_results();
            let mut new_results = Vec::new();
            for result in existing_results {
                new_results.push(result);
            }
            new_results.push(plugin_api::SearchResult::Single(Box::new(
                CalculatorResult { result },
            )));
            client_state.update_search_results(new_results);
        }
    }
    fn get_metadata(&self) -> plugin_api::Metadata {
        plugin_api::Metadata {
            name: "Calculator".to_string(),
            description: "A calculator plugin".to_string(),
            icon: None,
            url: None,
        }
    }
    fn destroy(&mut self) {
        println!("Calculator plugin destroyed!");
    }
    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(self.clone())
    }
}

impl CalculatorPlugin {
    pub fn calculate(input: &str) -> Result<f64, String> {
        let tokens = Token::tokenize(input);
        let postfix = Token::convert_infix_to_postfix(tokens);
        Token::evaluate_postfix_expression(postfix)
    }
}
