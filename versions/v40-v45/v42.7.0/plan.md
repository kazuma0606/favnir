# v42.7.0 実装計画 — `fav monitor`

## 目標

`fav monitor` CLI コマンドを stub 実装する。
`main.rs` にコマンドルーティングと `cmd_monitor` 関数を追加し、`driver.rs` にテスト 2 件を追加する。

---

## T0 — 事前確認

- [ ] `cargo test` が 2891 tests / 0 failures であることを確認
- [ ] `fav/Cargo.toml` version が `42.6.0` であることを確認
- [ ] `fav/src/main.rs` の `Some("watch")` アーム行番号を記録
- [ ] `fav/src/main.rs` の最終関数末尾行番号を記録（`cmd_monitor` 追加位置）
- [ ] `fav/src/driver.rs` の `v42600_tests` 閉じ `}` 行番号を記録
- [ ] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.7.0 エントリが存在することを確認

---

## T1 — `main.rs` — `Some("monitor")` アーム追加

`Some("watch")` アームの近傍（`Some("profile")` の直後など）に追加:

```rust
Some("monitor") => cmd_monitor(&args),
```

---

## T2 — `main.rs` — `cmd_monitor` 関数追加

既存の `cmd_watch` / `cmd_profile` 関数の直後に追加:

```rust
/// v42.7.0: fav monitor — パイプライン監視（stub）
/// 実際のメトリクス収集は v43.x 以降で実装。
fn cmd_monitor(_args: &[String]) {
    println!("fav monitor — pipeline metrics (stub)");
    println!("Throughput, event count, and latency monitoring will be available in v43.x.");
}
```

---

## T3 — `driver.rs` — `v42700_tests` モジュール追加

`v42600_tests` の閉じ `}` の直前（降順配置）に挿入:

```rust
// -- v42700_tests (v42.7.0) -- fav monitor --
mod v42700_tests {
    #[test]
    fn cargo_toml_version_is_42_7_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("42.7.0"), "Cargo.toml must contain 42.7.0");
    }

    #[test]
    fn monitor_cmd_exists() {
        let main_src = include_str!("../src/main.rs");
        assert!(main_src.contains("cmd_monitor(&args)"), "main.rs must contain cmd_monitor(&args)");
        assert!(main_src.contains("cmd_monitor"), "main.rs must contain cmd_monitor");
    }
}
```

注意: `v42600_tests` の `cargo_toml_version_is_42_6_0` をスタブ化（`assert!(true)`）してから追加する。

---

## T4 — `fav/Cargo.toml` バージョン bump

`version = "42.6.0"` → `version = "42.7.0"`

---

## T5 — `CHANGELOG.md` 更新

`[v42.6.0]` の直前に `[v42.7.0]` エントリを追加:

```markdown
## [v42.7.0] — 2026-07-12

### Added
- `fav monitor` コマンド — パイプライン監視 stub（スループット / イベント数 / レイテンシ表示は v43.x 以降）
- `cmd_monitor` 関数追加（`main.rs`）
- `v42700_tests`: `cargo_toml_version_is_42_7_0` / `monitor_cmd_exists`

### Notes
- 実際のメトリクス収集・TUI 表示は v43.x 以降で実装
```

---

## T6 — テスト実行・確認

- [ ] `cargo test` 実行
- [ ] failures = 0 を確認
- [ ] テスト数 = 2893 を確認（2891 + 2 件）
- [ ] `v42700_tests` 2 件 pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [ ] `versions/current.md` を v42.7.0（最新安定版、2893 tests）・v42.8.0（次に切る版）に更新
- [ ] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.7.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）、実績 2893 に修正
- [ ] `versions/v40-v45/v42.7.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## ファイル変更サマリー

| ファイル | 変更種別 |
|---|---|
| `fav/src/main.rs` | 変更 |
| `fav/src/driver.rs` | 変更 |
| `fav/Cargo.toml` | 変更 |
| `CHANGELOG.md` | 変更 |
| `versions/current.md` | 変更 |
| `versions/roadmap/roadmap-v42.1-v43.0.md` | 変更 |
