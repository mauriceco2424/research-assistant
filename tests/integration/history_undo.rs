use super::IntegrationHarness;
use anyhow::Result;
use researchbase::chat::ChatSession;
use std::fs;

#[test]
fn history_range_and_figure_undo() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut chat = ChatSession::new()?;
    let base = chat.create_base("history-base")?;
    chat.select_base(&base.id)?;

    let sample_dir = harness.workspace_path().join("samples");
    fs::create_dir_all(&sample_dir)?;
    fs::write(sample_dir.join("paper.pdf"), b"pdf")?;
    chat.ingest_path_a(&sample_dir)?;
    chat.figures_extract(None, "figure consent")?;

    let history = chat.history_show(Some("7d"))?;
    assert!(history.iter().any(|line| line.contains("Figure extraction")));

    let undo = chat.undo_last_figure_extraction()?;
    assert!(undo.contains("Undid figure extraction"));
    Ok(())
}
