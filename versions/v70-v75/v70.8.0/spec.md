# v70.8.0 Spec — `fav doctor` 強化（プロジェクト健全性チェック）

Date: 2026-08-09
Status: 計画中

---

## Background

`cmd_doctor_run` は driver.rs に既存（v54.5.0 実装）。現状のチェック項目は:
1. fav バージョン表示
2. Rust toolchain 表示
3. fav.toml 存在確認
4. .fav-cache 存在確認

本バージョンでは v70 スプリントで追加された機能を活かし、以下の検査項目を追加する:
- **Paper Rune 検出**: `rune.toml` が存在するが実装 `.fav` ファイルが空のディレクトリ
- **CHANGELOG 整合性**: 現行バージョンのエントリが `CHANGELOG.md` に存在するか

---

## Goals

1. `doctor_check_paper_rune(rune_dir: &str) -> DoctorCheck` を driver.rs に追加
2. `doctor_check_changelog_entry(changelog_content: &str, version: &str) -> DoctorCheck` を driver.rs に追加
3. `v708000_tests` モジュールを driver.rs 末尾に追加（2 テスト）
4. テスト 2 件追加 → 3578 tests

**注**:
- `cmd_doctor_run` への自動統合はしない（コア関数のみ追加し、テストで単体検証）
- `--fix` フラグ実装はスコープ外（v70.9.0 以降）— ロードマップ v70.8.0 セクションに記載はあるが、実装コストに対して効果が小さいため延期
- self-hosting coverage チェックは v70.7.0 で `compute_self_coverage()` として既に独立実装済みのため本バージョンでは `cmd_doctor_run` への統合を行わない（v70.9.0 安定化時に統合予定）

---

## Syntax / API Examples

```bash
$ fav doctor
[OK]   fav version: 70.8.0
[OK]   Rust toolchain: stable (toolchain info unavailable)
[WARN] fav.toml: not found in current directory (run fav new <name> to create one)
[OK]   .fav-cache: not present (will be created on first run)
```

（出力プレフィックスは `DoctorStatus::prefix()` の実装: `[OK]  ` / `[WARN]` / `[FAIL]`）

### `doctor_check_paper_rune` の動作

```
rune_dir/
  rune.toml     ← 存在
  <name>.fav    ← 空（0 バイトまたは空白のみ）
```
→ `DoctorStatus::Fail`、detail: `"Paper Rune detected (rune.toml exists but implementation is empty)"`

```
rune_dir/
  rune.toml     ← 存在
  <name>.fav    ← 非空
```
→ `DoctorStatus::Ok`

### `doctor_check_changelog_entry` の動作

```
changelog_content.contains("[v70.8.0]") == true  → DoctorStatus::Ok
changelog_content.contains("[v70.8.0]") == false → DoctorStatus::Fail
```

---

## Implementation Details

### `doctor_check_paper_rune`

```rust
pub fn doctor_check_paper_rune(rune_dir: &str) -> DoctorCheck {
    let dir = std::path::Path::new(rune_dir);
    let rune_toml = dir.join("rune.toml");
    if !rune_toml.exists() {
        return DoctorCheck { status: DoctorStatus::Ok, label: format!("rune: {rune_dir}"), detail: String::new() };
    }
    // rune.toml が存在する場合、実装 .fav ファイルを探す
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("rune");
    let fav_file = dir.join(format!("{name}.fav"));
    let is_empty = if fav_file.exists() {
        std::fs::read_to_string(&fav_file).map(|s| s.trim().is_empty()).unwrap_or(true)
    } else {
        true
    };
    if is_empty {
        DoctorCheck {
            status: DoctorStatus::Fail,
            label: format!("rune: {rune_dir}"),
            detail: "Paper Rune detected (rune.toml exists but implementation is empty)".to_string(),
        }
    } else {
        DoctorCheck { status: DoctorStatus::Ok, label: format!("rune: {rune_dir}"), detail: String::new() }
    }
}
```

### `doctor_check_changelog_entry`

```rust
pub fn doctor_check_changelog_entry(changelog_content: &str, version: &str) -> DoctorCheck {
    let marker = format!("[{version}]");
    if changelog_content.contains(&marker) {
        DoctorCheck { status: DoctorStatus::Ok, label: "CHANGELOG.md".to_string(), detail: format!("{version} entry found") }
    } else {
        DoctorCheck { status: DoctorStatus::Fail, label: "CHANGELOG.md".to_string(), detail: format!("{version} エントリが存在しない") }
    }
}
```

---

## Success Criteria

- [ ] `doctor_detects_paper_rune`: `DoctorStatus::Fail` が返ることを assert
- [ ] `doctor_detects_missing_changelog_entry`: `DoctorStatus::Fail` が返ることを assert
- [ ] `cargo test v708000` で 2 件 pass
- [ ] `cargo test` 全体で 3578 tests pass（0 failures）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `doctor_check_paper_rune` + `doctor_check_changelog_entry` + `v708000_tests` |
| `fav/Cargo.toml` | `version` を `"70.7.0"` → `"70.8.0"` |
| `CHANGELOG.md` | v70.8.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v70.8.0 に更新 |
