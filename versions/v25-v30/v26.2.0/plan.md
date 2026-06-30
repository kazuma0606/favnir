# v26.2.0 実装計画 — nats Rune 実質化

## 実装方針

- nats Rune は kafka / kinesis Rune と同じ**シングルファイルパターン**（`runes/nats/nats.fav` のみ）で実装する
- Cargo 依存追加は**しない**（スタブ実装でモック値を返す、実 NATS 接続は v26.7 E2E デモで別途）
- `NatsConn(String)` は `VMValue::Str` にマップ（KafkaConn / KinesisConn と同パターン）
- 環境変数: `NATS_URL`（デフォルト: `nats://localhost:4222`）

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep 'version = ' fav/Cargo.toml                       # "26.1.0" であること
cat benchmarks/v26.1.0.json                            # "test_count":2047 であること
cargo test --bin fav 2>&1 | tail -3                    # 2047 件 PASS であること
ls runes/nats/ 2>/dev/null || echo "not found"         # 未存在であること
```

### Step 1: `fav/Cargo.toml` bump（26.1.0 → 26.2.0）

```toml
version = "26.2.0"
```

### Step 2: VM Primitive 5 件追加（`fav/src/backend/vm.rs`）

> **順序の理由**: nats.fav が vm.rs の primitive を呼び出すため、vm.rs を先に追加する（kinesis と同順）。

挿入位置: Kinesis primitive ブロックの直後（`"Kinesis.get_records_raw"` wasm32 arm の後）。

**各 primitive の `#[cfg]` ガード方針**:
Kinesis と同様に、各 primitive は以下のペアで実装すること:

```rust
#[cfg(not(target_arch = "wasm32"))]
"NATS.connect_raw" => {
    // 実際の接続処理 / スタブ実装
}
#[cfg(target_arch = "wasm32")]
"NATS.connect_raw" => {
    Err("NATS not supported on wasm32".to_string())
}
```

> wasm32 フォールバックのエラーメッセージは各 primitive で `"NATS not supported on wasm32"` に統一すること（primitive 名による差異を設けない）。

追加する primitive:

| primitive 名 | 引数 | 戻り値 | スタブ実装 |
|---|---|---|---|
| `"NATS.connect_raw"` | `url: String` | `NatsConn` (Str) | URL 検証（`nats://` or env var）、`VMValue::Str(url)` を返す |
| `"NATS.publish_raw"` | `conn, subject, payload: String` | `Unit` | スタブ: `VMValue::Unit` を返す |
| `"NATS.subscribe_raw"` | `conn, subject: String` | `String`（JSON） | スタブ: `"{}"` を返す |
| `"NATS.jetstream_publish_raw"` | `conn, stream, payload: String` | `String`（seq） | スタブ: `"seq-js-0001"` を返す |
| `"NATS.jetstream_consume_raw"` | `conn, stream, consumer: String` | `String`（JSON 配列） | スタブ: `"[]"` を返す |

### Step 2.5: `cargo build` — vm.rs コンパイルエラーなし確認

```bash
cargo build --bin fav 2>&1 | grep -E "^error" | head -10
```

### Step 3: `runes/nats/nats.fav` 新規作成

kafka.fav / kinesis.fav パターン（シングルファイル）で作成（spec.md §4 参照）:

```favnir
type NatsConn(String)
type NatsMsg = { subject: String, payload: String, reply: String }

public fn connect(url: String) -> Result<NatsConn, String> !Stream { ... }
public fn publish(conn: NatsConn, subject: String, payload: String) -> Result<Unit, String> !Stream { ... }
public fn subscribe(conn: NatsConn, subject: String) -> Result<String, String> !Stream { ... }
public fn jetstream_publish(conn: NatsConn, stream: String, payload: String) -> Result<String, String> !Stream { ... }
public fn jetstream_consume(conn: NatsConn, stream: String, consumer: String) -> Result<String, String> !Stream { ... }
```

### Step 4: `site/content/docs/runes/nats.mdx` 新規作成

5 条件クリア状況・API ドキュメント・Docker 実行手順を含む MDX を作成:

- LocalStack/nats-server セットアップ: `docker run -p 4222:4222 nats:latest -js`
- 環境変数: `NATS_URL=nats://localhost:4222`
- API リファレンス: 5 関数
- JetStream の使い方例

### Step 5: `CHANGELOG.md` 更新

先頭に `[v26.2.0]` エントリを追加:

