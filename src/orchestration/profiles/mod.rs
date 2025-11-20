//! AI profile orchestration modules.
//!
//! This namespace keeps long-term profile orchestration isolated from other
//! orchestration features (acquisition, reports, etc.) so we can evolve the
//! profile pipeline independently.

pub mod api;
pub mod defaults;
pub mod knowledge;
pub mod linking;
pub mod model;
pub mod storage;
pub mod service;
pub mod summarize;
pub mod scope;
pub mod interview;
pub mod render;
pub mod governance;
pub mod regenerate;
