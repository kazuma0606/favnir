# v14.0.0 Spec — 能力型完成宣言

Date: 2026-06-11

---

## 概要

v13.1.0〜v13.10.0 で capability-context 設計の全コンポーネントが揃った。
v14.0.0 はその完成宣言バージョンであり、以下を確定する:

1. `--legacy` フラグなしでは `!Effect` 記法が完全に使えないことを CI レベルで確認
2. `self/compiler.fav` / `self/checker.fav` の E0025 件数がゼロであることをテストで保証
3. CHANGELOG / README を「能力型完成宣言」として更新
4. バージョン 14.0.0 に昇格

---

## 1. 完成条件の最終確認

| 確認項目 | 対応バージョン | 状態 |
|---|---|---|
| `interface` 継承構文（`LoadCtx: CommonCtx`）のコンパイル時チェック | v13.1.0 | ✓ |
| `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` interface 実装 | v13.2.0 | ✓ |
| `HttpClient` / `Io` / `Env` interface 実装 | v13.3.0 | ✓ |
| `LoadCtx` / `WriteCtx` / `MigrateCtx` による capability 充足チェック | v13.4.0 | ✓ |
| `AppCtx` + `Ctx.build` / `Ctx.mock` Rune 実装 | v13.5.0 | ✓ |
| `ctx.field.method()` 構文実装 + E2E デモ書き換え（型チェック通過） | v13.6.0 | ✓ |
| `seq` pipeline での ctx 型推論 + E0022 | v13.7.0 | ✓ |
| ambient effect 禁止（E0023）+ `self/` 全件移行 | v13.8.0 | ✓ |
| 型状態パターン統合 + lineage 解析更新 | v13.9.0 | ✓ |
| `!` 記法廃止 + 糖衣構文 + `fav migrate` ツール | v13.10.0 | ✓ |
| `--legacy` 以外で `!` 記法が完全に使えないことを CI で確認 | **v14.0.0** | |
| E2E デモ実際の実行 PASS=5 確認 | v14.0.0 以降 | |

---

## 2. CI self-check: E0025 ゼロ保証

`self/compiler.fav` と `self/checker.fav` に対して `check_bang_notation` を実行し、
E0025 が 0 件であることをテストで保証する。

```
// テスト形式
let errors = check_bang_notation(&prog);
assert!(errors.is_empty(), "compiler.fav must have 0 E0025 after capability migration");
```

これにより「`--legacy` なしで動作する Favnir セルフホスト実装には `!Effect` が一切含まれない」
ことがコンパイル時に自動検証される。

---

## 3. 能力型完成宣言

### 3-1. 宣言内容

「Favnir の副作用は通常の型システムで表現される。
`capability 引数がなければ純粋` が言語レベルで保証される。
`!Postgres` / `!AWS` 等のエフェクト型は廃止され、
`DbRead` / `DbWrite` / `StorageWrite` 等の capability interface で置き換えられた。
新しいクラウドサービスの追加は言語仕様の変更を必要とせず、
interface に `impl` を追加するだけで完了する。
Claude Code / Codex 等の AI ツールは `Ctx.mock(...)` によって
本番接続なしにパイプライン全体をテスト可能である。」

### 3-2. CHANGELOG 更新

`CHANGELOG.md` に v14.0.0 セクションを追加:
- capability-context 設計の完成
- E0023（ambient effect 禁止）、E0024（型状態チェック）、E0025（bang notation 廃止）
- `fav migrate --from-effects` による移行支援ツール
- `Ctx { db: DbRead }` 糖衣構文

### 3-3. README 更新

`README.md` の「エフェクト型」説明を capability-context 設計の説明に更新。
旧 `!Postgres` の例を `ctx: LoadCtx` 形式に置き換え。

---

## 4. E2E デモ（v14.0.0 以降）

`infra/e2e-demo/fav2py/` および `infra/e2e-demo/airgap/` の実際の実行（PASS=5 確認）は
v14.0.0 リリース後に別途実施する（インフラ変更を伴うため v14.0.0 本体には含めない）。
確認後に `versions/v14.0.0/tasks.md` の該当チェックボックスを更新する。

---

## 5. エラーコード一覧（v14.0.0 時点の capability-context 系）

| コード | タイトル | 導入バージョン |
|---|---|---|
| W008 | ambient effect call（警告） | v13.1.0 |
| E0020 | capability interface has no such method | v13.2.0 |
| E0021 | capability not in context | v13.4.0 |
| E0022 | ctx-aware pipeline called with wrong number of arguments | v13.7.0 |
| E0023 | ambient effect call is not allowed（エラー） | v13.8.0 |
| E0024 | type state mismatch | v13.9.0 |
| E0025 | bang notation removed | v13.10.0 |
| W009 | direct Rune call is deprecated | v13.2.0 |
| W010 | effect migration requires manual review | v13.10.0 |

---

## 6. 影響範囲

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v140000_tests` モジュール追加（CI self-check テスト） |
| `fav/Cargo.toml` | `version = "14.0.0"` |
| `CHANGELOG.md` | v14.0.0 セクション追加 |
| `README.md` | capability-context 説明更新 |
