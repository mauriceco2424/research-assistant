use crate::bases::{
    category_slug, Base, BaseManager, CategoryAssignmentsIndex, CategoryRecord,
    CategorySnapshotStore, CategoryStore, LibraryEntry,
};
use crate::orchestration::{log_event, EventType, ReportProgressTracker};
use crate::reports::config_store::{ReportConfigDefaults, ReportConfigStore};
use crate::reports::consent_registry::{ConsentRegistry, FIGURE_GALLERY_OPERATION};
use crate::reports::figure_gallery::{FigureGalleryPreparer, GalleryAsset};
use crate::reports::html_renderer::{HtmlRenderer, RenderConfig, RenderedFile};
use crate::reports::manifest::{ReportBuildRequest, ReportManifest, ReportScopeMode};
use crate::reports::manifest_writer::{planned_entry, unchanged_entry, ReportManifestWriter};
use crate::reports::visualizations::VisualizationSelector;
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const BUILD_QUEUE_FILE: &str = "reports/build_queue.json";
const REPORT_DIR: &str = "reports";
const CATEGORY_SUBDIR: &str = "categories";
const MAX_HISTORY: usize = 20;
const STALE_SNAPSHOT_DAYS: i64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueueJob {
    request: ReportBuildRequest,
    status: BuildJobStatus,
    queued_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    error_message: Option<String>,
}

