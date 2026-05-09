use super::VM;
use crate::backend::codegen::codegen_program;
use crate::frontend::parser::Parser;
use crate::middle::compiler::compile_program;
use crate::value::Value;
use std::collections::HashMap;

fn eval(src: &str) -> Value {
    let prog = Parser::parse_str(src, "test").expect("parse error");
    let ir = compile_program(&prog);
    let artifact = codegen_program(&ir);
    let main_idx = artifact.fn_idx_by_name("main").expect("main not found");
    VM::run(&artifact, main_idx, vec![]).expect("runtime error")
}

fn eval_fn(src: &str, fname: &str, args: Vec<Value>) -> Value {
    let prog = Parser::parse_str(src, "test").expect("parse error");
    let ir = compile_program(&prog);
    let artifact = codegen_program(&ir);
    let fn_idx = artifact.fn_idx_by_name(fname).expect("fn not found");
    VM::run(&artifact, fn_idx, args).expect("runtime error")
}

fn eval_with_db(src: &str, db_path: &std::path::Path) -> Value {
    let prog = Parser::parse_str(src, "test").expect("parse error");
    let ir = compile_program(&prog);
    let artifact = codegen_program(&ir);
    let main_idx = artifact.fn_idx_by_name("main").expect("main not found");
    let db_path = db_path.to_string_lossy().into_owned();
    VM::run_with_db_path(&artifact, main_idx, vec![], Some(&db_path)).expect("runtime error").0
}

#[test]
fn legacy_vm_test_literals() {
    assert_eq!(eval_fn("fn f() -> Int { 42 }", "f", vec![]), Value::Int(42));
    assert_eq!(eval_fn("fn f() -> Bool { true }", "f", vec![]), Value::Bool(true));
    assert_eq!(eval_fn("fn f() -> String { \"hi\" }", "f", vec![]), Value::Str("hi".into()));
}

#[test]
fn legacy_vm_test_function_apply_and_closure() {
    assert_eq!(
        eval_fn("fn add(a: Int, b: Int) -> Int { a + b }", "add", vec![Value::Int(3), Value::Int(4)]),
        Value::Int(7)
    );
    assert_eq!(
        eval_fn("fn f() -> Int { bind g <- |x| x + 1; g(10) }", "f", vec![]),
        Value::Int(11)
    );
}

#[test]
fn legacy_vm_test_pipeline_and_bind() {
    let src = "
        stage Double: Int -> Int = |n| { n + n }
        stage Inc: Int -> Int = |n| { n + 1 }
        fn f(x: Int) -> Int { bind y <- x |> Double; y |> Inc }
    ";
    assert_eq!(eval_fn(src, "f", vec![Value::Int(3)]), Value::Int(7));
}

#[test]
#[ignore = "record destructuring still needs dedicated VM parity"]
fn legacy_vm_test_bind_record_destruct() {
    let src = "
        type Point = { x: Int y: Int }
        fn sum(p: Point) -> Int { bind { x, y } <- p; x + y }
    ";
    let point = Value::Record({
        let mut m = HashMap::new();
        m.insert("x".into(), Value::Int(3));
        m.insert("y".into(), Value::Int(4));
        m
    });
    assert_eq!(eval_fn(src, "sum", vec![point]), Value::Int(7));
}

#[test]
#[ignore = "variant destructuring still needs dedicated VM parity"]
fn legacy_vm_test_bind_variant_destruct() {
    let src = "
        type Wrap = | Val(Int)
        fn unwrap(w: Wrap) -> Int { bind Val(v) <- w; v }
    ";
    let wrapped = Value::Variant("Val".into(), Some(Box::new(Value::Int(99))));
    assert_eq!(eval_fn(src, "unwrap", vec![wrapped]), Value::Int(99));
}

#[test]
fn legacy_vm_test_match_and_if() {
    let src = "
        type Color = | Red | Green | Blue
        fn pick(c: Color, b: Bool) -> Int {
            if b {
                match c {
                    Red => 0
                    Green => 1
                    Blue => 2
                }
            } else {
                9
            }
        }
    ";
    assert_eq!(eval_fn(src, "pick", vec![Value::Variant("Green".into(), None), Value::Bool(true)]), Value::Int(1));
    assert_eq!(eval_fn(src, "pick", vec![Value::Variant("Green".into(), None), Value::Bool(false)]), Value::Int(9));
}

#[test]
fn legacy_vm_test_record_construct_and_field_access() {
    let src = r#"
        type User = { name: String age: Int }
        public fn main() -> Int {
            bind user <- User { name: "Alice", age: 30 }
            user.age
        }
    "#;
    assert_eq!(eval(src), Value::Int(30));
}

#[test]
fn legacy_vm_test_option_and_result() {
    assert_eq!(
        eval("public fn main() -> Int { Option.unwrap_or(Option.some(42), 0) }"),
        Value::Int(42)
    );
    assert_eq!(
        eval("public fn main() -> Int { Result.unwrap_or(Result.err(\"fail\"), 9) }"),
        Value::Int(9)
    );
}

#[test]
fn legacy_vm_test_db_execute_query() {
    let src = r#"
        public fn main() -> Unit !Db {
            bind _ <- Db.execute("CREATE TABLE t (id INTEGER, name TEXT)");
            bind _ <- Db.execute("INSERT INTO t VALUES (?, ?)", 1, "Alice");
            bind _ <- Db.execute("INSERT INTO t VALUES (?, ?)", 2, "Bob");
            bind rows <- Db.query("SELECT id, name FROM t ORDER BY id");
            IO.println(Debug.show(rows))
        }
    "#;
    let db_path = std::env::temp_dir().join(format!("favnir-legacy-vm-db-{}.sqlite3", uuid::Uuid::new_v4()));
    let result = eval_with_db(src, &db_path);
    assert_eq!(result, Value::Unit);
}

#[test]
fn legacy_vm_test_file_read_write_roundtrip() {
    use tempfile::NamedTempFile;
    let tmp = NamedTempFile::new().expect("tempfile");
    let path = tmp.path().to_str().expect("path").replace('\\', "/");
    let content = "hello from Favnir";
    let src = format!(
        r#"
public fn main() -> String !File {{
    File.write("{path}", "{content}");
    File.read("{path}")
}}
"#
    );
    assert_eq!(eval(&src), Value::Str(content.into()));
}
