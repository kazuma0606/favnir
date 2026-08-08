# v62.4.0 Plan — AOT エフェクトディスパッチ最適化（Pure ステージのインライン化）

Version: 62.4.0
Status: 未着手

---

## 実装順序

### Step 1: `cranelift_aot.rs` — `AotStats` + `is_aot_pure` + `analyze_for_inlining` 追加
`AotStats` 構造体をファイル先頭側（`CraneliftBackend` 定義の直前）に追加。
`is_aot_pure` をモジュールレベル private 関数として追加（`lower_expr` の近くに配置）。
`analyze_for_inlining` を `impl CraneliftBackend` 末尾に追加。
`cargo build` でエラーなし確認。

### Step 2: `driver.rs` — `cmd_build_aot_stats` 追加
`cmd_build_link_target` の直後に追加。
`cargo build` でエラーなし確認。

### Step 3: `main.rs` — `--aot-stats` フラグ追加
`Some("build")` アームのループ内に `"--aot-stats"` アームを追加。
`let mut aot_stats = false;` を変数宣言部に追加。
`if aot_stats { ... } else if link { ... }` の順で分岐を追加。
`aot_stats` ブランチでは `file` 変数からファイルを読み込み `cmd_build_aot_stats(&src)` を呼ぶ
（`--link` ブランチと同様のファイル読み込みパターンを使用）。
`cargo build` でエラーなし確認。

### Step 4: `driver.rs` — `v62400_tests` 追加
`v62300_tests` の直前（ファイル先頭方向）に挿入。
`cargo test v62400` で 2 件 PASS 確認。

### Step 5: 全テスト
`cargo test -j 8 -- --test-threads=8` で 3390 tests passed, 0 failed を確認。

### Step 6: ドキュメント更新

---

## 設計メモ

### `is_aot_pure` の判定基準

`IRExpr` の exhaustive match:
- **pure**: `Lit`（Str を除く） / `Local` / `BinOp`（再帰） / `If`（再帰） / `Block`（再帰）
- **not pure（`_ => false`）**: Global / TrfRef / CallTrfLocal / Call / Match / FieldAccess /
  Closure / Collect / Emit / RecordConstruct / RecordSpread / Par / AssertSchema
- 特例: `Lit::Str` → `false`（`lower_lit` が非サポートとして Err を返す）

### テスト設計

`aot_pure_stage_inlined`:
- `fn add(a: Int, b: Int) -> Int { a + b }` → Lit/Local/BinOp のみ → pure → `inlined`
- `fn main() -> Bool { 1 + 2 == 3 }` → 同様に pure → `inlined`
- `stats.inlined` に `"add"` が含まれることを assert

`aot_effectful_stage_not_inlined`:
- `fn greeting() -> String { "hello" }` → `Lit::Str` を含む → `is_aot_pure = false` → `dispatched`
- `stats.dispatched` に `"greeting"` が含まれることを assert
- これによりテスト名の意図（非 pure な stage が dispatch 対象）を満たす

### ロードマップとの乖離

- `effects` フィールドによる判定 → `is_aot_pure` による IR 構造解析に変更（T6 の実績欄に明記）
- 「削減バイト数」表示 → v62.9.0 に後送り（T6 の実績欄に明記）
