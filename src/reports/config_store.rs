use crate::bases::{Base, BaseManager};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = "reports/config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfigDefaults {
    pub include_figures: bool,
    pub include_visualizations: Vec<String>,
    pub excluded_assets: Vec<String>,
    pub consent_refresh_days: u32,
}

impl Default for ReportConfigDefaults {
    fn default() -> Self {
        Self {
            include_figures: false,
            include_visualizations: Vec::new(),
            excluded_assets: Vec::new(),
            consent_refresh_days: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportConfigOverrides {
    pub include_figures: Option<bool>,
    pub include_visualizations: Option<Vec<String>>,
    pub excluded_assets: Option<Vec<String>>,
}

pub struct ReportConfigStore<'a> {
    manager: &'a BaseManager,
    base: Base,
    path: PathBuf,
}

impl<'a> ReportConfigStore<'a> {
    pub fn new(manager: &'a BaseManager, base: &Base) -> Self {
        let path = base.ai_layer_path.join(CONFIG_FILE_NAME);
        Self {
            manager,
            base: base.clone(),
            path,
        }
    }

    pub fn load_defaults(&self) -> Result<ReportConfigDefaults> {
        if !self.path.exists() {
            return Ok(ReportConfigDefaults::default());
        }
        let raw = fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read report config {}", self.path.display()))?;
        let cfg: ReportConfigDefaults = serde_json::from_str(&raw)
            .with_context(|| format!("Invalid report config {}", self.path.display()))?;
        Ok(cfg)
    }

    pub fn save_defaults(&self, defaults: &ReportConfigDefaults) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let data = serde_json::to_string_pretty(defaults)?;
        fs::write(&self.path, data)
            .with_context(|| format!("Failed to persist {}", self.path.display()))
    }

    pub fn apply_overrides(
        &self,
        defaults: &ReportConfigDefaults,
        overrides: &ReportConfigOverrides,
    ) -> ReportConfigDefaults {
        ReportConfigDefaults {
            include_figures: overrides
                .include_figures
                .unwrap_or(defaults.include_figures),
            include_visualizations: overrides
                .include_visualizations
                .clone()
                .unwrap_or_else(|| defaults.include_visualizations.clone()),
            excluded_assets: overrides
                .excluded_assets
                .clone()
                .unwrap_or_else(|| defaults.excluded_assets.clone()),
            consent_refresh_days: defaults.consent_refresh_days,
        }
    }

    pub fn base(&self) -> &Base {
        &self.base
    }

    pub fn manager(&self) -> &'a BaseManager {
        self.manager
    }
}
