# Quickstart: Paper Discovery Consent Workflow

1) Check out branch `009-paper-discovery`.
2) Run chat-first discovery request (topic/gap/session) and verify candidates return in <30s.
3) Approve a batch with acquisition mode (metadata-only or metadata+pdf); confirm consent recorded in orchestration events with prompt manifest references.
4) Verify deduplication: candidates matching existing DOI/arXiv/title+author+year are flagged and not duplicated.
5) Confirm outcomes: successes stored locally with provenance; failures marked NEEDS_PDF with reasons; chat summarizes results.
6) Test offline/unavailable-remote-AI: discovery blocks or degrades gracefully with clear user notice and no hidden calls.
