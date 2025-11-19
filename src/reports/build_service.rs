use crate::bases::{Base, BaseManager};
use crate::orchestration::ReportProgressTracker;
use crate::reports::config_store::ReportConfigStore;
use crate::reports::manifest::{ReportBuildRequest, ReportManifest, ReportOutputEntry};
use anyhow::{bail, Result};
use chrono::Utc;
use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ReportBuildResult {
    pub manifest: ReportManifest,
    pub category_path: PathBuf,
    pub global_path: PathBuf,
}

pub struct ReportBuildService<'a> {
    manager: &'a BaseManager,
    queue: VecDeque<ReportBuildRequest>,
}

impl<'a> ReportBuildService<'a> {
    pub fn new(manager: &'a BaseManager) -> Self {
        Self {
            manager,
            queue: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, request: ReportBuildRequest) {
        self.queue.push_back(request);
    }

    pub fn next(&mut self) -> Option<ReportBuildRequest> {
        self.queue.pop_front()
    }

    pub fn run_now(&self, base: &Base, request: &ReportBuildRequest) -> Result<ReportBuildResult> {
        let config_store = ReportConfigStore::new(self.manager, base);
        let defaults = config_store.load_defaults()?;
        let effective = config_store.apply_overrides(&defaults, &request.overrides);
        let scope_label = format!("{:?}", request.scope.mode);
        let mut progress = ReportProgressTracker::new(self.manager, base);
        progress.start(&format!("reports {}", scope_label))?;
        progress.update("Loaded configuration defaults", 5)?;
        let mut manifest =
            ReportManifest::new(base, request.request_id, format!("{:?}", effective));
        manifest.started_at = Utc::now();
        // Placeholder rendering paths — actual rendering occurs in later tasks.
        let cat_path = base
            .user_layer_path
            .join("reports")
            .join("category_report.html");
        let global_path = base
            .user_layer_path
            .join("reports")
            .join("global_report.html");
        manifest.add_output(ReportOutputEntry {
            path: cat_path.clone(),
            scope: "category".into(),
            hash: String::new(),
            kind: "html".into(),
        });
        manifest.add_output(ReportOutputEntry {
            path: global_path.clone(),
            scope: "global".into(),
            hash: String::new(),
            kind: "html".into(),
        });
        progress.update("HTML outputs prepared", 70)?;
        manifest.completed_at = Utc::now();
        manifest.duration_ms = 0;
        progress.finish("Report build completed")?;
        Ok(ReportBuildResult {
            manifest,
            category_path: cat_path,
            global_path,
        })
    }

    pub fn ensure_singleton(&self, running: bool) -> Result<()> {
        if running {
            bail!("A report job is already running");
        }
        Ok(())
    }

    pub fn manager(&self) -> &'a BaseManager {
        self.manager
    }
}
