use anyhow::Result;

use crate::{
    bases::{Base, BaseManager},
    orchestration::profiles::api::{self, KnowledgeSummary},
};

pub struct LearningInterface<'a> {
    manager: &'a BaseManager,
}

impl<'a> LearningInterface<'a> {
    pub fn new(manager: &'a BaseManager) -> Self {
        Self { manager }
    }

    pub fn knowledge_summary(&self, base: &Base) -> Result<KnowledgeSummary> {
        api::get_knowledge_summary(self.manager, base)
    }
}
