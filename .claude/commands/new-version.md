Start planning a new Favnir version.

The user will provide a version number (e.g. `v20.1.0`) as $ARGUMENTS.

Steps:
1. Read `fav/Cargo.toml` to confirm the current version number.
2. Read `CHANGELOG.md` (first 60 lines) to understand the most recent completed versions.
3. Find and read the relevant roadmap:
   - Use Glob on `versions/roadmap/` to list all roadmap `.md` files
   - Grep for the target version number/codename to locate the section
   - Read that section to extract the intended scope and deliverables
   - **If no entry is found**: stop and tell the user:
     "このバージョンのロードマップエントリが見つかりません。
      versions/roadmap/ に該当バージョンの記述を追加してから再実行してください。"
     Do NOT proceed to create spec/plan/tasks without a roadmap entry.
4. Determine the version directory based on the version number:
   - v9.x.x 〜 v20.0.x  → `versions/v9-v20/<version>/`
   - v20.1.x 〜 v25.x.x → `versions/v20-v25/<version>/`
   - v25.1.x 〜 v30.x.x → `versions/v25-v30/<version>/`
   - v30.1.x 〜 v35.x.x → `versions/v30-v35/<version>/`
   - v35.1.x 〜 v40.x.x → `versions/v35-v40/<version>/`
   - v40.1.x 〜 v45.x.x → `versions/v40-v45/<version>/`
   - v45.1.x 〜 v50.x.x → `versions/v45-v50/<version>/`
   - v50.1.x 〜 v55.x.x → `versions/v50-v55/<version>/`
   - v55.1.x 〜 v60.x.x → `versions/v55-v60/<version>/`
   - v60.1.x 〜 v65.x.x → `versions/v60-v65/<version>/`
   - v65.1.x 〜 v70.x.x → `versions/v65-v70/<version>/`
   - v70.1.x 〜 v75.x.x → `versions/v70-v75/<version>/`
   - v75.1.x 〜 v80.x.x → `versions/v75-v80/<version>/`
   - v80.1.x 〜 v85.x.x → `versions/v80-v85/<version>/`
   Create the directory.
5. Create three files in that directory, **grounded in the roadmap section from step 3**:
   - `spec.md` — feature specification with: Background, Goals, Syntax/API examples, Success Criteria, Error codes (if any), Files to modify
   - `plan.md` — numbered implementation steps in dependency order (AST → parser → checker → compiler → VM → tests → docs)
   - `tasks.md` — checkbox task list derived from plan.md, plus CHANGELOG and site docs tasks

   **tasks.md の最終確認タスク（T-last）には必ず以下の CI チェックを含めること:**
   ```
   - [ ] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
   - [ ] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
   - [ ] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
   ```
   これらは CI の "Clippy" / "Self-fmt" ステップと同一であり、ローカルでの事前確認を必須とする。
   （ローカル Rust バージョンが CI より古い場合でも `--locked` フラグで Cargo.lock 固定版を使用することで差異を最小化できる）
6. After creating the files, **always** invoke the spec-reviewer agent to review them.
   The spec-reviewer will cross-check spec/plan/tasks against the roadmap and report any gaps.

The directory naming convention is `versions/v30-v35/v35.0B/` (with codename or patch number, lowercase v).

If the user didn't specify the theme, read the roadmap for that version first and confirm the theme with the user before creating files.
