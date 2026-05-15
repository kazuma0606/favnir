- [x] 0
- [x] 1
- [x] 2
- [x] 3
- [x] 4
- [x] 5
- [x] 6
- [x] 7

完了:
- Phase 0: バージョン更新 (Cargo.toml v2.9.0, main.rs v2.9.0)
- Phase 1: E067 解消（collect 内 for）— checker.rs の E067 ガード削除、collect_yield_types 追加
- Phase 2: Type::Stream(Box<Type>) 追加 — checker.rs に Stream 型を追加
- Phase 3: VMValue::Stream と VM ハンドラ追加 — vm.rs に VMStream/VMValue::Stream/materialize_stream/Stream.* ハンドラ
- Phase 4: compiler.rs グローバル登録 — "Stream" を2箇所のグローバルリストに追加
- Phase 5: テスト追加（collect+for 3件 + Stream 7件 + checker 2件 = +12件）
- Phase 6: examples/stream_demo 作成 (fav.toml + src/main.fav)
- Phase 7: langspec.md 作成、progress.md 完了

最終テスト数: 637（v2.8.0 ベースライン 625 + 12）

注記:
- Stream.collect → Stream.to_list にリネーム（collect キーワードとの競合回避）
- Stream.from と Stream.of は同一実装（リストから有限ストリーム生成）
- Stream.gen で無限ストリーム生成（Stream.take 必須）
