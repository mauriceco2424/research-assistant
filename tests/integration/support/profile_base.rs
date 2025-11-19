use super::super::IntegrationHarness;
use anyhow::Result;
use researchbase::bases::{Base, BaseManager, ProfileLayout};
use researchbase::chat::ChatSession;
use std::path::PathBuf;

pub struct ProfileBaseFixture {
    harness: IntegrationHarness,
    pub manager: BaseManager,
    pub base: Base,
}

impl ProfileBaseFixture {
    pub fn new(base_name: &str) -> Self {
        let harness = IntegrationHarness::new();
        let mut manager = harness.base_manager();
        let base = harness.create_base(&mut manager, base_name);
        Self {
            harness,
            manager,
            base,
        }
    }

    pub fn workspace(&self) -> PathBuf {
        self.harness.workspace_path().to_path_buf()
    }

    pub fn profile_json_path(&self, profile_type: &str) -> PathBuf {
        ProfileLayout::new(&self.base).profile_json(profile_type)
    }

    pub fn profile_html_path(&self, profile_type: &str) -> PathBuf {
        ProfileLayout::new(&self.base).profile_html(profile_type)
    }

    pub fn chat_session(&self) -> Result<ChatSession> {
        let mut chat = ChatSession::new()?;
        chat.select_base(&self.base.id)?;
        Ok(chat)
    }
}
