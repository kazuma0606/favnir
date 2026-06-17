# v17.8.0 — パッケージシステム成熟（rune registry v2）仕様

## 概要

Rune を「パッケージ」として `fav.toml` で依存管理できるようにする。
`fav add` / `fav update` / `fav remove` / `fav publish` で Cargo ライクなエコシステムを整備する。

---

## 1. fav.toml 依存管理

### 1.1 新規セクション

```toml
[dependencies]
csv         = "^2.0.0"
bigquery    = "^1.0.0"
my-company/etl-utils = "^0.5.0"    # スコープ付き（社内パッケージ）

[dev-dependencies]
test-fixtures = "^1.0.0"

[registry]
url = "https://registry.favnir.dev"   # デフォルト（省略可）
```

### 1.2 バージョン文字列の形式

| 形式 | 意味 | 例 |
|---|---|---|
| `"^2.0.0"` | メジャーバージョン固定（2.x.x） | `^2.0.0` → `>=2.0.0, <3.0.0` |
| `"~2.1.0"` | マイナーバージョン固定（2.1.x） | `~2.1.0` → `>=2.1.0, <2.2.0` |
| `"=2.1.3"` | 完全固定 | `=2.1.3` のみ |
| `"*"` | 任意の最新版 | 最新版 |

### 1.3 既存 `FavToml` への追加フィールド

```rust
pub struct FavToml {
    // 既存フィールド...
    pub dev_dependencies: Vec<DependencySpec>,   // 新規
    pub registry_url: Option<String>,            // 新規（[registry] url）
}
```

`DependencySpec` は既存の enum をそのまま利用：
- `Semver { name: String, version: String }` — `^2.0.0` 形式
- `Registry { name: String, registry: String, version: String }` — 明示的レジストリ指定
- `Path { name: String, path: String }` — ローカルパス

---

## 2. fav.lock 形式

### 2.1 フォーマット（TOML）

```toml
# このファイルは fav が自動生成します。手動編集しないでください。

[[package]]
name = "csv"
version = "2.1.0"
checksum = "sha256:abc123..."
source = "registry:https://registry.favnir.dev"

[[package]]
name = "bigquery"
version = "1.0.3"
checksum = "sha256:def456..."
source = "registry:https://registry.favnir.dev"
```

### 2.2 既存 `LockedPackage` への追加フィールド

```rust
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub resolved_path: String,   // 既存
    pub checksum: Option<String>, // 新規
    pub source: Option<String>,   // 新規
}
```

---

## 3. CLI コマンド

### 3.1 `fav add`

```bash
fav add csv                  # 最新版を追加（fav.toml と fav.lock を更新）
fav add csv@2.1.0            # バージョン指定（^2.1.0 として保存）
fav add --dev test-fixtures  # [dev-dependencies] に追加
```

**動作:**
1. registry API に `GET /packages/{name}` を送り利用可能バージョン一覧を取得
2. バージョン指定がない場合は最新版を選択
3. `fav.toml` の `[dependencies]`（または `[dev-dependencies]`）に追記
4. `fav.lock` を更新（checksum / source を書き込む）

### 3.2 `fav update`

```bash
fav update                   # 全パッケージを semver 範囲内で更新
fav update csv               # 特定パッケージのみ更新
```

**動作:**
1. `fav.toml` の `[dependencies]` を読む
2. 各パッケージについて semver 制約を満たす最新版を registry から取得
3. `fav.lock` を更新

### 3.3 `fav remove`

```bash
fav remove csv               # 依存から削除
```

**動作:**
1. `fav.toml` から該当パッケージのエントリを削除
2. `fav.lock` から該当パッケージエントリを削除

### 3.4 `fav publish`

```bash
fav publish                  # registry に公開
fav publish --dry-run        # 公開内容確認（実際には公開しない）
```

**動作:**
1. `fav.toml` の `[package]` セクションを検証（name / version 必須）
2. `--dry-run` なら内容を表示して終了
3. registry API に `POST /packages` を送信（認証トークン必要）
4. 成功メッセージを表示

### 3.5 `fav login`（新規）

```bash
fav login                    # ブラウザを開いて認証、トークンを ~/.fav/credentials に保存
```

---

## 4. Registry API クライアント

### 4.1 エンドポイント

| メソッド | パス | 説明 |
|---|---|---|
| `GET` | `/packages/{name}` | パッケージのバージョン一覧取得 |
| `GET` | `/packages/{name}/{version}` | 特定バージョンの詳細取得 |
| `POST` | `/packages` | パッケージ公開（認証必要） |

### 4.2 レスポンス形式（GET /packages/{name}）

```json
{
  "name": "csv",
  "versions": ["1.0.0", "1.1.0", "2.0.0", "2.1.0"],
  "latest": "2.1.0"
}
```

### 4.3 実装場所

`fav/src/registry/client.rs`（新規）に実装。
`ureq` クレート（既存依存）を使用。

---

## 5. Semver 解決ロジック

### 5.1 `fav/src/registry/resolver.rs`（新規）

```rust
pub struct SemVer { pub major: u32, pub minor: u32, pub patch: u32 }

pub enum VersionReq {
    Caret(SemVer),   // ^
    Tilde(SemVer),   // ~
    Exact(SemVer),   // =
    Any,             // *
}

pub fn parse_version_req(s: &str) -> Option<VersionReq>
pub fn matches_req(req: &VersionReq, v: &SemVer) -> bool
pub fn resolve_best(req: &VersionReq, available: &[SemVer]) -> Option<SemVer>
```

### 5.2 `^` の解決規則

- `^2.0.0` → `>=2.0.0, <3.0.0`
- `^0.2.0` → `>=0.2.0, <0.3.0`（メジャーが 0 の場合はマイナー固定）
- `^0.0.3` → `=0.0.3`（0.0.x の場合は完全固定）

---

## 6. 認証

- トークンは `~/.fav/credentials` に保存（TOML 形式）
- `fav publish` 時に読み込んで `Authorization: Bearer <token>` ヘッダに付与
- `fav login` で取得・保存（v17.8.0 ではスタブ実装でも可）

---

## 7. エラーコード

| コード | 説明 |
|---|---|
| `E0328` | パッケージが registry に存在しない |
| `E0329` | semver 制約を満たすバージョンが存在しない |
| `E0330` | `fav publish` に認証トークンがない |

---

## 8. テスト（v178000_tests）

| テスト名 | 内容 |
|---|---|
| `version_is_17_8_0` | Cargo.toml に "17.8.0" が含まれる |
| `fav_toml_dependencies_parse` | `[dependencies]` セクションが `FavToml` として解析される |
| `fav_lock_generates` | `fav.lock` が `LockFile::save` で生成される |
| `semver_caret_resolve` | `^2.0.0` が `2.x.x` の最新版を解決する |
| `cmd_add_updates_toml` | `cmd_add` が `fav.toml` の `[dependencies]` を更新する |

---

## 9. 完了条件

- [ ] `fav.toml` の `[dependencies]` / `[dev-dependencies]` / `[registry]` が解析される
- [ ] `fav.lock` が `[[package]]` + `checksum` + `source` フィールドで生成される
- [ ] `^` / `~` / `=` / `*` の semver 解決が正しく動作する
- [ ] `fav add csv` で `fav.toml` と `fav.lock` が更新される
- [ ] `fav publish --dry-run` が公開内容を表示して終了する
- [ ] `cargo test v178000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
