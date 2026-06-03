## 概要

<!-- 何を・なぜ変えたかを 1〜3 行で説明してください -->

## 変更の種類

<!-- 該当するものにチェックを入れてください -->

- [ ] バグ修正（動作を変えない修正）
- [ ] 新機能
- [ ] 破壊的変更（既存の `.fav` ファイルの動作が変わる）
- [ ] ドキュメントのみ
- [ ] リファクタリング（動作変更なし）
- [ ] インフラ・CI

## 不変条件チェックリスト

<!-- [INVARIANTS.md](../INVARIANTS.md) に基づく必須確認項目です -->

**コア不変条件（I）**
- [ ] `cargo test bootstrap_full_self_hosting` が通る
- [ ] `fav check self/compiler.fav` がエラーなし
- [ ] `fav check self/checker.fav` がエラーなし
- [ ] `cargo test` の件数が変更前以上である

**変更内容に応じた追加確認**
- [ ] 新機能の場合、対応するテストが含まれている
- [ ] 新しい Rune を追加した場合、エフェクト宣言が `checker.fav` と `checker.rs` の両方に登録済み
- [ ] 構文を変更した場合、`compiler.fav` と `parser.rs` の両方を更新済み
- [ ] 破壊的変更の場合、CHANGELOG.md に記載済み

## テスト

<!-- どのテストで動作を確認したか記載してください -->

```bash
cargo test <テスト名>
```

## 関連 issue / バージョン

<!-- 例: versions/v10.2.0/tasks.md の "!Snowflake エフェクト追加" -->
