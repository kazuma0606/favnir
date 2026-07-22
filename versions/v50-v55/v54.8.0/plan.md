# Plan: v54.8.0 — MILESTONE.md Production 3.0 エントリ追加

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3199 passed, 0 failed を確認

cargo clippy -- -D warnings
# → warnings なしであることを確認

# v54800_tests が未存在を確認
rg -n "v54800_tests" fav/src/driver.rs  # → 0 件

# v54700_tests の行番号を確認（挿入位置）
rg -n "v54700_tests" fav/src/driver.rs

# MILESTONE.md に "Production 3.0" が未存在を確認
grep "Production 3.0" MILESTONE.md  # → 0 件

# Cargo.toml が 54.7.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "54.7.0"
```

---

## ステップ 2: `MILESTONE.md` — v55.0.0 エントリ追加

ファイル先頭（`# Favnir Milestones` の直後・`## v54.0.0` の直前）に追加:

```markdown
## v55.0.0（予定）— Production 3.0

> 「型安全なガード節、スケールする並列パイプライン、
>  保証されたデータ品質、そして考えを助ける開発体験。
>  Favnir はデータエンジニアが現場で選ぶ言語になった。
>
>  これが Favnir v55.0 — Production 3.0 の姿である。」

**Production 3.0** の宣言予定バージョン。（中略）

**v51〜v54 達成内容:**
- v51（DX 3.0）: 全エラーコード診断・LSP インレイヒント・trace/watch
- v52（Performance & Scale）: par 並列実行・バックプレッシャー・bench 回帰検出・WASM 最適化
- v53（Data Quality 2.0）: assert_schema・lineage 強化・audit-log・OTel 強化
- v54（Integration Sprint）: fav explain 全コード・watch-diff・CI 統合・dq-report・doctor
```

注意: v52 達成内容に `WASM 最適化` を含める（v52.0.0 セクションとの整合）。

---

## ステップ 3: `driver.rs` — `v54800_tests` 追加

`v54700_tests` の直前に追加（詳細は spec.md §実装スコープ §2 参照）:

- `milestone_has_production3`: `"Production 3.0"` + `"v55.0.0（予定）"` の 2 アサーション
- `milestone_has_v55`: `"## v55.0.0"` ヘッダーマッチ（単純な `"v55"` ではなく）

`cargo build` → コンパイルエラーなし確認（`include_str!` パス検証）。

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`version = "54.7.0"` → `version = "54.8.0"`

---

## ステップ 5: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# 期待値: 3201 passed, 0 failed
```

```bash
cargo clippy -- -D warnings
# 期待値: warnings なし
```

---

## ステップ 6: 後処理

- `CHANGELOG.md`: v54.8.0 エントリ追加（v54.7.0 の直上）
- `versions/current.md` を v54.8.0（3201 tests）に更新
- `roadmap-v54.1-v55.0.md` の v54.8.0 実績欄を COMPLETE に更新
- `Cargo.lock` が自動更新されていることを確認し、コミットに含める
- `tasks.md` を COMPLETE に更新（T0〜T6 全 `[x]`）

コードレビュー対応（実施済み）:
- [MED] `milestone_has_v55` が `"v55"` のみで偽陽性リスク → `"## v55.0.0"` ヘッダーマッチに変更
- [MED] `v55.0.0（予定）` の「予定」明示がテストで未検証 → `milestone_has_production3` に追加アサーション
- [LOW] MILESTONE.md の v52 達成内容に WASM 最適化が省略 → `WASM 最適化` を追記