impl QueueJob {
    fn new(request: ReportBuildRequest) -> Self {
        Self {
            request,
            status: BuildJobStatus::Queued,
            queued_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BuildJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BuildQueueState {
    active: Option<QueueJob>,
    queued: VecDeque<QueueJob>,
    history: VecDeque<QueueJob>,
}

impl Default for BuildQueueState {
    fn default() -> Self {
        Self {
            active: None,
            queued: VecDeque::new(),
            history: VecDeque::new(),
        }
    }
}

#[derive(Debug)]
pub struct ReportBuildResult {
    pub manifest: ReportManifest,
    pub scope_label: String,
    pub duration_ms: i64,
    pub orchestration_id: Uuid,
    pub updated_files: Vec<PathBuf>,
    pub figures_enabled: bool,
    pub visualization_types: Vec<String>,
}

pub struct ReportBuildService<'a> {
    manager: &'a BaseManager,
    base: Base,
    state_path: PathBuf,
    state: BuildQueueState,
}

impl<'a> ReportBuildService<'a> {
    pub fn new(manager: &'a BaseManager, base: &Base) -> Result<Self> {
        let state_path = base.ai_layer_path.join(BUILD_QUEUE_FILE);
        let state = Self::load_state(&state_path)?;
        Ok(Self {
            manager,
            base: base.clone(),
            state_path,
            state,
        })
    }

    pub fn enqueue(&mut self, request: ReportBuildRequest) -> Result<Uuid> {
        let job = QueueJob::new(request.clone());
        self.state.queued.push_back(job);
        self.persist_state()?;
        Ok(request.request_id)
    }

    pub fn ensure_singleton(&self) -> Result<()> {
        if let Some(active) = &self.state.active {
            bail!(
                "Report job {} is still running; wait for completion before queueing another request.",
                active.request.request_id
            );
        }
        Ok(())
    }

    pub fn run_next(&mut self) -> Result<Option<ReportBuildResult>> {
        if self.state.active.is_some() {
            return Ok(None);
        }
        let mut job = match self.state.queued.pop_front() {
            Some(job) => job,
            None => return Ok(None),
        };
        job.status = BuildJobStatus::Running;
        job.started_at = Some(Utc::now());
        self.state.active = Some(job.clone());
        self.persist_state()?;
        let mut progress = ReportProgressTracker::new(self.manager, &self.base);
        let scope_label = job.request.scope_label();
        progress.start(&format!("reports {}", scope_label))?;
        let result = self.execute_job(&job.request, &mut progress);
        match result {
            Ok(build_result) => {
                progress.finish("Report build completed")?;
                job.status = BuildJobStatus::Succeeded;
                job.completed_at = Some(Utc::now());
                self.append_history(job);
                self.state.active = None;
                self.persist_state()?;
                Ok(Some(build_result))
            }
            Err(err) => {
                progress.finish("Report build failed")?;
                job.status = BuildJobStatus::Failed;
                job.error_message = Some(err.to_string());
                job.completed_at = Some(Utc::now());
                self.append_history(job);
                self.state.active = None;
                self.persist_state()?;
                Err(err)
            }
        }
    }

    pub fn manager(&self) -> &'a BaseManager {
        self.manager
    }

    fn execute_job(
        &self,
        request: &ReportBuildRequest,
        progress: &mut ReportProgressTracker,
    ) -> Result<ReportBuildResult> {
        let config_store = ReportConfigStore::new(self.manager, &self.base);
        let defaults = config_store.load_defaults()?;
        progress.update("Loaded Base defaults", 5)?;
        let effective = config_store.apply_overrides(&defaults, &request.overrides);
        let consent_registry = ConsentRegistry::new(self.manager, &self.base);
        let mut consent_tokens = Vec::new();
        let config_signature = signature_for(&effective, request);
        let entries = self.manager.load_library_entries(&self.base)?;
        if entries.is_empty() {
            bail!("No library entries found for this Base. Ingest papers before running reports.");
        }
        let category_store = CategoryStore::new(&self.base)?;
        let categories = category_store.list()?;
        self.validate_freshness(request, &categories)?;
        progress.update("Validated AI-layer freshness", 15)?;
        let assignments_index = CategoryAssignmentsIndex::new(&self.base)?;
        let entries_by_id: HashMap<Uuid, LibraryEntry> = entries
            .iter()
            .cloned()
            .map(|entry| (entry.entry_id, entry))
            .collect();
        let scope_plan = ScopePlanner::new(
            &self.base,
            request,
            &categories,
            &assignments_index,
            &entries_by_id,
        )?;
        self.fail_if_overwrite_prohibited(request, &scope_plan)?;
        let mut figure_assets: HashMap<Uuid, Vec<GalleryAsset>> = HashMap::new();
        if effective.include_figures {
            let consent = consent_registry.ensure_active(FIGURE_GALLERY_OPERATION)?;
            consent_tokens.push(consent.consent_id);
            let gallery = FigureGalleryPreparer::new(&self.base);
            figure_assets = gallery.prepare(&entries, scope_plan.entry_categories())?;
            progress.update("Prepared figure galleries", 30)?;
        } else {
            progress.update("Figures disabled for this run", 30)?;
        }
        progress.update("Prepared scope plan", 40)?;
        let render_config = RenderConfig {
            include_figures: effective.include_figures,
            include_visualizations: effective.include_visualizations.clone(),
        };
        let viz_selector = VisualizationSelector::new(&self.base);
        let viz_selection =
            viz_selector.select(&effective.include_visualizations, &consent_registry)?;
        consent_tokens.extend(viz_selection.consent_tokens.clone());
        let renderer = HtmlRenderer::new(
            &self.base,
            request.request_id,
            render_config,
            viz_selection.render_entries.clone(),
        )?;
        let staging_guard = StagingGuard::new(renderer.staging_dir().to_path_buf());
        let mut rendered_files: Vec<RenderedFile> = Vec::new();
        if scope_plan.global_requested {
            rendered_files.push(renderer.render_global(&entries, &figure_assets)?);
        }
        for cat in &scope_plan.render_categories {
            rendered_files.push(renderer.render_category(
                &cat.record,
                &cat.entries,
                &cat.pinned,
                &figure_assets,
            )?);
        }
        progress.update("Rendered HTML to staging", 65)?;
        simulate_failure_if_requested()?;
        let mut backup_guard = FileBackupGuard::new();
        for file in &rendered_files {
            backup_guard.capture(&file.final_path)?;
        }
        let mut updated_outputs = Vec::new();
        for file in &rendered_files {
            file.commit()?;
            updated_outputs.push(planned_entry(&file.final_path, &file.scope));
        }
        backup_guard.commit();
        progress.update("Committed HTML outputs", 80)?;
        let mut unchanged = Vec::new();
        for (scope, path) in &scope_plan.unchanged_outputs {
            if path.exists() {
                unchanged.push(unchanged_entry(path, scope)?);
            }
        }
        updated_outputs.extend(unchanged);
        let figure_count: usize = figure_assets.values().map(|items| items.len()).sum();
        let viz_dataset_ids = viz_selection.dataset_ids.clone();
        let mut metadata = HashMap::new();
        metadata.insert(
            "include_figures".into(),
            serde_json::json!(effective.include_figures),
        );
        metadata.insert(
            "include_visualizations".into(),
            serde_json::json!(effective.include_visualizations),
        );
        let duration_ms = progress.elapsed_ms();
        let writer = ReportManifestWriter::new(self.manager, &self.base);
        let manifest = writer.write(
            request,
            updated_outputs,
            config_signature,
            progress.started_at(),
            duration_ms,
            progress.job_id(),
            figure_count,
            entries.len(),
            consent_tokens.clone(),
            viz_dataset_ids.clone(),
            metadata,
        )?;
        log_event(
            self.manager,
            &self.base,
            EventType::ReportsGenerated,
            serde_json::json!({
                "request_id": request.request_id,
                "scope": request.scope.label(),
                "outputs": manifest.outputs.iter().map(|o| o.path.display().to_string()).collect::<Vec<_>>(),
                "duration_ms": duration_ms,
                "orchestration_id": manifest.orchestration_id,
            }),
        )?;
        drop(staging_guard);
        Ok(ReportBuildResult {
            manifest,
            scope_label: request.scope_label(),
            duration_ms,
            orchestration_id: progress.job_id(),
            updated_files: rendered_files
                .into_iter()
                .map(|file| file.final_path)
                .collect(),
            figures_enabled: effective.include_figures,
            visualization_types: effective.include_visualizations.clone(),
        })
    }

    fn fail_if_overwrite_prohibited(
        &self,
        request: &ReportBuildRequest,
        scope_plan: &ScopePlanner,
    ) -> Result<()> {
        if request.overwrite_existing {
            return Ok(());
        }
        let mut existing = Vec::new();
        for path in &scope_plan.target_paths {
            if path.exists() {
                existing.push(path.display().to_string());
            }
        }
        if !existing.is_empty() {
            bail!(
                "Existing report files would be overwritten. Re-run with overwrite confirmation to continue:\n{}",
                existing.join("\n")
            );
        }
        Ok(())
    }

    fn validate_freshness(
        &self,
        request: &ReportBuildRequest,
        categories: &[CategoryRecord],
    ) -> Result<()> {
        let snapshot_store = CategorySnapshotStore::new(&self.base)?;
        let snapshots = snapshot_store.list()?;
        let latest = snapshots.first().context(
            "No category snapshots found. Regenerate categories before running reports.",
        )?;
        let age = Utc::now().signed_duration_since(latest.taken_at);
        let age_days = age.num_days();
        if age_days > STALE_SNAPSHOT_DAYS {
            bail!(
                "Latest category snapshot is {age_days} days old (>{STALE_SNAPSHOT_DAYS}). Rerun categorization or capture a fresh snapshot before rebuilding reports."
            );
        }
        let tracked: HashSet<String> = latest.files.iter().cloned().collect();
        for category_id in &request.scope.categories {
            let needle = format!("definitions/{}.json", category_id);
            if !tracked.contains(&needle) {
                let name = categories
                    .iter()
                    .find(|record| &record.definition.category_id == category_id)
                    .map(|record| record.definition.name.clone())
                    .unwrap_or_else(|| category_id.to_string());
                bail!(
                    "Category '{name}' is missing from the latest snapshot. Run `/speckit.tasks` for the categorization spec or regenerate categories before rebuilding reports."
                );
            }
        }
        Ok(())
    }

    fn append_history(&mut self, job: QueueJob) {
        if self.state.history.len() >= MAX_HISTORY {
            self.state.history.pop_front();
        }
        self.state.history.push_back(job);
    }

    fn persist_state(&self) -> Result<()> {
        if let Some(parent) = self.state_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(&self.state)?;
        fs::write(&self.state_path, data)?;
        Ok(())
    }

    fn load_state(path: &Path) -> Result<BuildQueueState> {
        if !path.exists() {
            return Ok(BuildQueueState::default());
        }
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }
}

struct CategoryRenderPlan {
    record: CategoryRecord,
    entries: Vec<LibraryEntry>,
    pinned: Vec<LibraryEntry>,
}

struct ScopePlanner {
    global_requested: bool,
    render_categories: Vec<CategoryRenderPlan>,
    unchanged_outputs: Vec<(String, PathBuf)>,
    target_paths: Vec<PathBuf>,
    entry_categories: HashMap<Uuid, Vec<String>>,
}

impl ScopePlanner {
    fn new(
        base: &Base,
        request: &ReportBuildRequest,
        categories: &[CategoryRecord],
        assignments: &CategoryAssignmentsIndex,
        entries_by_id: &HashMap<Uuid, LibraryEntry>,
    ) -> Result<Self> {
        let mut planner = Self {
            global_requested: matches!(request.scope.mode(), ReportScopeMode::All)
                || matches!(request.scope.mode(), ReportScopeMode::Categories)
                || matches!(request.scope.mode(), ReportScopeMode::GlobalOnly)
                || request
                    .scope
                    .includes
                    .iter()
                    .any(|token| token.eq_ignore_ascii_case("global")),
            render_categories: Vec::new(),
            unchanged_outputs: Vec::new(),
            target_paths: Vec::new(),
            entry_categories: HashMap::new(),
        };
        let mut selected_ids: HashSet<Uuid> = request.scope.categories.iter().cloned().collect();
        if selected_ids.is_empty() && matches!(request.scope.mode(), ReportScopeMode::All) {
            selected_ids.extend(
                categories
                    .iter()
                    .map(|record| record.definition.category_id),
            );
        }
        for record in categories {
            let category_id = record.definition.category_id;
            let assigned = collect_entries_for_category(assignments, entries_by_id, &category_id)?;
            let pinned = collect_pinned(entries_by_id, record);
            planner.track_entries(&record.definition.name, &assigned);
            planner.track_entries(&record.definition.name, &pinned);
            if selected_ids.contains(&category_id) {
                planner.render_categories.push(CategoryRenderPlan {
                    record: record.clone(),
                    entries: assigned.clone(),
                    pinned: pinned.clone(),
                });
                planner
                    .target_paths
                    .push(category_output_path(base, &record.definition.name));
            } else {
                planner.unchanged_outputs.push((
                    format!("category:{}", record.definition.name),
                    category_output_path(base, &record.definition.name),
                ));
            }
        }
        planner.ensure_uncategorized(entries_by_id);
        if planner.global_requested {
            planner
                .target_paths
                .push(base.user_layer_path.join(REPORT_DIR).join("global.html"));
        } else {
            planner.unchanged_outputs.push((
                "global".into(),
                base.user_layer_path.join(REPORT_DIR).join("global.html"),
            ));
        }
        Ok(planner)
    }

