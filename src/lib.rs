pub mod bases;
pub mod chat;
pub mod ingestion;
pub mod acquisition;
pub mod reports;
pub mod orchestration;

// Re-export commonly used types for convenience.
pub use bases::{AppConfig, Base, BaseManager};
pub use acquisition::{AcquisitionBatch, CandidatePaper};
pub use orchestration::{OrchestrationEvent, OrchestrationLog};
