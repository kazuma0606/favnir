# v14.0.0 Plan — 能力型完成宣言

Date: 2026-06-11

---

## Phase A — CI self-check テスト追加

**ファイル**: `fav/src/driver.rs`

`v140000_tests` モジュールを追加し、以下のテストを実装:

1. `version_is_14_0_0` — `CARGO_PKG_VERSION == "14.0.0"`
2. `e0025_self_compiler_zero` — `self/compiler.fav` に E0025 が 0 件であることを確認:
   ```rust
   let src = include_str!("../self/compiler.fav");
   let prog = Parser::parse_str(src, "compiler.fav").expect("parse");
   let errors = check_bang_notation(&prog);
   assert!(errors.is_empty(), "compiler.fav must have 0 E0025");
   ```
3. `e0025_self_checker_zero` — `self/checker.fav` に E0025 が 0 件であることを確認（同様）
4. `e0023_and_e0025_both_zero_compiler` — E0023 + E0025 の両方が 0 件
5. `capability_context_design_complete` — v13.1.0〜v13.10.0 の全機能が揃っていることを smoke test で確認:
   - `check_bang_notation` が存在する
   - `check_ambient_errors` が存在する
   - `check_type_state_errors` が存在する
   - `migrate_effects_in_source` が存在する

---

## Phase B — CHANGELOG 更新

**ファイル**: `CHANGELOG.md`

v14.0.0 セクションを先頭に追加:

```markdown
## [14.0.0] — 2026-06-11 — 能力型完成宣言

### Breaking Changes
- `!Effect` 記法は非 legacy モードで E0025 エラーになる（v13.10.0 から）
- ambient effect 呼び出し（ctx なしの `IO.println` 等）は E0023 エラーになる（v13.8.0 から）

### New Features (v13.1.0〜v13.10.0 集約)
- `interface` 継承構文（`LoadCtx: CommonCtx`）
- `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` / `HttpClient` / `Io` / `Env` capability interface
- `LoadCtx` / `WriteCtx` / `MigrateCtx` コンテキスト interface（capability 充足チェック付き）
- `AppCtx` 具象型 + `Ctx.build` / `Ctx.mock` Rune
- `ctx.field.method()` フィールドアクセス構文
- `seq Pipeline(ctx)` — ctx 型推論
- E0024 型状態パターンチェック
- `Ctx { db: DbRead }` 糖衣構文（v13.10.0）
- `fav migrate --from-effects` 移行ツール（v13.10.0）

### Error Codes Added
- W008: ambient effect call（警告）
- E0020〜E0025: capability-context 系エラー
- W009: direct Rune call deprecated
- W010: effect migration requires manual review

### Migration
`fav migrate --from-effects <file>` で旧 `!Effect` 記法を自動変換。
`--legacy` フラグで移行期間中も旧記法を許容（今後廃止予定）。
```

---

## Phase C — README 更新

**ファイル**: `README.md`

1. 「Effects」セクションを「Capability Context」セクションに更新
2. 旧コード例:
   ```
   fn load() -> Result<Rows, String> !Postgres { ... }
   ```
   を新コード例:
   ```
   fn load(ctx: LoadCtx) -> Result<Rows, String> { ... }
   ```
   に置き換え
3. `fav migrate --from-effects` の使い方を追記
4. capability-context 設計の概要（1 段落）を追加

---

## Phase D — バージョンバンプ + テスト確認 + コミット

1. `fav/Cargo.toml` → `version = "14.0.0"`
2. `cargo test v140000` で Phase A テスト全件パス確認
3. `cargo test` で全件パス確認（目標: 1505+ tests）
4. `git commit -m "feat: v14.0.0 — 能力型完成宣言"`

---

## 実装順序

```
A (tests) ← B (CHANGELOG) — 並行可
         ← C (README) — 並行可
         ← D (bump+test+commit) ← A,B,C 完了後
```

---

## リスク・注意点

1. **`self/compiler.fav` に E0025 が残っている可能性**: `self/` ファイルは v13.8.0 で E0023 移行済みだが、`!Effect` 記法が Parser 的に AST に残っていても `effects` フィールドが空の場合がある。テスト実行で確認が必要。
2. **E2E デモの実行**: v14.0.0 本体には含めない。インフラ（ECS/RDS/S3）の準備が整ってから別途実行。
3. **既存 `!Effect` テストとの衝突**: lint テストやエフェクト関連の既存テストが `!Effect` 記法を使っている場合がある。それらは内部テスト（`--legacy` 経由でない直接の `check_bang_notation` テスト）なので問題なし。
4. **CHANGELOG / README は git tracked**: 変更を commit に含める。

---

## 完了後の状態

- v13.0.0（言語信頼性宣言）+ v14.0.0（能力型完成宣言）でFavnir のコア言語設計が安定宣言済みとなる
- 次の大きなマイルストーンは v15.0.0（未定）
