# v69.4.0 実装計画 — `fav migrate --ai`

Status: DRAFT
Version: 69.4.0

---

## 実装ステップ

### Step 1: `driver.rs` — ヘルパー関数と `cmd_migrate_ai` の追加

`migrate_enterprise_import` 関数の直前（`/// v59.5.0:` ドキュメントコメント行の前）に挿入する。

1. `fn extract_stage_name(line: &str) -> Option<&str>` — private ヘルパー
2. `pub fn cmd_migrate_ai(src, output, dry_run, interactive)` — 本体

挿入後の順序:

```
fn extract_stage_name(...)        ← 新規追加（private）
pub fn cmd_migrate_ai(...)        ← 新規追加（pub）
/// v59.5.0: migrate_enterprise_import
pub fn migrate_enterprise_import(...)   ← 既存
```

### Step 2: `main.rs` — use 宣言に `cmd_migrate_ai` を追加

`cmd_migrate_dry_run, migrate_enterprise_import` を含む `use` 宣言行に `cmd_migrate_ai,` を追加。

### Step 3: `main.rs` — `Some("migrate")` アームにフラグ追加

変数宣言ブロック（`let mut in_place = false;` の前）に追加:
- `let mut ai_mode = false;`
- `let mut interactive = false;`
- `let mut output_path: Option<String> = None;`

フラグ解析ループ（既存 `"--in-place"` の前）に追加:
- `"--ai"` アーム
- `"--interactive"` アーム
- `"--output"` アーム（値付きフラグ）

ディスパッチ（`if to_version.as_deref() == Some("enterprise")` の直前）に追加:
- `if ai_mode { ... cmd_migrate_ai(...); return; }`

### Step 4: ビルド確認

```sh
cargo build 2>&1 | grep "^error"
```

エラーゼロを確認する。警告は許容するが、未使用 import 警告が出る場合は修正する。

### Step 5: テスト確認

```sh
cargo test --bin fav -- --test-threads=8
```

3545 tests passed, 0 failed を確認する（テスト数変化なし）。

### Step 6: 手動動作確認

```sh
echo 'public stage Transform: String -> String = |s| { s }' > /tmp/test.fav
./target/debug/fav migrate --ai /tmp/test.fav --dry-run
```

`Suggestions:` または `[INFO]` を含む出力が得られることを確認する。

---

## 依存関係

- Step 1 完了後に Step 2・3 を並行実施可能
- Step 4 は Step 1〜3 完了後
- Step 5・6 は Step 4 完了後

---

## sub-version ポリシー

v69.x では Cargo.toml / CHANGELOG.md は変更しない。v70.0.0 宣言時に一括更新する。
