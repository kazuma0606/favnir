Run a spec or code review for the current version.

If $ARGUMENTS is a version number (e.g. `v20.1.0`):
  Use the spec-reviewer agent to review `versions/v9-v20/$ARGUMENTS/spec.md`,
  `versions/v9-v20/$ARGUMENTS/plan.md`, and `versions/v9-v20/$ARGUMENTS/tasks.md`.

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
