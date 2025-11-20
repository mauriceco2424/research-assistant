//! AI profile orchestration modules.
//!
//! This namespace keeps long-term profile orchestration isolated from other
//! orchestration features (acquisition, reports, etc.) so we can evolve the
//! profile pipeline independently.

pub mod api;
pub mod defaults;
pub mod governance;
pub mod interview;
pub mod knowledge;
pub mod linking;
pub mod model;
pub mod regenerate;
pub mod render;
pub mod scope;
pub mod service;
pub mod storage;
pub mod summarize;
