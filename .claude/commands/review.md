Run a spec or code review for the current version.

If $ARGUMENTS is a version number (e.g. `v20.1.0`) or a codename (e.g. `v35.0B`):
  Use Glob to find the version directory across all generation folders:
  `versions/v9-v20/`, `versions/v20-v25/`, `versions/v25-v30/`, `versions/v30-v35/`
  Then use the spec-reviewer agent to review the spec.md, plan.md, and tasks.md found there.
  The spec-reviewer will also cross-check against the roadmap in `versions/roadmap/`.

If $ARGUMENTS is `code` or empty:
  Use the code-reviewer agent to review recently changed files:
  ```bash
  cd /c/Users/yoshi/favnir && git diff --name-only HEAD~1 HEAD
  ```
  Then review each changed file against the code-reviewer checklist.

If $ARGUMENTS is `wasm`:
  Use the wasm-compat-checker agent.

If $ARGUMENTS is `roadmap`:
  Use the roadmap-drift-detector agent.

If $ARGUMENTS is `match`:
  Use the exhaustive-match-checker agent on recently modified AST/IR files.

If $ARGUMENTS is `changelog`:
  Use the changelog-verifier agent for the current version in Cargo.toml.

Report findings with priority labels [HIGH] / [MED] / [LOW].
End with a summary: "レビュー完了 — X 件の指摘" or "レビュー完了 — 問題なし".
