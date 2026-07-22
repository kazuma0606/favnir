# Spec: v54.5.0 — fav doctor 環境診断コマンド

Status: COMPLETE
Date: 2026-07-23

---

## 概要

`fav doctor` コマンドを追加する。
Rust バージョン・`fav` バージョン・`fav.toml` 有効性・`.fav-cache` 整合性を一括チェックし、
`[OK]` / `[WARN]` / `[FAIL]` プレフィクス付きの診断レポートを出力する。

---

## 実装スコープ

### 1. `driver.rs` — 型定義 + ロジック関数

#### `DoctorStatus` enum

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoctorStatus { Ok, Warn, Fail }
```

`prefix()` メソッドが固定幅 6 文字のプレフィクスを返す（出力アラインメント用）:
- `Ok`   → `"[OK]  "` (4 + 2 padding)
- `Warn` → `"[WARN]"`
- `Fail` → `"[FAIL]"`

#### `DoctorCheck` struct

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub status: DoctorStatus,
    pub label: String,
    pub detail: String,
}
```

#### `cmd_doctor_collect(checks: &[DoctorCheck]) -> String`

チェックリストを整形済みテキストに変換する純粋関数（環境非依存・テスト容易）:

```
[OK]    fav version: 54.5.0
[WARN]  rune kafka: version 2.1.0 declared but not installed
```

出力形式:
- `detail` が空 → `"<prefix>  <label>"`
- `detail` が非空 → `"<prefix>  <label>: <detail>"`
- 行間は `\n` で結合

#### `cmd_doctor_run() -> Vec<DoctorCheck>`

実行環境から診断チェックを収集する（環境依存・`cmd_doctor_collect` とは分離）:

1. **fav version**: `env!("CARGO_PKG_VERSION")` → 常に `Ok`
2. **Rust toolchain**: `RUSTUP_TOOLCHAIN` 環境変数 → 取得失敗時は fallback 文字列 → 常に `Ok`
3. **fav.toml**: カレントディレクトリに存在すれば `Ok`、なければ `Warn`
4. **.fav-cache**: カレントディレクトリに存在すれば `Ok (intact)`、なければ `Ok (will be created)`

### 2. `main.rs` — `fav doctor` コマンド

`Some("dq-report")` の直前（`Some("watch")` よりも前）に追加:

```rust
Some("doctor") => {
    let checks = driver::cmd_doctor_run();
    let report = driver::cmd_doctor_collect(&checks);
    println!("{report}");
}
```

### 3. `driver.rs` — `v54500_tests` 追加

`v54400_tests` の直前に追加（2 テスト）:

```rust
mod v54500_tests {
    fn cmd_doctor_passes_clean_env()   { /* cmd_doctor_collect([Ok, Ok]) → contains "[OK]" + "fav version" */ }
    fn cmd_doctor_detects_missing_rune() { /* cmd_doctor_collect([Ok, Warn]) → contains "[WARN]" + "rune kafka" */ }
}
```

注意: 両テストは `cmd_doctor_collect` を直接呼び、`cmd_doctor_run` は呼ばない
（`cmd_doctor_run` は cwd / 環境変数依存のため単体テスト困難）。
テスト `cmd_doctor_detects_missing_rune` は WARN フォーマットの検証に特化し、
コメントでその旨を明記する。

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `cmd_doctor_passes_clean_env` | `cmd_doctor_collect` に `Ok` チェック 2 件を渡した結果が `"[OK]"` と `"fav version"` を含む |
| `cmd_doctor_detects_missing_rune` | `cmd_doctor_collect` に `Warn` チェックを渡した結果が `"[WARN]"` と `"rune kafka"` を含む |

---

## バージョン更新

- `fav/Cargo.toml`: `"54.4.0"` → `"54.5.0"`

---

## 完了条件

1. `cargo test -j 8 -- --test-threads=8` → 3195 passed, 0 failed（ベース 3193 + 2 件追加）
2. `v54500_tests` 2 件 pass:
   - `cmd_doctor_passes_clean_env`
   - `cmd_doctor_detects_missing_rune`
3. `cargo test` 全通過後に `cargo clippy -- -D warnings` → 警告なし確認（順序重要）

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `DoctorCheck` / `DoctorStatus` / `cmd_doctor_collect` / `cmd_doctor_run` 追加 / `v54500_tests` 追加 |
| `fav/src/main.rs` | `fav doctor` コマンド追加 |
| `fav/Cargo.toml` | version 更新 |
| `fav/Cargo.lock` | version 更新に伴い自動更新 |
| `CHANGELOG.md` | v54.5.0 エントリ追加 |
| `versions/current.md` | v54.5.0 / 3195 tests に更新 |
| `versions/roadmap/roadmap-v54.1-v55.0.md` | v54.5.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `cmd_doctor_collect` と `cmd_doctor_run` を分離することで、テスト容易性を確保。
- `prefix()` は固定幅 6 文字（`[OK]  ` は 2 padding 付き）で出力アラインメントを保証。
  コメントにパディング意図を明記。
- v54.5.0 では rune インストール状態の実チェック（`fav.toml` の rune 宣言解析）は将来バージョンに延期。
  ロードマップのサンプル出力（`[WARN] rune kafka: ...`）は将来実装の想定例示。
  v54.5.0 の `cmd_doctor_detects_missing_rune` は `cmd_doctor_collect` の WARN フォーマット検証のみ実施。
- `cmd_doctor_run` の Rust バージョン取得は `RUSTUP_TOOLCHAIN` 環境変数からチャンネル名を取得。
  ロードマップサンプルの `"1.79.0"` は例示であり、実際の出力はチャンネル名文字列（例: `stable-x86_64-pc-windows-msvc`）。
  取得失敗時は `"stable (toolchain info unavailable)"` にフォールバック。
  `rustc --version` による正確なバージョン番号取得は将来バージョンで実施予定。
- サイト MDX（`site/content/docs/tools/doctor.mdx`）は v54.6.0 の README/CONTRIBUTING 整備と合わせて追加予定。
  v54.5.0 スコープには含まない。
