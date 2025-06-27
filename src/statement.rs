pub use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Statement {
    pub content: String,
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}
