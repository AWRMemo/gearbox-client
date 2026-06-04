pub use relay_core::ai::*;
pub use relay_core::types::{
    ConnectionCandidate, ConnectionSuggestion, EnrichmentOutput, Highlight,
};
pub mod embedding;
pub mod embedding_model_manager;
pub mod error;
pub mod llama_service;
pub mod model_manager;
pub mod quality_monitor;

// Legacy fallback.rs removed — src-tauri/ai/fallback.rs was a duplicate of relay-core/src/ai/fallback.rs.
// Use relay_core::ai::fallback::FallbackService instead.
