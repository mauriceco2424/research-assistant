# Quickstart – Reports & Visualizations

1. **Regenerate reports**
   ```bash
   cargo run -- reports regenerate --scope all --base <BASE_ID>
   ```
   - Confirms overwrite if HTML exists.
   - Streams orchestration progress every ≤5 seconds in chat/logs.

2. **Configure defaults**
   ```bash
   cargo run -- reports configure --base <BASE_ID> --include-figures on --visualizations concept_map timeline
   ```
   - Persists defaults per Base.
   - Prompts for consent manifests when enabling figures/viz requiring approvals.

3. **Share a bundle**
   ```bash
   cargo run -- reports share \
     --base <BASE_ID> \
     --manifest <MANIFEST_ID> \
     --format zip \
     --dest ./exports/base.zip \
     --include-figures on \
     --include-visualizations on \
     --overwrite
   ```
   - Creates bundle + provenance summary referencing manifest + consent tokens.
   - Use `--format directory` to emit an unpacked folder instead of a zip archive.
   - `--include-figures` / `--include-visualizations` default to the saved config values; pass `off` to exclude heavy assets.

4. **Verify manifest reproducibility**
   ```bash
   cargo run --bin verify_manifest -- <PATH_TO_MANIFEST_JSON>
   ```
   - Recomputes hashes for every output recorded in the manifest and fails fast if any HTML/assets drift from the audit log.
   - Use after `reports regenerate` or before sharing bundles to prove the AI-layer inputs can recreate byte-identical files.

5. **Run tests**
   ```bash
   cargo test reports::
   npm run test:chat-commands
   ```
   - Rust tests validate manifest creation, consent enforcement, bundling.
   - TypeScript harness ensures chat commands issue correct payloads.

6. **Verify outputs**
   - Generated HTML lives under `/User/<Base>/reports/` with `manifest.json` files alongside category/global subfolders.
   - Bundles store audit metadata next to exported archives.
