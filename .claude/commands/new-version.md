Start planning a new Favnir version.

The user will provide a version number (e.g. `v20.1.0`) as $ARGUMENTS.

Steps:
1. Read `fav/Cargo.toml` to confirm the current version number.
2. Read `CHANGELOG.md` (first 60 lines) to understand the most recent completed versions.
3. Read the relevant roadmap section for this version from `versions/roadmap-v20.1-v25.0.md`
   (or `versions/roadmap-master.md` for v17-v20 range).
4. Determine the version directory based on the version number:
   - v9.x.x 〜 v20.0.x → `versions/v9-v20/$ARGUMENTS/`
   - v20.1.x 〜 v25.x.x → `versions/v20-v25/$ARGUMENTS/`
   Create the directory.
5. Create three files in that directory:
   - `spec.md` — feature specification with: Background, Goals, Syntax/API examples, Success Criteria, Error codes (if any), Files to modify
   - `plan.md` — numbered implementation steps in dependency order (AST → parser → checker → compiler → VM → tests → docs)
   - `tasks.md` — checkbox task list derived from plan.md, plus CHANGELOG and site docs tasks
6. After creating the files, **always** invoke the spec-reviewer agent to review them for gaps before reporting completion.

The directory naming convention is `versions/v20-v25/v20.1.0/` (with patch number, lowercase v).

If the user didn't specify the theme, read the roadmap for that version first and confirm the theme with the user before creating files.
