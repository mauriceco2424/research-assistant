use super::IntegrationHarness;
use anyhow::Result;
use researchbase::chat::ChatSession;
use std::fs;

#[test]
fn figure_reprocess_requires_consent() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut chat = ChatSession::new()?;
    let base = chat.create_base("fig-reprocess")?;
    chat.select_base(&base.id)?;

    let sample_dir = harness.workspace_path().join("fig-reprocess-src");
    fs::create_dir_all(&sample_dir)?;
    fs::write(sample_dir.join("doc.pdf"), b"pdf")?;
    chat.ingest_path_a(&sample_dir)?;
    chat.figures_extract(None, "initial consent")?;

    let paper_id = chat.manager().load_library_entries(&base)?[0].entry_id;
    let err = chat
        .reprocess_figures(paper_id, "")
        .expect_err("reprocess without consent should fail");
    assert!(err.to_string().contains("approval"));
    Ok(())
}
