pub mod acquisition;
pub mod api;
pub mod bases;
pub mod chat;
pub mod ingestion;
pub mod models;
pub mod orchestration;
pub mod profiles;
pub mod reports;
pub mod services;
pub mod storage;
pub mod writing;

// Re-export commonly used types for convenience.
pub use acquisition::CandidatePaper;
pub use bases::{AppConfig, Base, BaseManager};
pub use orchestration::{AcquisitionBatch, OrchestrationEvent, OrchestrationLog};
