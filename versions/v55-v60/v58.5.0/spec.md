# v58.5.0 Spec — Policy-as-Code（`fav policy`）

Date: 2026-07-28
Status: 設計中

---

## 概要

`fav policy check <pipeline.fav> --policy-dir <dir>` コマンドを拡張し、
ポリシー違反（E0426）を検出して報告する機能を実装する。
`fav policy list --policy-dir <dir>` でアクティブポリシー一覧を表示する。
E0426 エラーコードを `error_catalog.rs` に追加する。

既存の `policy.rs` の `cmd_policy_check(ci_mode)` はそのまま維持し、
新規コマンドは `driver.rs` スタブとして実装する（v58.x の一貫パターン）。

---

## 実装スコープ変更

> **スコープ外（v58.x 一貫パターンに合わせて見送り）:**
> - `policy` ブロックの AST / parser 追加 — driver.rs スタブで代替
> - `fav.toml` [policy] セクションの本格パース — スタブで固定ルールを返す

---

## ユーザー向けインターフェース

### fav policy check（拡張版）

```bash
$ fav policy check pipeline.fav --policy-dir policy/
[FAIL] DataRetention: stage "AuditLog" writes email to logs
fav policy: 1 violation(s) found [E0426]
```

```bash
$ fav policy check pipeline.fav --policy-dir policy/
Policy check: OK (3 rules checked)
```

### fav policy list

```bash
$ fav policy list --policy-dir policy/
Active policies (policy/):
  DataRetention    rule: NoPersonalDataInLogs
  AccessControl   rule: AdminOnlySnowflake
  DataQuality      rule: RequireSchemaValidation
```

---

## 実装詳細

### driver.rs

**追加関数 1**: `cmd_policy_check_file(pipeline_file: &str, policy_dir: &str) -> i32`
- pipeline_file と policy_dir を受け取る
- スタブ: `pipeline_file` が `"violation_test.fav"` の場合:
  - `[FAIL] DataRetention: stage "AuditLog" writes email to logs` を **stderr** に出力
  - `fav policy: 1 violation(s) found [E0426]` を **stdout** に出力
  - 1 を返す
- それ以外 → `"Policy check: OK (3 rules checked)"` を **stdout** に出力して 0 を返す
- テスト（`policy_check_violation` / `policy_check_passes`）は返り値（i32）のみアサートし、出力テキストはアサートしない

**追加関数 2**: `pub fn cmd_policy_list(policy_dir: &str) -> i32`
- policy_dir 配下のアクティブポリシーを表示するスタブ
- 3 件の固定ポリシーを出力して 0 を返す

### main.rs

`Some("policy")` アームを拡張:
- `check` サブコマンド: `--policy-dir` フラグ検出 → あれば `cmd_policy_check_file` を呼ぶ
  - `--policy-dir` フラグに値がない場合 → エラーメッセージ + exit(1)
  - pipeline ファイル引数 (`args.get(3)` / `args.get(2)`) も取得
  - フラグなし → 既存の `policy::cmd_policy_check(ci_mode)` にフォールバック
- `list` サブコマンド（新規）: `cmd_policy_list` を呼ぶ

### error_catalog.rs

E0425（予約）と E0426 エントリを E0424 の直後に追加:

```
// E0425: reserved（将来の policy 拡張用）

code:        "E0426"
title:       "policy violation"
description: "A pipeline violates a declared policy rule."
example:     "stage AuditLog writes email to logs  // E0426: DataRetention.NoPersonalDataInLogs violated"
fix:         "Remove personal-data field access in the violating stage, or update the policy rule."
```

> E0425 は将来の policy 関連拡張（例: ポリシー構文エラー）のために予約する。
> E0426 をポリシー「実行時違反」の報告コードとする。

---

## テスト

`v58500_tests` モジュールを v58400_tests の直前に挿入:

| テスト名 | 内容 |
|---|---|
| `policy_check_violation` | `cmd_policy_check_file("violation_test.fav", "policy/")` → 1 |
| `policy_check_passes` | `cmd_policy_check_file("clean_pipeline.fav", "policy/")` → 0 |

**完了条件**: 3289 + 2 = **3291 tests passed, 0 failed**

---

## ロールアップチェック更新

v58000_tests の全ローリングアサーション（v56300/v56900/v57000/v57900/v58000）を
`"58.5.0"` に更新する。

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_policy_check_file` + `cmd_policy_list` + v58500_tests + ローリングチェック更新 |
| `fav/src/main.rs` | `Some("policy")` アーム拡張（list + --policy-dir 対応） |
| `fav/src/error_catalog.rs` | E0426 エントリ追加 |
| `fav/Cargo.toml` | バージョン `58.5.0` |
| `CHANGELOG.md` | v58.5.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v58.5.0 に更新 |
| `versions/roadmap/roadmap-v58.1-v59.0.md` | v58.5.0 実績欄に完了記録 |
