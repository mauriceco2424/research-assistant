use crate::bases::{category_slug, Base, BaseManager, CategoryRecord, CategoryStore};
use crate::orchestration::{ConsentOperation, ConsentScope};
use crate::reports::build_service::{ReportBuildResult, ReportBuildService};
use crate::reports::config_store::{
    ReportConfigDefaults, ReportConfigOverrides, ReportConfigStore,
};
use crate::reports::consent_registry::{ConsentRegistry, FIGURE_GALLERY_OPERATION};
use crate::reports::manifest::{
    ReportBuildRequest, ReportScope, ReportScopeMode, ShareBundleDescriptor,
};
use crate::reports::share_builder::ShareFormat;
use crate::reports::share_service::ReportShareService;
use crate::reports::visualizations::VisualizationKind;
use anyhow::{bail, Context, Result};
use serde_json::json;
use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

pub struct ReportRegenerateOptions {
    pub scope: Option<String>,
    pub categories: Vec<String>,
    pub includes: Vec<String>,
    pub overrides: ReportConfigOverrides,
    pub overwrite_existing: bool,
}

impl Default for ReportRegenerateOptions {
    fn default() -> Self {
        Self {
            scope: None,
            categories: Vec::new(),
            includes: Vec::new(),
            overrides: ReportConfigOverrides::default(),
            overwrite_existing: false,
        }
    }
}

pub struct ReportsCommandBridge<'a> {
    manager: &'a BaseManager,
}

pub enum RegenerateOutcome {
    Completed(ReportBuildResult),
    Queued { request_id: Uuid, scope: String },
}

impl<'a> ReportsCommandBridge<'a> {
    pub fn new(manager: &'a BaseManager) -> Self {
        Self { manager }
    }

    pub fn configure(
        &self,
        base: &Base,
        options: ReportConfigureOptions,
    ) -> Result<ReportConfigureOutcome> {
        let store = ReportConfigStore::new(self.manager, base);
        let mut defaults = store.load_defaults()?;
        if let Some(days) = options.consent_refresh_days {
            defaults.consent_refresh_days = days;
        }
        let mut consent_ids = Vec::new();
        let registry = ConsentRegistry::new(self.manager, base);
        if let Some(include_figures) = options.include_figures {
            defaults.include_figures = include_figures;
            if include_figures {
                let manifest = registry.require_local_consent(
                    FIGURE_GALLERY_OPERATION,
                    &["figure_bitmaps", "captions"],
                    defaults.consent_refresh_days,
                    options.consent_text.as_deref(),
                    json!({
                        "operation": "figure_gallery",
                        "reason": "reports configure enable figure galleries"
                    }),
                )?;
                consent_ids.push(manifest.consent_id);
            }
        }
        if let Some(excluded) = options.excluded_assets.clone() {
            defaults.excluded_assets = excluded;
        }
        if let Some(list) = &options.include_visualizations {
            let kinds = normalize_visualizations(list)?;
            defaults.include_visualizations =
                kinds.iter().map(|kind| kind.label().to_string()).collect();
            for kind in kinds {
                if kind.requires_remote_layout() {
                    let approval = options.consent_text.as_deref().context(
                        "Remote visualization layouts require `--consent \"<reason>\"` text.",
                    )?;
                    let manifest = registry.require_remote_consent(
                        ConsentOperation::VisualizationRemoteLayout,
                        ConsentScope::default(),
                        approval,
                        json!({
                            "operation": kind.label(),
                            "reason": "reports configure enable visualization"
                        }),
                        defaults.consent_refresh_days,
                        &format!("visualization:{}", kind.label()),
                        &["visualization_layout"],
                        &format!("visualization::{}", kind.label()),
                    )?;
                    consent_ids.push(manifest.consent_id);
                }
            }
        }
        store.save_defaults(&defaults)?;
        Ok(ReportConfigureOutcome {
            defaults,
            consent_ids,
        })
    }

