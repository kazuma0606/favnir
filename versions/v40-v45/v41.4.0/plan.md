# v41.4.0 実装プラン — ガード付き match

**目標**: ガード付き match アームを checker.fav に統合する

---

## フェーズ 1 — ast_lower_checker.rs 変更

1. `v3` ヘルパーの直後に `v4` ヘルパーを追加
2. `lower_arms` 関数を変更:
   - `arm.guard.is_some()` の場合 → `v4("EArmG", pat, guard, body, acc)`
   - guard なし → 既存の `v3("EArm", pat, body, acc)` 維持

**影響範囲**: `ast_lower_checker.rs` のみ（checker.fav 呼び出し側）

---

## フェーズ 2 — checker.fav: EArmG バリアント + トラバーサル

1. `Expr` 型に `| EArmG(Pat, Expr, Expr, Expr)` を追加（`EArmNil` 直後）
2. 各関数に EArmG ケースを追加:
   - `infer_arms_effects` — guard 式のエフェクトも収集
   - `check_rebind` — body/rest を再帰チェック
   - `check_w006_arms` — body/rest の W006 チェック
   - `infer_arms` — body の型推論（EArm と同じロジック）
   - `collect_arm_ctors` — **ガード付き `_` をスキップ**（新規ロジック）

---

## フェーズ 3 — driver.rs テスト更新

1. `v41300_tests::cargo_toml_version_is_41_3_0` をスタブ化
2. `v41400_tests` モジュールを追加（3 件）

---

## フェーズ 4 — バージョンドキュメント更新

1. `Cargo.toml`: `version = "41.4.0"`
2. `CHANGELOG.md`: `[v41.4.0]` エントリ追加
3. `versions/roadmap/roadmap-v41.1-v42.0.md`: v41.4.0 を COMPLETE にマーク
4. `versions/current.md`: 最新安定版を v41.4.0 に更新

---

## 実装順序

```
ast_lower_checker.rs (v4 + lower_arms)
  → checker.fav (EArmG 追加 + 各関数)
  → cargo test（中間確認）
  → driver.rs (v41300スタブ化 + v41400追加)
  → Cargo.toml バージョン bump
  → CHANGELOG.md 更新
  → cargo test（最終確認）
```

---

## リスク評価

| リスク | 影響度 | 対処 |
|---|---|---|
| EArmG の `_ ` フィールドインデックスずれ | HIGH | spec §2.2〜§2.6 の `_0`〜`_3` を正確に使用 |
| 既存 EArm のパスが壊れる | MED | EArm ケースは一切変更しない |
| `infer_arms` の EArmG で型推論が EArm と乖離 | LOW | EArm と同じロジックを流用 |
| guard なし arm が EArmG として lowering される | HIGH | `arm.guard.is_some()` で分岐（Rust 側） |
