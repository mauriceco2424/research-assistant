//! Guided interview orchestration for AI profiles.
//!
//! A lightweight flow manager coordinates the phases of an interview so
//! services can capture answers deterministically before writing them to disk.

use super::model::ProfileType;
use super::service::ProfileFieldChange;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterviewPhase {
    Prompting,
    Reviewing,
    Completed,
}

#[derive(Debug, Clone)]
pub struct InterviewFlow {
    profile_type: ProfileType,
    state: InterviewPhase,
    responses: Vec<ProfileFieldChange>,
}

impl InterviewFlow {
    pub fn new(profile_type: ProfileType) -> Self {
        Self {
            profile_type,
            state: InterviewPhase::Prompting,
            responses: Vec::new(),
        }
    }

    pub fn record_response(&mut self, change: ProfileFieldChange) {
        self.responses.push(change);
        if self.state == InterviewPhase::Prompting {
            self.state = InterviewPhase::Reviewing;
        }
    }

    pub fn advance(&mut self) {
        self.state = match self.state {
            InterviewPhase::Prompting => InterviewPhase::Reviewing,
            InterviewPhase::Reviewing => InterviewPhase::Completed,
            InterviewPhase::Completed => InterviewPhase::Completed,
        };
    }

    pub fn finalize(mut self) -> Vec<ProfileFieldChange> {
        self.state = InterviewPhase::Completed;
        self.responses
    }

    pub fn profile_type(&self) -> ProfileType {
        self.profile_type
    }
}
