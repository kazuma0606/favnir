# v60.0.0 Plan — Enterprise 1.0 宣言 ★クリーンアップ

Date: 2026-07-30
Status: 未着手

---

## 実装方針

v60.0.0 は「コード追加なし」の宣言・安定化バージョン。

実装は以下の順序で行う:

1. **Cargo.toml バージョン更新**（最初に実施）
2. **CHANGELOG.md 更新**（v60.0.0 エントリ追加）
3. **MILESTONE.md 更新**（Enterprise 1.0 正式宣言文エントリ追加）
4. **README.md 更新**（Enterprise 1.0 リリース文に更新）
5. **driver.rs 更新**（v60000_tests 追加 + rolling check 更新）
   - `include_str!` 参照ファイル（CHANGELOG.md / MILESTONE.md / README.md）は T2〜T4 完了後に追加する
6. **テスト実行確認**（3330 tests pass）
7. **cargo clean**（クリーンアップ）
8. **事後処理**（`roadmap-v59.1-v60.0.md` / `roadmap-v55.1-v60.0.md` の両方を更新 / `current.md` 更新 / tasks.md COMPLETE）

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `59.9.0` → `60.0.0` |
| `CHANGELOG.md` | v60.0.0 エントリ追加 |
| `MILESTONE.md` | `## v60.0.0（2026-07-30）— Enterprise 1.0` 正式エントリ追加 |
| `README.md` | Enterprise 1.0 "予定" → "宣言" 更新 |
| `fav/src/driver.rs` | v60000_tests 4 件追加 + rolling check 8 件更新 |

---

## Rolling Check 更新詳細

### 更新前（v59.9.0）→ 更新後（v60.0.0）

```
version = "59.9.0"  →  version = "60.0.0"
should be 59.9.0    →  should be 60.0.0
```

対象モジュール（8 件）:
- v59000_tests、v58900_tests、v58000_tests、v57900_tests
- v57000_tests、v56900_tests、v56300_tests、v59900_tests

`replace_all` を使い、コメント行（`// -- vXXXXX_tests (vX.Y.Z) --` 形式）は対象外。

### v60000_tests の rolling check 追加について

v60000_tests に追加する `cargo_toml_version_is_60_0_0` は、v60.1.0 以降の rolling check プール（9件目）に入る。

---

## MILESTONE.md 更新の注意

現在の `## v60.0.0（予定）— Enterprise 1.0` エントリを削除して正式版に置き換える。
正式版のエントリには宣言文（引用文）+ v56〜v59 達成内容のリストを追記する。

---

## cargo clean タイミング

テスト全通過確認後（T6 完了後）に実行する。

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

clean 後にビルドが通ることを確認する（`cargo build` で確認）。

---

## テスト数見込み

| フェーズ | テスト数 |
|---|---|
| ベース（v59.9.0） | 3326 |
| v60000_tests 4 件追加 | 3326 + 4 = **3330** |
