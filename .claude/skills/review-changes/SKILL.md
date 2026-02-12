# Review Changes

Review uncommitted changes for common issues before committing.

## Steps

1. Run `git diff` and `git diff --cached` to see all staged and unstaged changes
2. Run `git status` to see untracked files

## Checks

Review the diff for these issues:

- **Emojis in UI strings**: egui's default font cannot render emojis. Flag any emoji characters in strings passed to `ui.label()`, `ui.heading()`, `ui.button()`, or similar egui calls. The `icon` field in `register_operations!` should be `""` (empty string).
- **Missing tests**: If new operations or significant logic was added, check that corresponding tests exist in `tests/operations_tests.rs` or inline `#[cfg(test)]` modules.
- **Clippy warnings**: Run `cargo clippy -- -D warnings` and report any issues.
- **Unused imports/code**: Flag any dead code or unused imports in changed files.
- **Missing registration**: If a new operation file was added, verify it's registered in all three places: `mod.rs` (module + macro + enum + delegating macro), `editor.rs` (enum + all match arms).
- **Serde compatibility**: If `BitOperation` variants changed, check that `#[serde(tag = "type")]` and `#[serde(flatten)]` are preserved for backward compatibility.

## Output

Provide a summary:
- List of files changed with a one-line description of each change
- Any issues found from the checks above
- Overall assessment: ready to commit, or changes needed
