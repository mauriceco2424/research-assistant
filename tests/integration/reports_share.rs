use super::IntegrationHarness;
use anyhow::Result;
use researchbase::bases::BaseManager;
use researchbase::reports::manifest::{
    ReportManifest, ReportOutputEntry,
};
use researchbase::reports::share_builder::ShareFormat;
use researchbase::reports::share_service::ReportShareService;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn share_creates_zip_bundle_with_manifest() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "reports-share-zip");
    let manifest_id = seed_manifest(&base)?;
    let destination = base
        .user_layer_path
        .join("exports")
        .join("bundle.zip");
    let service = ReportShareService::new(&manager);
    let descriptor = service.share(
        &base,
        manifest_id,
        destination.clone(),
        ShareFormat::Zip,
        true,
        true,
        false,
    )?;
    assert!(destination.exists(), "bundle {} missing", destination.display());
    assert_eq!(descriptor.format, "zip");
    assert!(descriptor.size_bytes.unwrap_or(0) > 0);
    let share_manifest = base
        .ai_layer_path
        .join("reports/share_manifests")
        .join(format!("{}.json", descriptor.bundle_id));
    assert!(
        share_manifest.exists(),
        "expected share manifest {} to exist",
        share_manifest.display()
    );
    Ok(())
}

fn seed_manifest(base: &researchbase::bases::Base) -> Result<Uuid> {
    let report_dir = base.user_layer_path.join("reports");
    fs::create_dir_all(&report_dir)?;
    let html_path = report_dir.join("global.html");
    fs::write(&html_path, "<html><body>example</body></html>")?;
    let request_id = Uuid::new_v4();
    let mut manifest = ReportManifest::new(base, request_id, "sig".into());
    manifest.outputs.push(ReportOutputEntry {
        path: html_path,
        scope: "global".into(),
        hash: "".into(),
        kind: "html".into(),
    });
    manifest.persist(base)?;
    Ok(manifest.manifest_id)
}
