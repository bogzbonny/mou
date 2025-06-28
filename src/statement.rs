use {
    std::fmt::Display,
    std::ops::Deref,
    std::ops::DerefMut,
    swiftide::indexing::{IndexingStream, Node},
};

#[derive(Debug, Clone)]
pub struct Statement {
    pub content: String,
}

impl Statement {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Statements(pub Vec<Statement>);

impl Deref for Statements {
    type Target = Vec<Statement>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Statements {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Statement>> for Statements {
    fn from(statements: Vec<Statement>) -> Self {
        Self(statements)
    }
}

impl From<Statements> for IndexingStream {
    fn from(val: Statements) -> Self {
        let nodes: Vec<Node> = val
            .iter()
            .map(|statement| Node::new(statement.to_string()))
            .collect();
        IndexingStream::from_nodes(nodes)
    }
}
