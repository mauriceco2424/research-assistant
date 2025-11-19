use crate::acquisition::figure_store::{FigureAssetRecord, FigureStore};
use crate::bases::{Base, LibraryEntry};
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const ASSETS_DIR: &str = "reports/assets/figures";

#[derive(Debug, Clone)]
pub struct GalleryAsset {
    pub caption: String,
    pub file_path: PathBuf,
}

pub struct FigureGalleryPreparer<'a> {
    base: &'a Base,
}

impl<'a> FigureGalleryPreparer<'a> {
    pub fn new(base: &'a Base) -> Self {
        Self { base }
    }

    pub fn prepare(
        &self,
        entries: &[LibraryEntry],
        entry_categories: &HashMap<Uuid, Vec<String>>,
    ) -> Result<HashMap<Uuid, Vec<GalleryAsset>>> {
        let store = FigureStore::new(self.base);
        let mut map: HashMap<Uuid, Vec<GalleryAsset>> = HashMap::new();
        if entries.is_empty() {
            return Ok(map);
        }
        let entry_ids: HashSet<Uuid> = entries.iter().map(|entry| entry.entry_id).collect();
        self.clean_assets_dir()?;
        for record in store.load_records()? {
            if !entry_ids.contains(&record.paper_id) {
                continue;
            }
            if !record.image_path.exists() {
                continue;
            }
            let dest = self.copy_asset(&record, entry_categories)?;
            map.entry(record.paper_id).or_default().push(GalleryAsset {
                caption: record.caption.clone(),
                file_path: dest,
            });
        }
        Ok(map)
    }

    fn clean_assets_dir(&self) -> Result<()> {
        let dir = self.base.user_layer_path.join(ASSETS_DIR);
        if dir.exists() {
            fs::remove_dir_all(&dir)
                .with_context(|| format!("Failed to reset {}", dir.display()))?;
        }
        Ok(())
    }

    fn copy_asset(
        &self,
        record: &FigureAssetRecord,
        entry_categories: &HashMap<Uuid, Vec<String>>,
    ) -> Result<PathBuf> {
        let category_slug = entry_categories
            .get(&record.paper_id)
            .and_then(|list| list.first())
            .cloned()
            .unwrap_or_else(|| "uncategorized".to_string());
        let dest_dir = self
            .base
            .user_layer_path
            .join(ASSETS_DIR)
            .join(category_slug)
            .join(record.paper_id.to_string());
        fs::create_dir_all(&dest_dir)
            .with_context(|| format!("Failed to create {}", dest_dir.display()))?;
        let filename = record
            .image_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("figure.png");
        let dest_path = dest_dir.join(filename);
        fs::copy(&record.image_path, &dest_path).with_context(|| {
            format!(
                "Failed to copy figure {} -> {}",
                record.image_path.display(),
                dest_path.display()
            )
        })?;
        Ok(dest_path)
    }
}
