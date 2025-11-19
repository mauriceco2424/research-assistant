use super::IntegrationHarness;
use anyhow::Result;
use researchbase::chat::commands::reports::ReportConfigureOptions;
use researchbase::chat::ChatSession;
use researchbase::reports::config_store::ReportConfigStore;

#[test]
fn configure_requires_consent_for_figures() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "reports-configure-consent");
    let mut chat = ChatSession::new()?;
    chat.select_base(&base.id)?;

    let mut options = ReportConfigureOptions::default();
    options.include_figures = Some(true);
    let err = chat.reports_configure(options).expect_err("expected consent error");
    assert!(
        err.to_string().contains("Consent approval"),
        "Unexpected error: {err:?}"
    );

    let mut approved = ReportConfigureOptions::default();
    approved.include_figures = Some(true);
    approved.consent_text = Some("Enable figure galleries for testing".into());
    let summary = chat.reports_configure(approved)?;
    assert!(
        summary.contains("include_figures: true"),
        "Summary missing toggle: {summary}"
    );

    let store = ReportConfigStore::new(&manager, &base);
    let defaults = store.load_defaults()?;
    assert!(defaults.include_figures);
    Ok(())
}

#[test]
fn configure_updates_visualizations() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "reports-configure-viz");

    let mut chat = ChatSession::new()?;
    chat.select_base(&base.id)?;

    let mut options = ReportConfigureOptions::default();
    options.include_visualizations = Some(vec!["timeline".into(), "backlog_chart".into()]);
    let summary = chat.reports_configure(options)?;
    assert!(
        summary.contains("timeline"),
        "Expected visualizations listed in summary: {summary}"
    );

    let store = ReportConfigStore::new(&manager, &base);
    let defaults = store.load_defaults()?;
    assert_eq!(
        defaults.include_visualizations,
        vec!["timeline".to_string(), "backlog_chart".to_string()]
    );
    Ok(())
}
