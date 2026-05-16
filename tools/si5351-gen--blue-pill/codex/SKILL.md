# Local Skill Memo

This is a project-local workflow note, not an auto-registered Codex system skill.

Use it as an explicit context file when starting a new session in this repo.

## Workflow

1. Read `codex/AGENTS.md`
2. Read `codex/notes.md`
3. Read `codex/worklog.md`
4. Inspect current git status before editing
5. Verify changes with:
   - `cargo fmt`
   - `cargo check --bin si5351-gen`
   - `cargo build` when image size may matter

## Editing Priorities

- Preserve current hardware mappings unless the user explicitly changes wiring
- Keep `CLK0` and `CLK1` as the active outputs unless requested otherwise
- Avoid unnecessary divergence in the vendored `src/support/si5351.rs`
- Keep UI changes logged in `codex/worklog.md`
