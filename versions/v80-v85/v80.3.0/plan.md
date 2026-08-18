# Plan: v80.3.0 — `TestFixture` / `DataFactory` モックデータ生成

実装依存順（既存モジュール追記 → テスト追加）

> `lib.rs` 変更不要。`driver.rs` はバイナリクレートのため `fav_core::test_framework::*` を使用。
> `#[cfg(test)] mod v80300_tests` パターン（v80.1.0/v80.2.0 の慣例）。

---

## Step 1: `fav/src/test_framework.rs` に型と実装を追加

`format_golden_diff` / `load_golden_dataset` の後ろに以下を追記する。

```rust
// ─── TestFixture / DataFactory ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum FieldSpec {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

/// 1 行分のフィールド仕様: (列名, FieldSpec) のペア列。
pub type RowSpec = Vec<(String, FieldSpec)>;

#[derive(Debug)]
pub struct TestFixture {
    pub name: String,
    pub schema: Vec<String>,
    pub rows: Vec<RowSpec>,
}

#[derive(Debug)]
pub struct DataFactory {
    pub seed: u64,
}

impl DataFactory {
    pub fn from_seed(seed: u64) -> DataFactory {
        DataFactory { seed }
    }

    pub fn generate_rows(&self, spec: &TestFixture, count: usize) -> Vec<Vec<String>> {
        if spec.rows.is_empty() {
            return Vec::new();
        }
        let n = spec.rows.len();
        let stride = self.seed.max(1) as usize;
        (0..count)
            .map(|i| {
                let template = &spec.rows[(i * stride + i) % n];
                // テンプレートをマップに変換して schema 順に並べ替える
                let field_map: std::collections::HashMap<&str, &FieldSpec> = template
                    .iter()
                    .map(|(k, v)| (k.as_str(), v))
                    .collect();
                spec.schema
                    .iter()
                    .map(|col| match field_map.get(col.as_str()) {
                        Some(FieldSpec::Str(s))   => s.clone(),
                        Some(FieldSpec::Int(n))   => n.to_string(),
                        Some(FieldSpec::Float(f)) => f.to_string(),
                        Some(FieldSpec::Bool(b))  => b.to_string(),
                        Some(FieldSpec::Null) | None => String::new(),
                    })
                    .collect()
            })
            .collect()
    }
}
```

---

## Step 2: `fav/src/driver.rs` に `mod v80300_tests` を追加

`mod v80200_tests { ... }` の直後に以下を追加する。

```rust
#[cfg(test)]
mod v80300_tests {
    use fav_core::test_framework::*;

    fn make_fixture() -> TestFixture {
        TestFixture {
            name: "users".to_string(),
            schema: vec!["name".to_string(), "age".to_string()],
            rows: vec![
                vec![
                    ("name".to_string(), FieldSpec::Str("alice".to_string())),
                    ("age".to_string(),  FieldSpec::Int(30)),
                ],
                vec![
                    ("name".to_string(), FieldSpec::Str("bob".to_string())),
                    ("age".to_string(),  FieldSpec::Int(25)),
                ],
            ],
        }
    }

    #[test]
    fn data_factory_generates_rows() {
        let factory = DataFactory::from_seed(1);
        let fixture = make_fixture();
        let rows = factory.generate_rows(&fixture, 2);
        assert_eq!(rows.len(), 2);
        // 各行の列数が schema の列数（2）と一致する
        for row in &rows {
            assert_eq!(row.len(), fixture.schema.len());
        }
        // seed=1, stride=1: row[0] = rows[(0*1+0)%2] = rows[0] = alice/30
        assert_eq!(rows[0], vec!["alice", "30"]);
        // row[1] = rows[(1*1+1)%2] = rows[0] = alice/30
        assert_eq!(rows[1], vec!["alice", "30"]);
    }

    #[test]
    fn test_fixture_schema_matches_rows() {
        let factory = DataFactory::from_seed(0);
        let fixture = make_fixture();
        let rows = factory.generate_rows(&fixture, 3);
        assert_eq!(rows.len(), 3);
        for row in &rows {
            assert_eq!(row.len(), fixture.schema.len(),
                "each generated row must have exactly schema.len() columns");
        }
        // seed=0, stride=1: rows[0] = (0*1+0)%2 = 0 → alice/30
        assert_eq!(rows[0], vec!["alice", "30"]);
    }
}
```

---

## Step 3: `cargo test` で全 pass を確認

```bash
cargo test 2>&1 | tail -5
```

3816 tests, 0 failures であることを確認する。
