use researchbase::bases::{Base, BaseManager};
use std::env;
use std::path::Path;
use tempfile::TempDir;

pub struct IntegrationHarness {
    workspace: TempDir,
}

impl IntegrationHarness {
    pub fn new() -> Self {
        let workspace = TempDir::new().expect("failed to create temp workspace");
        env::set_var("RESEARCHBASE_HOME", workspace.path());
        Self { workspace }
    }

    pub fn workspace_path(&self) -> &Path {
        self.workspace.path()
    }

    pub fn base_manager(&self) -> BaseManager {
        BaseManager::new().expect("failed to initialize BaseManager for tests")
    }

    pub fn create_base(&self, manager: &mut BaseManager, name: &str) -> Base {
        manager.create_base(name).expect("failed to create base")
    }
}

mod metadata_only;
mod ingestion_progress;
