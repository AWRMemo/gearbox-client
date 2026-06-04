use std::fmt;

#[derive(Debug)]
pub enum AiError {
    ModelNotFound(String),
    ModelLoad(String),
    Inference(String),
    PromptTooLong { tokens: usize, limit: usize },
    Parse(String),
    Json(String),
    Fallback(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::ModelNotFound(s) => write!(f, "model not found: {s}"),
            AiError::ModelLoad(s) => write!(f, "model load error: {s}"),
            AiError::Inference(s) => write!(f, "inference error: {s}"),
            AiError::PromptTooLong { tokens, limit } => {
                write!(f, "prompt too long ({tokens} tokens, limit {limit})")
            }
            AiError::Parse(s) => write!(f, "parse error: {s}"),
            AiError::Json(s) => write!(f, "json error: {s}"),
            AiError::Fallback(s) => write!(f, "fallback error: {s}"),
        }
    }
}

impl std::error::Error for AiError {}

impl From<AiError> for String {
    fn from(e: AiError) -> Self {
        e.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_error_display() {
        let e = AiError::Parse("broken json".to_string());
        assert_eq!(e.to_string(), "parse error: broken json");
    }

    #[test]
    fn test_ai_error_into_string() {
        let e: String = AiError::Inference("decode failed".to_string()).into();
        assert!(e.contains("decode failed"));
    }
}
