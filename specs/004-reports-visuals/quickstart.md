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
   cargo run -- reports share --base <BASE_ID> --manifest <MANIFEST_ID> --format zip --dest ./exports/base.zip
   ```
   - Creates bundle + provenance summary referencing manifest + consent tokens.

4. **Run tests**
   ```bash
   cargo test reports::
   npm run test:chat-commands
   ```
   - Rust tests validate manifest creation, consent enforcement, bundling.
   - TypeScript harness ensures chat commands issue correct payloads.

5. **Verify outputs**
   - Generated HTML lives under `/User/<Base>/reports/` with `manifest.json` files alongside category/global subfolders.
   - Bundles store audit metadata next to exported archives.
