pub mod citations;
pub mod compile;
pub mod drafting;
pub mod outline;
pub mod project;
pub mod style;
pub mod undo;

pub use style::{
    RemoteStyleConsent, StyleInterviewOutcome, StyleInterviewQuestion, StyleInterviewQuestionId,
    StyleInterviewResponse, StyleModelIngestionResult, StyleModelSource,
};

pub type WritingResult<T> = anyhow::Result<T>;

pub fn not_implemented(feature: &str) -> anyhow::Error {
    anyhow::anyhow!("Writing module placeholder hit for {feature}")
}
