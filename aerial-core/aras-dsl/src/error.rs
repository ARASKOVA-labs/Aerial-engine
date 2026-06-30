#[derive(Debug)]
pub enum ArasError {
    Parse(String),
}

impl std::fmt::Display for ArasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArasError::Parse(e) => write!(f, "Parse Error: {}", e),
        }
    }
}

impl std::error::Error for ArasError {}
