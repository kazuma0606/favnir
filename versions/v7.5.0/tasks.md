# Favnir v7.5.0 Tasks

Date: 2026-05-28
Theme: Rune 読み込みのセルフホスト化（TOML パーサー + Rune ローダー）

---

## Phase A: VM プリミティブ追加

### vm.rs

- [x] A-1: `IO.path_join_raw(base: String, segment: String) -> String`
- [x] A-2: `IO.home_dir_raw() -> Option<String>`
- [x] A-3: `IO.cwd_raw() -> String`
- [x] A-4: `IO.is_dir_raw(path: String) -> Bool`

### checker.rs

- [x] A-5: `("IO", "path_join_raw")` → `Type::String`
- [x] A-6: `("IO", "home_dir_raw")` → `Type::Option(Box::new(Type::String))`
- [x] A-7: `("IO", "cwd_raw")` → `Type::String`
- [x] A-8: `("IO", "is_dir_raw")` → `Type::Bool`

---

## Phase B: runes/toml/toml.fav

- [x] B-1: `runes/toml/rune.toml` 作成
- [x] B-2: `runes/toml/toml.fav` 作成
  - [x] B-2-1: `type RuneMeta = { name, version, entry, effects }` 型定義
  - [x] B-2-2: `type ParseState = { section, doc }` 型定義
  - [x] B-2-3: `fn classify_line` — empty/comment/section/keyval
  - [x] B-2-4: `fn extract_section_name` — `[name]` → `"name"`
  - [x] B-2-5: `fn strip_quotes` — `"csv"` → `csv`
  - [x] B-2-6: `fn parse_array_inner` — `"Io","DbRead"` → comma-joined
  - [x] B-2-7: `fn parse_val_flat` — raw value → flat string
  - [x] B-2-8: `fn process_line_section / process_line_keyval / process_line`
  - [x] B-2-9: `public fn parse(src: String) -> Result<Map<String, String>, String>`
  - [x] B-2-10: `public fn get_str`
  - [x] B-2-11: `public fn get_arr`
  - [x] B-2-12: `public fn read_rune_meta(path: String) -> Result<RuneMeta, String> !IO`
- [x] B-3: `fav check runes/toml/toml.fav` — no errors

**実装ノート**:
- `Map<K, V>` をフィールドに持つ named type は使えない（parser が `,` をエラーとする）
  → TomlDoc は型エイリアスとせず `Map<String, String>` を直接使用
- Favnir レコード型フィールドはコンマでなく改行区切り

---

## Phase C: runes/rune_loader/loader.fav

- [x] C-1: `runes/rune_loader/rune.toml` 作成
- [x] C-2: `runes/rune_loader/loader.fav` 作成
  - [x] C-2-1: `type ResolveResult = Found | NotFound | Error` 型定義
  - [x] C-2-2: `type SemVer = { major, minor, patch }` 型定義
  - [x] C-2-3: `fn parse_semver`
  - [x] C-2-4: `fn semver_ge / semver_lt / semver_eq`
  - [x] C-2-5: `public fn matches_constraint`（`^X.Y` / exact）
  - [x] C-2-6: `fn extract_entry_from_toml` — rune.toml から entry を取得
  - [x] C-2-7: `fn read_entry`
  - [x] C-2-8: `fn resolve_from_registry`
  - [x] C-2-9: `public fn installed_versions`
  - [x] C-2-10: `public fn resolve_version`
  - [x] C-2-11: `public fn resolve`
- [x] C-3: `fav check runes/rune_loader/loader.fav` — no errors

**実装ノート**:
- `List.nth` は未実装 → `List.first(List.drop(parts, n))` で代用
- `Option.some(v)` / `Option.none()` が constructor（パターンは `None`/`Some(v)`）
- raw string `r#"..."#` 内で Favnir コードに `"#"` が含まれると早期終了 → `r##"..."##` を使う

---

## Phase D: テスト（driver.rs）

### toml_rune_tests（3 件）

- [x] D-1: `toml_parse_simple_test` — `"[rune]"` セクション名抽出
- [x] D-2: `toml_parse_array_test` — `parse_array_inner("Io, DbRead")` → length 2
- [x] D-3: `toml_get_str_missing_test` — 存在しないキー → `None`

### rune_loader_tests（3 件）

- [x] D-4: `loader_semver_caret_matches_test` — `"1.2.3"` ∈ `"^1.0"`, `"2.0.0"` ∉ `"^1.0"`
- [x] D-5: `loader_semver_exact_test` — `"1.2.3"` == `"1.2.3"`, `"1.2.4"` != `"1.2.3"`
- [x] D-6: `loader_parse_semver_test` — `"2.10.3"` → major+minor+patch == 15

---

## Phase E: ドキュメント

- [x] E-1: `site/content/docs/runes/toml.mdx` 作成
- [x] E-2: `site/content/docs/runes/rune-loader.mdx` 作成

---

## Phase F: 最終確認

- [x] F-1: `cargo test` — 1087 tests passing（+6 新規）
- [x] F-2: `fav check runes/toml/toml.fav` — no errors
- [x] F-3: `fav check runes/rune_loader/loader.fav` — no errors
- [x] F-4: このファイルを完了状態に更新
- [ ] F-5: commit

---

## 完了条件

- `runes/toml/toml.fav` が `fav check` を通る ✓
- `runes/rune_loader/loader.fav` が `fav check` を通る ✓
- IO プリミティブ 4 件追加済み ✓
- 統合テスト 6 件追加 ✓
- 既存テスト 1081 件が全件通る（1087 passing） ✓
- ドキュメント 2 ページ追加 ✓
