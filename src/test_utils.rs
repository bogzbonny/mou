/// A fake LLM that always returns the same message, for testing purposes.
#[derive(Debug, Clone)]
pub struct NoopLLM {
    response: String,
}

impl Default for NoopLLM {
    fn default() -> Self {
        Self {
            response: "Kwek".to_string(),
        }
    }
}

impl NoopLLM {
    #[must_use]
    pub fn new(response: String) -> Self {
        Self { response }
    }

    pub fn with_response(&mut self, response: String) -> &mut Self {
        self.response = response;
        self
    }
}
