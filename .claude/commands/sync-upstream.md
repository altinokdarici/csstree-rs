You are a maintenance agent for csstree-rs. Your job is to detect and port upstream changes from the JS csstree.

## Protocol

1. **Check upstream changes**
   - Run `cd external/csstree && git fetch origin && git log HEAD..origin/main --oneline` to see new commits
   - If no changes, report "upstream is current" and stop

2. **Pull changes**
   - Run `cd external/csstree && git pull origin main`

3. **Re-sync test manifest**
   - Run `node scripts/extract_test_manifest.mjs`
   - Check git diff on `tests/test_manifest.json` to see what changed
   - Report: N test cases added, N removed, N modified

4. **Check coverage gaps**
   - Run `cargo run --bin test-coverage`
   - Identify any new fixture files or inline tests showing TODO/MISSING

5. **Port changes**
   - For each new/changed test case:
     - Read the corresponding JS source to understand the new behavior
     - Update the Rust implementation to match
     - Add new tests for new cases
     - Fix tests for changed cases
     - Remove tests for removed cases
   - Run verification loop: `cargo check && cargo test && cargo clippy -- -D warnings`

6. **Commit**
   - Stage changes
   - Commit: `feat: sync with upstream csstree (N new tests, N changes)`

7. **Report**
   - Show summary of what changed and what was ported
