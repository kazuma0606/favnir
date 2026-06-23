# v21.0.0 — Runtime Excellence マイルストーン宣言 タスク

## ステータス: DONE

---

## タスク一覧

### T1: `benchmarks/v21.0.0.json` — SLO 達成値記録

- [x] `benchmarks/v21.0.0.json` を作成（plan.md T1 の JSON に従う）
  - トップレベルキー `"metrics"` にフラットな数値を記録（v20.0.0.json 形式と統一）
  - `"slo_summary"` に目標・ベースライン・達成フラグを記録
  - 参考値を初期値とする（CI が master 上で実行されたとき実測値で上書きコミット）
- [x] `jq . benchmarks/v21.0.0.json` で valid JSON を確認
- [x] `jq .metrics benchmarks/v21.0.0.json` で `"metrics"` フィールドが取得できることを確認

---

### T2: `fav/Cargo.toml` バージョン更新

- [x] `version = "20.8.0"` → `"21.0.0"` に変更
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T3: `CHANGELOG.md` 更新

- [x] v20.2.0〜v20.8.0 の全エントリを追加（v20.1.0 エントリの上、各バージョン分）
  - v20.2.0 / v20.3.0 / v20.4.0 / v20.5.0 / v20.6.0 / v20.7.0 / v20.8.0 の 7 件すべて
- [x] v21.0.0 エントリを先頭に追加
- [x] 内容は plan.md の T3 セクションに従う
- [x] `grep "v20\.[2-8]\.0" CHANGELOG.md` で 7 件すべてが存在することを確認

---

### T4: `README.md` 更新

- [x] バージョンバッジ / 「現在のバージョン」を v21.0.0 に更新
- [x] Runtime Excellence セクションを Features 一覧に追加（plan.md T4 に従う）
- [x] バージョン履歴表に v20.2.0〜v21.0.0 を追加

---

### T5: `site/content/docs/performance/` MDX 作成

- [x] `site/content/docs/performance/runtime-excellence.mdx` を新規作成
  - SLO 達成表（5項目）
  - 各最適化へのセクションリンク（同ページ内アンカー優先、別ページは存在するもののみ）
  - plan.md の T5 セクションの MDX に従う
- [x] `site/content/docs/performance/nan-boxing.mdx` を新規作成（v20.3 解説）
- [x] `site/content/docs/performance/pushdown.mdx` を新規作成（v20.4 解説）

---

### T6: `fav/src/driver.rs` — `v210000_tests` 追加

- [x] `v208000_tests::version_is_20_8_0` に `#[ignore]` を追加
  - `#[cfg(not(target_arch = "wasm32"))]` → `#[test]` → `#[ignore]` の順序で追加（cfg の後に置く）
- [x] `v210000_tests` モジュールを追加（5件）
  - `version_is_21_0_0`
  - `changelog_has_v20x_entries`
  - `readme_mentions_nan_boxing`
  - `readme_mentions_pushdown`
  - `bench_v21_baseline_exists`
- [x] `cargo test v210000` — 5/5 PASS を確認

---

## テスト（v210000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_21_0_0` | Cargo.toml に `"21.0.0"` が含まれる |
| `changelog_has_v20x_entries` | CHANGELOG に v20.2.0 / v20.8.0 / v21.0.0 エントリが含まれる |
| `readme_mentions_nan_boxing` | README に "NaN-boxing" が含まれる |
| `readme_mentions_pushdown` | README に "pushdown" or "プッシュダウン" が含まれる |
| `bench_v21_baseline_exists` | `benchmarks/v21.0.0.json` が存在し `metrics` フィールドを含む |

---

## 完了条件チェックリスト

- [x] `benchmarks/v21.0.0.json` が存在し `"metrics"` フィールドを含む valid JSON
- [x] `fav/Cargo.toml` version が `21.0.0`
- [x] `CHANGELOG.md` に v20.2.0〜v20.8.0 の全エントリが含まれる
- [x] `CHANGELOG.md` に v21.0.0 エントリが含まれる
- [x] `README.md` に Runtime Excellence / NaN-boxing / pushdown の記載がある
- [x] `site/content/docs/performance/runtime-excellence.mdx` が存在する
- [x] `site/content/docs/performance/nan-boxing.mdx` が存在する
- [x] `site/content/docs/performance/pushdown.mdx` が存在する
- [x] `cargo test v210000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（exit 0）

---

## 優先度

```
T1（benchmarks/v21.0.0.json）  ← 最初（T6 のファイル存在テストに必要）
T2（Cargo.toml）               ← T1 と並列可
T3（CHANGELOG.md）             ← T1 と並列可
T4（README.md）                ← T1 と並列可
T5（runtime-excellence.mdx）   ← T1 と並列可
T6（driver.rs テスト）         ← T1〜T5 完了後
```

Rust コードへの変更は T2（バージョン） と T6（テスト）のみ。