    pub fn regenerate(
        &self,
        base: &Base,
        options: ReportRegenerateOptions,
    ) -> Result<RegenerateOutcome> {
        let scope_mode = normalize_scope(options.scope.as_deref());
        let category_store = CategoryStore::new(base)?;
        let records = category_store.list()?;
        let mut scope = ReportScope::default();
        scope.mode = scope_mode.clone();
        scope.includes = options.includes.clone();
        scope.categories = resolve_categories(&options.categories, &records)?;
        if matches!(scope.mode(), ReportScopeMode::Categories) && scope.categories.is_empty() {
            bail!("Use --category <name|id> when specifying the categories scope.");
        }
        if matches!(scope.mode(), ReportScopeMode::Includes) {
            let include_ids = resolve_categories(&options.includes, &records)?;
            for id in include_ids {
                if !scope.categories.contains(&id) {
                    scope.categories.push(id);
                }
            }
        }
        let request = ReportBuildRequest::new(
            base.id,
            scope,
            options.overrides,
            options.overwrite_existing,
        );
        let mut service = ReportBuildService::new(self.manager, base)?;
        let request_id = service.enqueue(request.clone())?;
        match service.run_next()? {
            Some(result) => Ok(RegenerateOutcome::Completed(result)),
            None => Ok(RegenerateOutcome::Queued {
                request_id,
                scope: request.scope.label(),
            }),
        }
    }

    pub fn share(&self, base: &Base, options: ReportShareOptions) -> Result<ShareOutcome> {
        let service = ReportShareService::new(self.manager);
        let descriptor = service.share(
            base,
            options.manifest_id,
            options.destination.clone(),
            options.format,
            options.include_figures,
            options.include_visualizations,
            options.overwrite,
        )?;
        Ok(ShareOutcome { descriptor })
    }
}

fn normalize_scope(scope: Option<&str>) -> String {
    match scope.unwrap_or("all").to_ascii_lowercase().as_str() {
        "global" | "global_only" => ReportScopeMode::GlobalOnly.as_str().into(),
        "categories" => ReportScopeMode::Categories.as_str().into(),
        "includes" => ReportScopeMode::Includes.as_str().into(),
        _ => ReportScopeMode::All.as_str().into(),
    }
}

fn resolve_categories(requested: &[String], records: &[CategoryRecord]) -> Result<Vec<Uuid>> {
    let mut ids = Vec::new();
    let mut seen: HashSet<Uuid> = HashSet::new();
    for value in requested {
        if value.eq_ignore_ascii_case("global") {
            continue;
        }
        if let Ok(uuid) = Uuid::parse_str(value) {
            if seen.insert(uuid) {
                ids.push(uuid);
            }
            continue;
        }
        let lower = value.to_ascii_lowercase();
        let found = records.iter().find(|record| {
            record.definition.name.to_ascii_lowercase() == lower
                || category_slug_cmp(&record.definition.name, &lower)
        });
        if let Some(record) = found {
            if seen.insert(record.definition.category_id) {
                ids.push(record.definition.category_id);
            }
        } else {
            bail!("Unknown category '{value}'. Provide the exact name or ID.");
        }
    }
    Ok(ids)
}

fn category_slug_cmp(name: &str, needle: &str) -> bool {
    category_slug(name) == needle.replace(' ', "-")
}

pub struct ReportConfigureOptions {
    pub include_figures: Option<bool>,
    pub include_visualizations: Option<Vec<String>>,
    pub excluded_assets: Option<Vec<String>>,
    pub consent_refresh_days: Option<u32>,
    pub consent_text: Option<String>,
}

impl Default for ReportConfigureOptions {
    fn default() -> Self {
        Self {
            include_figures: None,
            include_visualizations: None,
            excluded_assets: None,
            consent_refresh_days: None,
            consent_text: None,
        }
    }
}

pub struct ReportConfigureOutcome {
    pub defaults: ReportConfigDefaults,
    pub consent_ids: Vec<Uuid>,
}

fn normalize_visualizations(requested: &[String]) -> Result<Vec<VisualizationKind>> {
    let mut kinds: Vec<VisualizationKind> = Vec::new();
    for value in requested {
        let kind = VisualizationKind::from_label(value)?;
        if !kinds
            .iter()
            .any(|existing| existing.label() == kind.label())
        {
            kinds.push(kind);
        }
    }
    Ok(kinds)
}

pub struct ReportShareOptions {
    pub manifest_id: Uuid,
    pub destination: PathBuf,
    pub format: ShareFormat,
    pub include_figures: bool,
    pub include_visualizations: bool,
    pub overwrite: bool,
}

pub struct ShareOutcome {
    pub descriptor: ShareBundleDescriptor,
}
