Start planning a new Favnir version.

The user will provide a version number (e.g. `v20.1.0`) as $ARGUMENTS.

Steps:
1. Read `fav/Cargo.toml` to confirm the current version number.
2. Read `CHANGELOG.md` (first 60 lines) to understand the most recent completed versions.
3. Find and read the relevant roadmap:
   - Use Glob on `versions/roadmap/` to list all roadmap `.md` files
   - Grep for the target version number/codename to locate the section
   - Read that section to extract the intended scope and deliverables
4. Determine the version directory based on the version number:
   - v9.x.x 〜 v20.0.x  → `versions/v9-v20/<version>/`
   - v20.1.x 〜 v25.x.x → `versions/v20-v25/<version>/`
   - v25.1.x 〜 v30.x.x → `versions/v25-v30/<version>/`
   - v30.1.x 〜 v35.x.x → `versions/v30-v35/<version>/`
   Create the directory.
5. Create three files in that directory, **grounded in the roadmap section from step 3**:
   - `spec.md` — feature specification with: Background, Goals, Syntax/API examples, Success Criteria, Error codes (if any), Files to modify
   - `plan.md` — numbered implementation steps in dependency order (AST → parser → checker → compiler → VM → tests → docs)
   - `tasks.md` — checkbox task list derived from plan.md, plus CHANGELOG and site docs tasks
6. After creating the files, **always** invoke the spec-reviewer agent to review them.
   The spec-reviewer will cross-check spec/plan/tasks against the roadmap and report any gaps.

The directory naming convention is `versions/v30-v35/v35.0B/` (with codename or patch number, lowercase v).

If the user didn't specify the theme, read the roadmap for that version first and confirm the theme with the user before creating files.
