- [x] 0
- [x] 1
- [x] 2
- [x] 3
- [x] 4
- [x] 5
- [x] 6
- [x] 7

完了内容
- Phase 0: `fav/Cargo.toml` を `3.1.0` に更新し、`fav --version` を追加
- Phase 1: `fav/src/docs_server.rs` を追加し、ローカル HTTP サーバを実装
- Phase 2: stdlib JSON catalog を追加し `/api/stdlib` で配信
- Phase 3: `fav/src/docs_assets/` に `index.html` / `app.js` / `style.css` を追加
- Phase 4: `driver.rs` に `get_explain_json` と空 explain payload を追加
- Phase 5: `fav docs [file] [--port N] [--no-open]` を CLI に統合
- Phase 6: docs server / stdlib JSON / explain JSON のテストを追加
- Phase 7: `langspec.md` と `migration-guide.md` を作成