    fn track_entries(&mut self, category_name: &str, entries: &[LibraryEntry]) {
        let slug = category_slug(category_name);
        for entry in entries {
            let slots = self.entry_categories.entry(entry.entry_id).or_default();
            if !slots.iter().any(|existing| existing == &slug) {
                slots.push(slug.clone());
            }
        }
    }

    fn ensure_uncategorized(&mut self, entries_by_id: &HashMap<Uuid, LibraryEntry>) {
        for entry in entries_by_id.values() {
            self.entry_categories
                .entry(entry.entry_id)
                .or_insert_with(|| vec!["uncategorized".into()]);
        }
    }

    fn entry_categories(&self) -> &HashMap<Uuid, Vec<String>> {
        &self.entry_categories
    }
}

fn collect_entries_for_category(
    assignments: &CategoryAssignmentsIndex,
    entries: &HashMap<Uuid, LibraryEntry>,
    category_id: &Uuid,
) -> Result<Vec<LibraryEntry>> {
    let mut assigned = Vec::new();
    for assignment in assignments.list_for_category(category_id)? {
        if let Some(entry) = entries.get(&assignment.paper_id) {
            assigned.push(entry.clone());
        }
    }
    assigned.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(assigned)
}

fn collect_pinned(
    entries: &HashMap<Uuid, LibraryEntry>,
    record: &CategoryRecord,
) -> Vec<LibraryEntry> {
    let mut pinned = Vec::new();
    for paper_id in &record.definition.pinned_papers {
        if let Some(entry) = entries.get(paper_id) {
            pinned.push(entry.clone());
        }
    }
    pinned
}

fn category_output_path(base: &Base, name: &str) -> PathBuf {
    let slug = category_slug(name);
    base.user_layer_path
        .join(REPORT_DIR)
        .join(CATEGORY_SUBDIR)
        .join(format!("{slug}.html"))
}

fn signature_for(defaults: &ReportConfigDefaults, request: &ReportBuildRequest) -> String {
    let payload = json!({
        "defaults": defaults,
        "overrides": request.overrides,
        "scope": {
            "mode": request.scope.mode().as_str(),
            "categories": request.scope.categories,
            "includes": request.scope.includes,
        }
    });
    format!("{:x}", Sha256::digest(payload.to_string().as_bytes()))
}

fn simulate_failure_if_requested() -> Result<()> {
    if env::var("REPORTS_FORCE_DISK_ERROR")
        .map(|val| val == "1")
        .unwrap_or(false)
    {
        bail!("Simulated disk failure to verify rollback safety.");
    }
    Ok(())
}

struct StagingGuard {
    path: PathBuf,
}

impl StagingGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for StagingGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

struct FileBackupGuard {
    entries: Vec<BackupEntry>,
    committed: bool,
}

impl FileBackupGuard {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            committed: false,
        }
    }

    fn capture(&mut self, original: &Path) -> Result<()> {
        if !original.exists() {
            return Ok(());
        }
        let backup = original.with_extension(format!("{}.rbak", Uuid::new_v4().to_string()));
        if let Some(parent) = backup.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(original, &backup)?;
        self.entries.push(BackupEntry {
            original: original.to_path_buf(),
            backup,
        });
        Ok(())
    }

    fn commit(&mut self) {
        self.committed = true;
        for entry in &self.entries {
            let _ = fs::remove_file(&entry.backup);
        }
    }
}

impl Drop for FileBackupGuard {
    fn drop(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        if self.committed {
            for entry in &self.entries {
                let _ = fs::remove_file(&entry.backup);
            }
        } else {
            for entry in &self.entries {
                if entry.original.exists() {
                    let _ = fs::remove_file(&entry.original);
                }
                let _ = fs::rename(&entry.backup, &entry.original);
            }
        }
    }
}

struct BackupEntry {
    original: PathBuf,
    backup: PathBuf,
}
