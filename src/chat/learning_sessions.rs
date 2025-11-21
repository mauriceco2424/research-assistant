use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Default number of questions per learning session before asking to continue or stop.
pub const DEFAULT_QUESTION_COUNT: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningMode {
    Quiz,
    OralExam,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LearningScope {
    Base,
    Categories(Vec<String>),
    Papers(Vec<Uuid>),
    Concepts(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSessionContext {
    pub session_id: Uuid,
    pub base_id: Uuid,
    pub scope: LearningScope,
    pub mode: LearningMode,
    pub status: LearningStatus,
    pub default_question_count: usize,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub regeneration_pointer: Option<String>,
}

impl LearningSessionContext {
    pub fn new(base_id: Uuid, scope: LearningScope, mode: LearningMode) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4(),
            base_id,
            scope,
            mode,
            status: LearningStatus::Pending,
            default_question_count: DEFAULT_QUESTION_COUNT,
            started_at: now,
            updated_at: now,
            regeneration_pointer: None,
        }
    }

    pub fn with_default_question_count(mut self, count: usize) -> Self {
        self.default_question_count = count;
        self
    }

    pub fn activate(mut self) -> Self {
        self.status = LearningStatus::Active;
        self.updated_at = Utc::now();
        self
    }

    pub fn complete(mut self) -> Self {
        self.status = LearningStatus::Completed;
        self.updated_at = Utc::now();
        self
    }

    pub fn cancel(mut self) -> Self {
        self.status = LearningStatus::Cancelled;
        self.updated_at = Utc::now();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningQuestion {
    pub question_id: Uuid,
    pub prompt: String,
    #[serde(default)]
    pub target_concepts: Vec<String>,
    #[serde(default)]
    pub target_papers: Vec<Uuid>,
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub selection_rationale: Option<String>,
    #[serde(default)]
    pub expected_answer_outline: Option<String>,
}

impl LearningQuestion {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            question_id: Uuid::new_v4(),
            prompt: prompt.into(),
            target_concepts: Vec::new(),
            target_papers: Vec::new(),
            difficulty: None,
            selection_rationale: None,
            expected_answer_outline: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearningEvaluationOutcome {
    Correct,
    Partial,
    Incorrect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvaluation {
    pub question_id: Uuid,
    #[serde(default)]
    pub user_answer: Option<String>,
    pub outcome: LearningEvaluationOutcome,
    #[serde(default)]
    pub feedback: Option<String>,
    #[serde(default)]
    pub follow_up_recommendations: Vec<String>,
    #[serde(default)]
    pub kp_update_ref: Option<String>,
    #[serde(default)]
    pub evaluated_at: Option<DateTime<Utc>>,
}

impl LearningEvaluation {
    pub fn new(question_id: Uuid, outcome: LearningEvaluationOutcome) -> Self {
        Self {
            question_id,
            user_answer: None,
            outcome,
            feedback: None,
            follow_up_recommendations: Vec::new(),
            kp_update_ref: None,
            evaluated_at: Some(Utc::now()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSessionSummary {
    pub session_id: Uuid,
    #[serde(default)]
    pub questions: Vec<LearningQuestion>,
    #[serde(default)]
    pub evaluations: Vec<LearningEvaluation>,
    #[serde(default)]
    pub knowledge_profile_changes: Vec<String>,
    #[serde(default)]
    pub recommendations: Vec<String>,
}

