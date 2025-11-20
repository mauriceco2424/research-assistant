use researchbase::bases::{Base, BaseManager};
use std::env;
use std::path::Path;
use tempfile::TempDir;

pub struct IntegrationHarness {
    workspace: TempDir,
}

const CONTRACT_SPECS: &[&str] = &["specs/003-category-editing/contracts/categories.yaml"];

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
mod history_perf;
mod ingestion_scale;
mod metadata_consent;
mod metadata_offline;
mod metadata_multilang;
mod metadata_refresh;
mod figure_extraction;
mod history_undo;
mod figure_reprocess_consent;
mod categories_proposals;
mod categories_editing;
mod categories_status;
mod categories_narratives;
mod reports_errors;
mod reports_share;
mod reports_configure;
mod profile_knowledge_ready;
mod profile_interview;
mod profile_show_update;
mod profile_governance;
pub mod support;

#[test]
fn contracts_exist() {
    for path in CONTRACT_SPECS {
        assert!(
            Path::new(path).exists(),
            "Expected contract spec to exist: {path}"
        );
    }
}
