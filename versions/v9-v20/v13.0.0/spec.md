# Favnir v13.0.0 仕様書

Date: 2026-06-09
Theme: 言語信頼性宣言 — E2E デモ全通過 + v12.x 完成宣言

---

## 概要

v12.1.0〜v12.10.0 で以下の問題をすべて解消した：

| # | 問題 | 対応 |
|---|---|---|
| C-1 | `bind` 再束縛がエラーにならない | v12.1.0 E0018 |
| C-2 | `bind _` で Result が黙って捨てられる | v12.2.0 W006 |
| C-3 | `bind` が monadic bind でなかった | v12.3.0 LegacyBindCheck |
| C-4 | `seq` pipeline が fail-fast でなかった | v12.4.0 SeqStageCheck |
| M-1 | `fav run --verbose` / `--trace` がなかった | v12.5.0 |
| A-3 | `fav check --json` / `--show-types` がなかった | v12.5.0 |
| H-1/H-2 | Postgres Rune が TLS・詳細エラー未対応 | v12.6.0 |
| A-1/A-6 | `fav doc --builtins` / `fav explain` がなかった | v12.7.0 |
| A-5 | `fav scaffold` がなかった | v12.8.0 |
| CI | `fav test self/*.fav` が CI になかった | v12.9.0 |
| A-2 | 全エラーに `help:` がなかった | v12.10.0 |

v13.0.0 は上記修正の**「完成宣言」**バージョンである。
主な作業は：
1. E2E デモ（fav2py / airgap）の W006 修正 + 再確認
2. CHANGELOG.md に v12.1.0〜v12.10.0 の全変更を記録
3. README.md に v13.0.0 宣言を追記
4. バージョン番号の更新

---

## 機能 1: fav2py E2E デモ pipeline.fav の W006 修正

### 背景

`infra/e2e-demo/fav2py/src/pipeline.fav` の `LoadAndInsert` ステージに
`bind _ <- Postgres.execute_raw(...)` が 4 箇所存在する。
v12.2.0/v12.10.0 の W006 ルールにより `fav lint --deny-warnings` で exit 1 になる。

### 修正方針

`bind _ <- Postgres.execute_raw(...)` → `chain _ <- Postgres.execute_raw(...)` に変更。

- `chain _ <-` は `bind _ <-` と同じエラー伝播セマンティクスを持つが、
  W006 警告の対象にならない
- `--legacy` モードでも `chain` は正しく動作する
- `IO.println(...)` は `Unit` を返すため W006 の対象外（変更不要）

### 修正後の確認

```bash
./target/debug/fav lint --deny-warnings infra/e2e-demo/fav2py/src/pipeline.fav
# → exit 0 (no warnings)

./target/debug/fav check infra/e2e-demo/fav2py/src/pipeline.fav
# → no errors
```

---

## 機能 2: airgap E2E デモ 動作確認

`infra/e2e-demo/airgap/src/analyze.fav` は Postgres を使用せず、
W006 の対象となる `bind _ <- NS.fn(...)` パターンが存在しない。
`fav lint --deny-warnings` でクリーンであることを確認するのみ。

---

## 機能 3: CHANGELOG.md 更新

v12.1.0〜v12.10.0 の全変更を CHANGELOG.md に追記する。

各バージョンのエントリ形式：
```markdown
## [v12.X.0] — 2026-06-0X

### Added
- ...

### Changed
- ...

### Notes
- テスト: XXXX 件
```

---

## 機能 4: README.md 更新

以下を更新する：
- 冒頭のバージョン宣言行を v13.0.0 に更新
- 言語信頼性宣言文を追記：

```
v13.0.0（2026-06-09）で、言語信頼性宣言を完了しました。
型安全・エラー伝播・デバッグ可視性の三点において、
Favnir のランタイム挙動は型システムの宣言と一致することを保証します。
また、`fav check --json` と `fav doc --builtins --format json` を用いて
AI ツールが自律的にコードを修正できることを確認しました。
```

---

## 宣言内容

> 「型安全・エラー伝播・デバッグ可視性の三点において、
> Favnir のランタイム挙動は型システムの宣言と一致することを保証する。
> また、Claude Code / Codex 等の AI ツールが `fav check --json` と
> `fav doc --builtins --format json` を用いて自律的にコードを修正できることを確認する。」
>
> — Favnir v13.0.0 言語信頼性宣言

---

## テストケース

| テスト名 | 内容 |
|---|---|
| `version_is_13_0_0` | `CARGO_PKG_VERSION == "13.0.0"` |
| `fav2py_pipeline_no_w006` | pipeline.fav が `fav lint --deny-warnings` でクリーン |
| `airgap_pipeline_no_w006` | analyze.fav が `fav lint --deny-warnings` でクリーン |

---

## 完了条件

- [ ] fav2py/pipeline.fav の W006 がゼロ（`fav lint --deny-warnings` exit 0）
- [ ] airgap/analyze.fav の W006 がゼロ
- [ ] CHANGELOG.md に v12.1.0〜v12.10.0 + v13.0.0 の全エントリが記録されている
- [ ] README.md に v13.0.0 宣言文が追記されている
- [ ] `fav/Cargo.toml` version == `"13.0.0"`
- [ ] `cargo test` 全通過
- [ ] CI 全 green

---

## 非目標

- E2E デモ（airgap / fav2py）の AWS インフラ再構築
  （既存インフラは稼働中であり再 apply は不要）
- `--legacy` フラグの廃止（v13.0.0 では非推奨のまま残す）
- 新機能の追加（v13.0.0 は宣言バージョンであり機能追加なし）
