use crate::bases::{Base, BaseManager};
use crate::reports::manifest::{hash_path, ReportManifest, ShareBundleDescriptor};
use anyhow::{bail, Context, Result};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;
use zip::write::FileOptions;

pub struct ShareBundleBuilder<'a> {
    pub manager: &'a BaseManager,
    pub base: Base,
}

impl<'a> ShareBundleBuilder<'a> {
    pub fn new(manager: &'a BaseManager, base: &Base) -> Self {
        Self {
            manager,
            base: base.clone(),
        }
    }

    pub fn create_bundle(
        &self,
        manifest: &ReportManifest,
        destination: &Path,
    ) -> Result<ShareBundleDescriptor> {
        if destination.exists() {
            bail!("Destination {} already exists", destination.display());
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Unable to create {}", parent.display()))?;
        }
        let bundle_id = Uuid::new_v4();
        let file = File::create(destination)
            .with_context(|| format!("Failed to create {}", destination.display()))?;
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default();
        for output in &manifest.outputs {
            let path = &output.path;
            if !path.exists() {
                continue;
            }
            zip.start_file(
                path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("report.html"),
                options,
            )?;
            let mut data = Vec::new();
            File::open(path)?.read_to_end(&mut data)?;
            zip.write_all(&data)?;
        }
        zip.finish()?;
        let metadata = fs::metadata(destination)
            .with_context(|| format!("Missing bundle {}", destination.display()))?;
        let checksum = Some(hash_path(destination)?);
        Ok(ShareBundleDescriptor {
            bundle_id,
            manifest_id: manifest.manifest_id,
            destination: destination.to_path_buf(),
            format: "zip".into(),
            include_figures: true,
            include_visualizations: true,
            checksum,
            size_bytes: Some(metadata.len()),
        })
    }
}
