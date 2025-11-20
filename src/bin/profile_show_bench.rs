use std::{env, fs, time::Instant};

use anyhow::{Context, Result};
use chrono::Utc;
use researchbase::{
    bases::{BaseManager, ProfileLayout},
    orchestration::profiles::{
        defaults::default_knowledge_profile,
        model::{
            EvidenceKind, EvidenceRef, KnowledgeEntry, KnowledgeProfile, MasteryLevel,
            ProfileType, VerificationStatus,
        },
        service::ProfileService,
        storage::write_profile,
    },
};

fn main() -> Result<()> {
    let entries = env::args()
        .nth(1)
        .unwrap_or_else(|| "5".into())
        .parse::<usize>()
        .context("entries argument must be a positive integer")?;
    run_benchmark(entries)
}

fn run_benchmark(entry_count: usize) -> Result<()> {
    let mut manager = BaseManager::new()?;
    let base_name = format!("profile-bench-{entry_count}");
    let base = manager.create_base(&base_name)?;
    let service = ProfileService::new(&manager, &base);

    // Seed knowledge profile with deterministic entry_count size
    seed_knowledge_profile(&base, entry_count)?;

    // Warm-up call to ensure HTML render path exists
    let _ = service.show(ProfileType::Knowledge)?;
    let start = Instant::now();
    let output = service.show(ProfileType::Knowledge)?;
    let elapsed = start.elapsed();

    println!(
        "profile show knowledge | entries={} | highlights={} | duration={:.3?}",
        entry_count,
        output.summary.highlights.len(),
        elapsed
    );
    Ok(())
}

fn seed_knowledge_profile(base: &researchbase::bases::Base, entry_count: usize) -> Result<()> {
    let layout = ProfileLayout::new(base);
    let mut profile: KnowledgeProfile = if layout.profile_json("knowledge").exists() {
        let data = fs::read(layout.profile_json("knowledge"))?;
        serde_json::from_slice(&data)?
    } else {
        default_knowledge_profile()
    };
    profile.entries = (0..entry_count).map(make_entry).collect();
    profile.metadata.last_updated = Utc::now();
    profile.summary = Vec::new();
    profile.history.clear();
    write_profile(layout.profile_json("knowledge"), &profile)?;
    Ok(())
}

fn make_entry(idx: usize) -> KnowledgeEntry {
    KnowledgeEntry {
        concept: format!("Concept {}", idx + 1),
        mastery_level: match idx % 4 {
            0 => MasteryLevel::Novice,
            1 => MasteryLevel::Developing,
            2 => MasteryLevel::Proficient,
            _ => MasteryLevel::Expert,
        },
        evidence_refs: vec![EvidenceRef {
            kind: EvidenceKind::Note,
            identifier: format!("notes/{}.md", idx + 1),
            confidence: 0.8,
        }],
        weakness_flags: if idx % 7 == 0 {
            vec!["Needs refresh".into()]
        } else {
            Vec::new()
        },
        learning_links: Vec::new(),
        last_reviewed: Utc::now(),
        verification_status: if idx % 11 == 0 {
            VerificationStatus::Stale
        } else {
            VerificationStatus::Verified
        },
    }
}
