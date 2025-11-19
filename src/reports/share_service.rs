use crate::bases::{Base, BaseManager};
use crate::orchestration::{log_event, EventType};
use crate::reports::manifest::{read_manifest, ReportManifest, ShareBundleDescriptor};
use crate::reports::share_builder::{ShareBundleBuilder, ShareBundleOptions, ShareFormat};
use crate::reports::share_manifest::ShareManifestWriter;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub struct ReportShareService<'a> {
    manager: &'a BaseManager,
}

impl<'a> ReportShareService<'a> {
    pub fn new(manager: &'a BaseManager) -> Self {
        Self { manager }
    }

    pub fn share(
        &self,
        base: &Base,
        manifest_id: Uuid,
        destination: PathBuf,
        format: ShareFormat,
        include_figures: bool,
        include_visualizations: bool,
        overwrite: bool,
    ) -> Result<ShareBundleDescriptor> {
        if destination.exists() {
            if overwrite {
                if destination.is_dir() {
                    fs::remove_dir_all(&destination)
                        .with_context(|| format!("Failed clearing {}", destination.display()))?;
                } else {
                    fs::remove_file(&destination)
                        .with_context(|| format!("Failed deleting {}", destination.display()))?;
                }
            } else {
                bail!(
                    "Destination {} already exists. Re-run with --overwrite to replace it.",
                    destination.display()
                );
            }
        }
        let manifest = self.load_manifest(base, manifest_id)?;
        let builder = ShareBundleBuilder::new(self.manager, base);
        let bundle_options = ShareBundleOptions {
            manifest: &manifest,
            destination: destination.clone(),
            format,
            include_figures,
            include_visualizations,
        };
        let descriptor = builder.create_bundle(&bundle_options)?;
        ShareManifestWriter::new(self.manager, base).persist(&descriptor)?;
        log_event(
            self.manager,
            base,
            EventType::ReportsShared,
            serde_json::json!({
                "bundle_id": descriptor.bundle_id,
                "manifest_id": descriptor.manifest_id,
                "destination": descriptor.destination,
                "format": descriptor.format,
                "include_figures": descriptor.include_figures,
                "include_visualizations": descriptor.include_visualizations,
            }),
        )?;
        Ok(descriptor)
    }

    fn load_manifest(&self, base: &Base, manifest_id: Uuid) -> Result<ReportManifest> {
        let path = base
            .ai_layer_path
            .join("reports")
            .join("manifests")
            .join(format!("{manifest_id}.json"));
        read_manifest(&path).with_context(|| format!("Manifest {} not found", manifest_id))
    }
}
