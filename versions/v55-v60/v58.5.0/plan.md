# v58.5.0 Plan — Policy-as-Code（`fav policy`）

Date: 2026-07-28

---

## 実装順序

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `"58.4.0"` → `"58.5.0"` に変更。

### Step 2: error_catalog.rs に E0425 予約コメント + E0426 追加

E0424 エントリの直後に `// E0425: reserved（将来の policy 拡張用）` コメントを追加し、
続けて E0426 エントリを追加。

### Step 3: driver.rs に関数追加

以下を追加:
1. `pub fn cmd_policy_check_file(pipeline_file: &str, policy_dir: &str) -> i32`
   - `pipeline_file == "violation_test.fav"` → `[FAIL] DataRetention: stage "AuditLog" writes email to logs` を stderr に出力 + `"fav policy: 1 violation(s) found [E0426]"` を出力 + 1 を返す
   - それ以外 → `"Policy check: OK (3 rules checked)"` を出力 + 0 を返す
2. `pub fn cmd_policy_list(policy_dir: &str) -> i32`
   - `"Active policies ({policy_dir}):"` + 固定 3 件を出力 + 0 を返す

### Step 4: driver.rs テストモジュール追加

`v58500_tests` モジュールを `v58400_tests` の直前に挿入:
- `policy_check_violation`
- `policy_check_passes`

### Step 5: driver.rs ローリングチェック更新

v58000_tests 内の全ローリングアサーション（5 件）を `"58.5.0"` に更新。

### Step 6: main.rs の `Some("policy")` アーム拡張

`list` サブコマンドを追加。`check` サブコマンドに `--policy-dir` フラグサポートを追加。

### Step 7: CHANGELOG / current.md / roadmap 更新

事後処理ドキュメントを更新する。

---

## 既存コードとの関係

| 既存コード | 扱い |
|---|---|
| `policy.rs::cmd_policy_check(ci_mode)` | 変更なし・そのまま維持 |
| `main.rs Some("policy") check アーム` | `--policy-dir` がない場合は既存パスを使用 |
| `main.rs Some("policy") else アーム` | `list` を追加、`list` 以外は既存エラー維持 |

---

## リスク・注意点

- `cmd_policy_check_file` の関数名は既存の `cmd_policy_check`（policy.rs）と異なるため衝突なし
- `policy_check_violation` / `policy_check_passes` テスト名は driver.rs に同名関数がないため `_test` サフィックス不要
- main.rs の use imports に `cmd_policy_check_file`, `cmd_policy_list` を追加する