```markdown
## [v26.2.0] — 2026-06-26 — nats Rune 実質化

### Added
- `runes/nats/nats.fav` — NATS Rune（connect / publish / subscribe / jetstream_publish / jetstream_consume）
- `NATS.connect_raw` / `publish_raw` / `subscribe_raw` / `jetstream_publish_raw` / `jetstream_consume_raw` — VM primitive 5 件追加
- `site/content/docs/runes/nats.mdx` — NATS Rune ドキュメント新規作成
```

### Step 6: `benchmarks/v26.2.0.json` 新規作成

```json
{"version":"26.2.0","test_count":2053,"timestamp":"2026-06-26"}
```

### Step 7: `fav/src/driver.rs` に `v262000_tests` 追加

`v261000_tests` の直後に追加（6 件）:

```rust
// ── v262000_tests (v26.2.0) — nats Rune 実質化 ─────────────────────────
#[cfg(test)]
mod v262000_tests {
    #[test]
    fn nats_rune_has_connect_fn() {
        let src = include_str!("../../runes/nats/nats.fav");
        assert!(src.contains("fn connect"), "nats connect fn not found");
    }
    #[test]
    fn nats_rune_has_publish_fn() {
        let src = include_str!("../../runes/nats/nats.fav");
        assert!(src.contains("fn publish"), "nats publish fn not found");
    }
    #[test]
    fn nats_rune_has_subscribe_fn() {
        let src = include_str!("../../runes/nats/nats.fav");
        assert!(src.contains("fn subscribe"), "nats subscribe fn not found");
    }
    #[test]
    fn nats_rune_has_jetstream_publish_fn() {
        let src = include_str!("../../runes/nats/nats.fav");
        assert!(src.contains("fn jetstream_publish"), "nats jetstream_publish fn not found");
    }
    #[test]
    fn nats_rune_has_jetstream_consume_fn() {
        let src = include_str!("../../runes/nats/nats.fav");
        assert!(src.contains("fn jetstream_consume"), "nats jetstream_consume fn not found");
    }
    #[test]
    fn nats_rune_has_nats_msg_type() {
        let src = include_str!("../../runes/nats/nats.fav");
        assert!(src.contains("type NatsMsg"), "nats NatsMsg type not found");
    }
    #[test]
    fn changelog_has_v26_2_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v26.2.0]"), "CHANGELOG.md must contain '[v26.2.0]'");
    }
}
```

### Step 8: テスト確認

```bash
cd fav && cargo test v262000 --bin fav          # 7/7 PASS
cd fav && cargo test --bin fav -j 8 -- --test-threads=8 2>&1 | tail -4  # 2054 件 PASS
```

> `--bin fav` フラグが必須。テストモジュール（`v262000_tests`）は `driver.rs` 内に配置されるため
> `cargo test v262000` だけでは見つからない。

---

## ファイル変更一覧

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version bump 26.1.0 → 26.2.0 |
| `runes/nats/nats.fav` | **新規作成**（2 型 + 5 関数） |
| `fav/src/backend/vm.rs` | NATS primitive 5 件追加 |
| `site/content/docs/runes/nats.mdx` | **新規作成** |
| `CHANGELOG.md` | `[v26.2.0]` エントリ先頭に追加 |
| `benchmarks/v26.2.0.json` | **新規作成** |
| `fav/src/driver.rs` | `v262000_tests`（7 件）追加 |

---

## 注意事項

- `runes/nats/` ディレクトリは未存在。`nats.fav` 作成時に Write ツールが自動作成する。
- **vm.rs を先に実装してから nats.fav を作成する**（Step 2 → Step 3 の順序が重要）。
- `#[cfg(not(target_arch = "wasm32"))]` ガードと wasm32 フォールバックを各 primitive でペアで実装すること。
- `publish_raw` の戻り値は `VMValue::Unit`（kafka の `produce_raw` と同パターン）。
- `subscribe_raw` / `jetstream_consume_raw` の戻り値は JSON 文字列（`VMValue::Str`）。`List<NatsMsg>` への変換は呼び出し元が行う。
- `include_str!` のパスは `fav/src/driver.rs` から見た相対パス:
  - `runes/nats/nats.fav` → `"../../runes/nats/nats.fav"`
  - `CHANGELOG.md` → `"../../CHANGELOG.md"`
- vm.rs primitive 挿入位置: `"Kinesis.get_records_raw"` wasm32 arm の直後。

## リスクと対応

| リスク | 対応 |
|---|---|
| nats-server が起動していない環境でのテスト失敗 | primitive をスタブ実装（接続なしでモック値を返す） |
| `publish_raw` の戻り値型: `Result<Unit, String>` の `Unit` | kafka `produce_raw` と同パターンで `VMValue::Unit` を使用 |
| JetStream と Core NATS の概念分離 | シングルファイルで両方の関数を提供（prefix で区別: `publish` vs `jetstream_publish`） |
