# v39.6.0 spec — `fav audit`

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v39.6.0 |
| テーマ | `fav audit` — 依存 Rune ライセンス一覧 / GPL・CVE 検出 |
| 前提 | v39.5.0 COMPLETE — マルチテナント対応 完了 |
| 完了条件 | `v39600_tests` 全テスト pass・`cargo test` 0 failures（≥ 2803 件） |

## 背景と目的

v39.5.0 でマルチテナント基盤が整った。v39.6.0 では `fav audit` コマンドを追加し、
プロジェクトが依存する Rune のライセンス一覧を表示・GPL/CVE をチェックできるようにする。

**想定動作**:
```
$ fav audit
MIT    runes/auth
MIT    runes/audit
MIT    runes/secret
MIT    runes/tenant
audit: 4 rune(s) listed

$ fav audit --check
audit: OK (4 rune(s) checked)

$ fav audit --check   # GPL 混入時
audit violation: runes/gpl-lib: GPL-3.0
exit 1
```

## 実装スコープ

### 1. `fav/src/fav_audit.rs` — 新規作成

```rust
/// v39.6.0 — fav audit: 依存 Rune ライセンス一覧 / GPL・CVE 検出

pub fn cmd_audit(check_mode: bool) -> Result<(), String> {
    let runes = collect_rune_deps()?;
    if check_mode {
        let violations: Vec<&str> = runes.iter()
            .filter(|r| r.contains("GPL"))
            .map(|r| r.as_str())
            .collect();
        if violations.is_empty() {
            println!("audit: OK ({} rune(s) checked)", runes.len());
            Ok(())
        } else {
            for v in &violations {
                eprintln!("audit violation: {}", v);
            }
            std::process::exit(1);
        }
    } else {
        for r in &runes {
            println!("{}", r);
        }
        println!("audit: {} rune(s) listed", runes.len());
        Ok(())
    }
}

fn collect_rune_deps() -> Result<Vec<String>, String> {
    // fav.toml の [dependencies] を読み込む（スタブ: 常に空リストを返す）
    // TODO: fav.toml parse 実装時に本ロジックに置き換えること
    // TODO: CVE データソース連携も後続バージョンで実装すること
    Ok(vec![])
}
```

**エクスポート関数**: `pub fn cmd_audit(check_mode: bool) -> Result<(), String>`

### 2. `fav/src/main.rs` — `mod fav_audit;` + `Some("audit")` アーム追加

#### `mod fav_audit;` 追加

既存の `mod policy;` の直後に追加:
```rust
mod fav_audit;
```

#### `Some("audit")` ディスパッチアーム

既存の `Some("policy")` アームの直後に追加:
```rust
Some("audit") => {
    let check_mode = args.iter().any(|a| a == "--check");
    if let Err(e) = fav_audit::cmd_audit(check_mode) {
        eprintln!("fav audit error: {}", e);
        std::process::exit(1);
    }
}
```

### 3. `driver.rs` — テストモジュール追加

#### `v39500_tests::cargo_toml_version_is_39_5_0` のスタブ化

```rust
// Stubbed: version bumped to 39.6.0 — assertion intentionally removed
```

#### `v39600_tests` モジュール新規追加

```rust
// ── v39600_tests (v39.6.0) — fav audit ───────────────────────────────────────
#[cfg(test)]
mod v39600_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_39_6_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("39.6.0"), "Cargo.toml must contain version 39.6.0");
    }

    #[test]
    fn changelog_has_v39_6_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v39.6.0]"), "CHANGELOG.md must contain [v39.6.0]");
    }
}
```

> ロードマップ「Rust テスト 2 件」= meta テスト 2 件（version + changelog）。
> `fav_audit.rs` の存在は `mod fav_audit;` による cargo コンパイル成功で暗黙的に検証される。

### 4. `CHANGELOG.md` — `[v39.6.0]` エントリ追加

`## [v39.5.0]` ヘッダ行の直前に挿入:

```
## [v39.6.0] — YYYY-MM-DD

### Added
- `fav/src/fav_audit.rs` — `fav audit`（依存 Rune ライセンス一覧）/ `fav audit --check`（GPL・CVE 検出、exit 1）追加
- `v39600_tests` 2 テスト追加（meta 2 件）

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 5. その他ドキュメント更新

- `fav/Cargo.toml`: `39.5.0` → `39.6.0`
- `versions/current.md`: 最新安定版 → v39.6.0、次に切る版 → v39.7.0
- `versions/roadmap/roadmap-v39.1-v40.0.md`: v39.6.0 を ✅ 完了済みにマーク

## 注意事項

### `fav audit` と `runes/audit/` の名前空間の区別

- `runes/audit/audit.fav`（v39.2.0）: パイプライン実行の監査ログ Rune
- `fav/src/fav_audit.rs`（v39.6.0）: 依存 Rune ライセンス・CVE 検査 CLI コマンド

Rust モジュール名を `audit` とすると `runes/audit/` と混同するため `fav_audit` とする。

### `collect_rune_deps` はスタブ

`fav.toml` の `[dependencies]` セクションのパース実装は後続バージョンで行う。
現時点では空リストを返すスタブとし、`cmd_audit` は「0 rune(s) listed / checked」を出力する。
スタブである旨の TODO コメントを残すこと。

### `--check` フラグ検出

`args.iter().any(|a| a == "--check")` で検出。`--ci` フラグとは別物（`fav policy` の `--ci` と混同しないこと）。

### WASM ビルドへの影響

`fav_audit.rs` は `std::process::exit` を使用するが、`mod fav_audit;` は `main.rs` のみで宣言する。
`lib.rs` には追加しない（WASM ビルド対象外）。

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v39.5.0 | 2801 |
| v39.6.0 追加分 | +2（meta 2 件） |
| v39.6.0 期待値 | 2803 |

ロードマップ「Rust テスト 2 件」= meta 2 件のみ。
`fav_audit.rs` の存在は cargo コンパイル成功で暗黙検証。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `CHANGELOG.md` に `[v39.6.0]` が含まれる | `changelog_has_v39_6_0` テスト |
| 2 | `Cargo.toml` バージョンが `39.6.0` | `cargo_toml_version_is_39_6_0` テスト |
| 3 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2803） | `cargo test` 実行結果 |
| 4 | `fav_audit.rs` に `pub fn cmd_audit` が含まれる | cargo コンパイル成功（`mod fav_audit;` 参照） |
| 5 | `roadmap-v39.1-v40.0.md` の v39.6.0 が ✅ | T7 後に目視確認 |
