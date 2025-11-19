use crate::bases::{Base, BaseManager};
use crate::reports::manifest::{hash_path, ReportManifest, ShareBundleDescriptor};
use anyhow::{bail, Context, Result};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;
use zip::write::FileOptions;

pub struct ShareBundleBuilder<'a> {
    pub manager: &'a BaseManager,
    pub base: Base,
}

pub enum ShareFormat {
    Zip,
    Directory,
}

impl ShareFormat {
    fn as_str(&self) -> &'static str {
        match self {
            ShareFormat::Zip => "zip",
            ShareFormat::Directory => "directory",
        }
    }
}

pub struct ShareBundleOptions<'a> {
    pub manifest: &'a ReportManifest,
    pub destination: PathBuf,
    pub format: ShareFormat,
    pub include_figures: bool,
    pub include_visualizations: bool,
}

impl<'a> ShareBundleBuilder<'a> {
    pub fn new(manager: &'a BaseManager, base: &Base) -> Self {
        Self {
            manager,
            base: base.clone(),
        }
    }

    pub fn create_bundle(&self, options: &ShareBundleOptions) -> Result<ShareBundleDescriptor> {
        let destination = &options.destination;
        if destination.exists() {
            bail!("Destination {} already exists", destination.display());
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Unable to create {}", parent.display()))?;
        }
        let files = self.collect_files(options)?;
        let bundle_id = Uuid::new_v4();
        match options.format {
            ShareFormat::Zip => self.write_zip(bundle_id, options, files),
            ShareFormat::Directory => self.write_directory(bundle_id, options, files),
        }
    }

    fn collect_files(&self, options: &ShareBundleOptions) -> Result<Vec<(PathBuf, PathBuf)>> {
        let mut files = Vec::new();
        for output in &options.manifest.outputs {
            if output.path.exists() {
                let relative = self.relative_to_user_layer(&output.path);
                files.push((output.path.clone(), relative));
            }
        }
        if options.include_figures {
            let figures_dir = self.base.user_layer_path.join("reports/assets/figures");
            if figures_dir.exists() {
                for entry in WalkDir::new(&figures_dir)
                    .into_iter()
                    .filter_map(Result::ok)
                {
                    if entry.file_type().is_file() {
                        let rel = self.relative_to_user_layer(entry.path());
                        files.push((entry.into_path(), rel));
                    }
                }
            }
        }
        if options.include_visualizations {
            let viz_dir = self.base.user_layer_path.join("reports/visualizations");
            if viz_dir.exists() {
                for entry in WalkDir::new(&viz_dir).into_iter().filter_map(Result::ok) {
                    if entry.file_type().is_file() {
                        let rel = self.relative_to_user_layer(entry.path());
                        files.push((entry.into_path(), rel));
                    }
                }
            }
        }
        Ok(files)
    }

    fn write_zip(
        &self,
        bundle_id: Uuid,
        options: &ShareBundleOptions,
        files: Vec<(PathBuf, PathBuf)>,
    ) -> Result<ShareBundleDescriptor> {
        let destination = &options.destination;
        let file = File::create(destination)
            .with_context(|| format!("Failed to create {}", destination.display()))?;
        let mut zip = zip::ZipWriter::new(file);
        let entry_opts = FileOptions::default();
        for (source, relative) in &files {
            zip.start_file(relative.to_string_lossy(), entry_opts)?;
            let mut data = Vec::new();
            File::open(source)?.read_to_end(&mut data)?;
            zip.write_all(&data)?;
        }
        zip.finish()?;
        let metadata = fs::metadata(destination)
            .with_context(|| format!("Missing bundle {}", destination.display()))?;
        let checksum = Some(hash_path(destination)?);
        Ok(self.descriptor(bundle_id, options, metadata.len(), checksum))
    }

    fn write_directory(
        &self,
        bundle_id: Uuid,
        options: &ShareBundleOptions,
        files: Vec<(PathBuf, PathBuf)>,
    ) -> Result<ShareBundleDescriptor> {
        fs::create_dir_all(&options.destination)?;
        for (source, relative) in &files {
            let dest_path = options.destination.join(relative);
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source, &dest_path).with_context(|| {
                format!(
                    "Failed to copy {} -> {}",
                    source.display(),
                    dest_path.display()
                )
            })?;
        }
        let size = dir_size(&options.destination)?;
        Ok(self.descriptor(bundle_id, options, size, None))
    }

    fn descriptor(
        &self,
        bundle_id: Uuid,
        options: &ShareBundleOptions,
        size_bytes: u64,
        checksum: Option<String>,
    ) -> ShareBundleDescriptor {
        ShareBundleDescriptor {
            bundle_id,
            manifest_id: options.manifest.manifest_id,
            destination: options.destination.clone(),
            format: options.format.as_str().into(),
            include_figures: options.include_figures,
            include_visualizations: options.include_visualizations,
            checksum,
            size_bytes: Some(size_bytes),
        }
    }

    fn relative_to_user_layer(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.base.user_layer_path)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| {
                PathBuf::from(
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("asset.bin"),
                )
            })
    }
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut total = 0;
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            total += entry.metadata()?.len();
        }
    }
    Ok(total)
}
