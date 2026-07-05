# v31.0.0 実装計画 — Real-World Readiness マイルストーン宣言

## 前提

- `fav/Cargo.toml` version = `30.9.0`
- `cargo test` — 2418 passed（0 failures）
- v30.9.0 が COMPLETE であること

---

## 実装ステップ

### Step 1: バージョンバンプ

**`fav/Cargo.toml`**
- `version = "30.9.0"` → `version = "31.0.0"`

### Step 2: driver.rs スタブ化 + v310000_tests 追加

**`fav/src/driver.rs`**
1. `v309000_tests::cargo_toml_version_is_30_9_0` をスタブ化（本体を空にしコメントを追加）
2. `v310000_tests` モジュールを末尾に追加（`use super::*` なし）

```rust
mod v310000_tests {
    #[test]
    fn cargo_toml_version_is_31_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"31.0.0\""), "Cargo.toml must contain version = \"31.0.0\"");
    }
    #[test]
    fn milestone_real_world_readiness_declared() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Real-World Readiness"), "MILESTONE.md must contain 'Real-World Readiness'");
    }
    #[test]
    fn readme_mentions_v31_0() {
        let src = include_str!("../../README.md");
        assert!(src.contains("v31.0"), "README.md must contain 'v31.0'");
    }
    #[test]
    fn benchmark_v31_0_0_exists() {
        let src = include_str!("../../benchmarks/v31.0.0.json");
        assert!(src.contains("31.0.0"), "benchmarks/v31.0.0.json must contain '31.0.0'");
    }
}
```

### Step 3: MILESTONE.md 追記

`MILESTONE.md` 先頭（既存セクションの前）に追加:

```markdown
## v31.0.0 — Real-World Readiness（2026-07-02）

> 「`fav new --template postgres-etl my-project` で生成されたプロジェクトが、
>  `fav check` / `fav run` / `fav test` すべてで通り、
>  実データ（CSV 1000 行）を Postgres に書き込めること」
> = Real-World Readiness の完成を象徴するデモ

v31.0.0 をもって、Favnir の **Real-World Readiness** を正式に宣言する。

`fav new --template postgres-etl` による 4 ファイル構成テンプレート（types / validators / stages / main）が生成され、
`fav check` / `fav test` / `fav lint` の全コマンドが通過する。
`examples/csv-to-postgres/` に CSV 1000 行 → Postgres の実証パイプラインが実装され、
`fav test`（引数なし）がプロジェクト全体のテストを一括実行できるようになった。

### 達成コンポーネント（v30.1〜v30.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| ビルド軽量化 | v30.1 | `[profile.dev] debug = 0` で target/ 削減 |
| postgres-etl テンプレート v2 | v30.2 | 4 ファイル構成・`fav check` 全通過 |
| マルチファイル E2E | v30.3 | 5 コマンド（check/run/test/lint/fmt）全通過 |
| Rune import マルチファイル | v30.4 | 同一 Rune を複数ファイルから import 可能 |
| ドッグフードサンプル | v30.5 | `examples/csv-to-postgres/` 5 ステージ実装 |
| fav test プロジェクト統合 | v30.6 | 引数なし `fav test` でプロジェクト全体実行 |
| エラー表示改善 | v30.7 | ステージ名・ヒント付きランタイムエラー |
| fav new --list | v30.8 | 8 テンプレートの一覧表示 |
| ドッグフード修正 | v30.9 | `[project]` 解析・import 解決・UX hint |

**宣言日**: 2026-07-02
**宣言バージョン**: v31.0.0
```

### Step 4: README.md 更新

既存の最新マイルストーン行の直後（または先頭の目立つ場所）に追加:

```markdown
**v31.0（2026-07-02）で、[Real-World Readiness](./MILESTONE.md) マイルストーンを宣言しました。**
```

### Step 5: CHANGELOG.md 追記

先頭に追加:

```markdown
## [v31.0.0] — 2026-07-02

### Added
- Real-World Readiness マイルストーンを正式宣言
- `MILESTONE.md` に v31.0.0 セクション追加（v30.1〜v30.9 達成コンポーネント一覧）
- `benchmarks/v31.0.0.json` 追加

### Changed
- `Cargo.toml` version: `30.9.0` → `31.0.0`
```

### Step 6: benchmarks/v31.0.0.json 作成

```json
{
  "version": "31.0.0",
  "date": "2026-07-02",
  "milestone": "Real-World Readiness",
  "tests_passed": 2422,
  "tests_failed": 0,
  "notes": "cargo clean + cargo test after milestone declaration"
}
```

> `tests_passed` は `cargo test` 実行後に確認した実数で上書きする。

### Step 7: versions/current.md 更新

- 「最新安定版」欄を v31.0.0 に更新
- 「進行中バージョン」を「なし（v31.0.0 完了直後）」に更新
- 「次に切る版」を「v31.1.0 — TBD」に更新
- マイルストーン進捗表の `v31.0 — Real-World Readiness` を `**完了**` に変更

### Step 8: cargo clean + hello.fav 復元 + cargo build + cargo test

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
# hello.fav を復元
cat > tmp/hello.fav << 'EOF'
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() -> Bool {
    add(1, 2) == 3
}
EOF
cargo build 2>&1 | tail -5
cargo test 2>&1 | grep "test result"
du -sh target/
```

---

## ファイル変更一覧

| ファイル | 種別 | 変更内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `30.9.0` → `31.0.0` |
| `fav/src/driver.rs` | 更新 | v309000 スタブ化 + v310000_tests 追加 |
| `MILESTONE.md` | 更新 | v31.0.0 セクション追加 |
| `README.md` | 更新 | v31.0 マイルストーン一行追加 |
| `CHANGELOG.md` | 更新 | [v31.0.0] セクション追加 |
| `benchmarks/v31.0.0.json` | 新規 | ベンチマーク結果 |
| `versions/current.md` | 更新 | v31.0.0 に更新 |

---

## 完了判定

- `cargo test v310000` — 4/4 PASS
- `cargo test` — 全件 PASS（0 failures）
