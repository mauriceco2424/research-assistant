pub mod acquisition;
pub mod bases;
pub mod chat;
pub mod ingestion;
pub mod orchestration;
pub mod reports;

// Re-export commonly used types for convenience.
pub use acquisition::CandidatePaper;
pub use bases::{AppConfig, Base, BaseManager};
pub use orchestration::{AcquisitionBatch, OrchestrationEvent, OrchestrationLog};
