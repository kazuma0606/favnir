# v62.3.0 Plan — `fav build --target` クロスコンパイルサポート

Version: 62.3.0
Status: 未着手

---

## 実装順序

### Step 1: cranelift API 確認（T0 の一部、Cargo.toml 変更前）
`cranelift_codegen::isa::lookup_by_name("aarch64")` が cranelift 0.117 で利用可能かを確認する。
`target_lexicon` クレートが既存 Cargo.toml に登録されているかを確認する。
結果に応じて Step 2 の Cargo.toml 変更内容を決める。

### Step 2: Cargo.toml — `"arm64"` feature 追加
`cranelift-codegen` の features を `["x86", "arm64"]` に変更。
`target_lexicon` が必要な場合は同時に追加する。
`cargo build` でエラーなし確認。

### Step 3: `cranelift_aot.rs` — `lower_to_object_with_target` 追加
`impl CraneliftBackend` 内の `lower_to_object` の直後に `lower_to_object_with_target` を追加。
target 分岐（native / aarch64 / unsupported）を実装。
`pub(crate) fn lower_to_object_with_target_pub` ラッパーも追加。
`cargo build` でエラーなし確認。

### Step 4: `driver.rs` — `cmd_build_link_target` 追加・`cmd_build_link` 変更
`cmd_build_link` の直後に `cmd_build_link_target(src, out, target: Option<&str>)` を追加。
`cmd_build_link` を `cmd_build_link_target(src, out, None)` の薄いラッパーに変更。
`cargo build` でエラーなし確認。

### Step 5: `main.rs` — `--link` ブランチに `aot_target` 接続
`--link` ブランチで `target.contains('-')` チェックを行い、triple 形式なら `aot_target` として渡す。
`cmd_build_link` → `cmd_build_link_target` に切り替え。

### Step 6: `driver.rs` — `v62300_tests` 追加
`v62200_tests` の直前（ファイル先頭方向）に挿入。
`cargo test v62300` で 2 件 PASS 確認。

### Step 7: 全テスト
`cargo test -j 8 -- --test-threads=8` で 3388 tests passed, 0 failed を確認。

### Step 8: ドキュメント更新

---

## リスク

- **cranelift API の不確かさ**: Step 1 で API を確認してから Step 2（Cargo.toml）に進む。
  `lookup_by_name` が利用不可なら `target_lexicon` 追加が必要。
- **`target_lexicon` 未登録**: Cargo.lock を確認すれば cranelift の推移的依存として既に入っているか判断できる。
  入っていれば `Cargo.toml` への明示追加は不要な場合もある（ただし `use` には必要）。
- **`cmd_build_link` の変更**: 薄いラッパーにするだけなので `v62200_tests` は影響なし。
- **`target` 変数の graphql/proto/schema との衝突**: `"-"` 含むかどうかで AOT triple を判定するので衝突しない。
