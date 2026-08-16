# v79.7.0 実装計画 — OSS 公開強化・コミュニティ整備

Date: 2026-08-16

---

## 実装順序

### Step 1: CONTRIBUTING.md 更新

既存の `CONTRIBUTING.md` 末尾に以下を追記する:

```markdown
## Execution Effects の追加手順（v3 対応）

新しいエフェクト（`!MyEffect`）を追加する場合は以下の手順に従ってください:

1. `fav/src/ast.rs` に `Effect::MyEffect` バリアントを追加
2. `fav/src/middle/checker.rs` の `ns_to_effect` / `builtin_ret_ty` を更新
3. `fav/src/backend/cranelift_aot.rs` のマッチアームを更新
4. `fav/pipelines/health-check.fav` を使ってヘルスチェックを実行

## PipelineInvariant（invariant）の追加手順

`contract` ブロックに新しい不変条件（invariant）を追加する場合:

1. `infra/e2e-demo/` の `contract.fav` に `invariant:` 節を追記する
2. `fav verify <contract.fav>` で静的検証を確認する

## fav verify の使い方

```bash
fav verify <pipeline.fav>
```

`fav verify` はパイプラインの不変条件（`contract` ブロック）を静的検証します。
CI での使用を推奨します。
```

注意: `.github/CODEOWNERS` 更新・Rune validate ガイドはロードマップの `**実装内容:**` スコープ外のため本バージョンでは対象としない。

---

### Step 2: COMMUNITY.md 新規作成

```markdown
# Favnir コミュニティ

## RFC プロセス

新機能の提案は RFC（Request for Comments）プロセスを経て承認されます。

1. `versions/roadmap/` に RFC 草稿を作成する
2. GitHub Issues でディスカッションを開始する
3. コアチームのレビュー後、ロードマップに組み込む

## ディスカッション場所

- **GitHub Issues**: バグ報告・機能要望
- **GitHub Discussions**: 設計議論・RFC

## 行動規範

すべての参加者は [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) に従ってください。
```

---

### Step 3: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v79.7.0 エントリを追加。

---

### Step 4: driver.rs — v797000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

```rust
// --- v79.7.0: OSS 公開強化・コミュニティ整備 ---
#[cfg(test)]
mod v797000_tests {
    const CONTRIBUTING: &str = include_str!("../../CONTRIBUTING.md");
    const COMMUNITY:    &str = include_str!("../../COMMUNITY.md");

    #[test]
    fn oss_contributing_v2_exists() {
        assert!(CONTRIBUTING.contains("Execution Effects"), "CONTRIBUTING.md must mention Execution Effects");
        assert!(CONTRIBUTING.contains("fav verify"), "CONTRIBUTING.md must mention fav verify");
        assert!(CONTRIBUTING.contains("invariant"), "CONTRIBUTING.md must mention invariant");
    }

    #[test]
    fn oss_community_md_exists() {
        assert!(COMMUNITY.contains("RFC"), "COMMUNITY.md must describe RFC process");
        assert!(COMMUNITY.contains("GitHub"), "COMMUNITY.md must mention GitHub");
    }
}
```

注意: `use super::*` 不要。`const CONTRIBUTING` / `const COMMUNITY` パターンを採用。

---

### Step 5: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "79.6.0"` → `"79.7.0"` に更新。

driver.rs 内の escaped `\"79.6.0\"` を `\"79.7.0\"` に一括更新（sed）。
エラーメッセージ文字列（unescaped）の `79.6.0` も `79.7.0` に更新。

更新後に `grep -c "79\.6\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` → 出力が `1` であることを確認。
（残るのは `// --- v79.6.0: ドッグフーディング強化 ---` コメント行の 1 件のみ）

---

### Step 6: versions/current.md 更新

- `## 進行中バージョン` → `**v79.7.0**（OSS 公開強化・コミュニティ整備）`
- `## 次に切る版` → `**v79.8.0**（ドキュメント完全化 v3 リファレンス）`

---

### Step 7: 最終確認

```bash
cargo test v797000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3801 tests pass、v797000 2 件 pass を確認。

---

## 依存順序サマリ

```
CONTRIBUTING.md 更新（Step 1）
  → COMMUNITY.md 作成（Step 2）
  → CHANGELOG 更新（Step 3）
  → driver.rs テスト追加（Step 4）← 両ファイルが先に存在すること
  → Cargo.toml + エラーメッセージ更新（Step 5）
  → current.md 更新（Step 6）
  → 最終確認（Step 7）
```
