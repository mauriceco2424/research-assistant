# Data Model: Paper Discovery Consent Workflow

**Date**: 2025-11-21  
**Branch**: 009-paper-discovery

## Entities

### CandidatePaper
- `id` (local candidate id)
- `title`
- `authors` (list)
- `venue` (optional)
- `year` (optional)
- `source_link`
- `rationale` (topic/gap/session context)
- `identifiers` (DOI, arXiv/eprint)
- `duplicate_match` (bool + matched record id)

### ApprovalBatch
- `batch_id`
- `request_context` (topic/gap/session ref)
- `selected_candidate_ids`
- `acquisition_mode` (metadata-only | metadata+pdf)
- `consent_record` (timestamp, user, manifest reference)

### AcquisitionEvent
- `event_id`
- `batch_id`
- `candidate_id`
- `endpoints_contacted` (list)
- `prompt_manifest_ref` (if AI-assisted)
- `outcome` (success | needs_pdf | skipped)
- `error_reason` (optional)
- `provenance` (source suggestion + approval ref)

### StoredPaperRecord
- `paper_id`
- `metadata` (title, authors, venue, year, identifiers, source_link)
- `pdf_path` (optional)
- `status` (complete | needs_pdf)
- `provenance` (discovery source, approval batch, acquisition event)
- `dedup_keys` (identifiers; normalized title+first author+year)
