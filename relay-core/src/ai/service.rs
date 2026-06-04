pub use crate::types::{ConnectionCandidate, ConnectionSuggestion, EnrichmentOutput, Highlight};

pub trait AIService: Send + Sync {
    /// Enrich a single highlight. Returns EnrichmentOutput or a descriptive error.
    /// Implementations must NOT panic. All errors are recoverable.
    ///
    /// `candidates` should be pre-filtered by the caller (e.g., top-k from vector search).
    /// Passing all existing highlights is an anti-pattern and will degrade performance.
    fn enrich(
        &self,
        highlight: &Highlight,
        candidates: &[ConnectionCandidate],
    ) -> Result<EnrichmentOutput, String>;
}
