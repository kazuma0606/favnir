use arrow::array::{
    Array, ArrayRef, BooleanArray, BooleanBuilder, Float64Array, Float64Builder, Int64Array,
    Int64Builder, StringArray, StringBuilder,
};
use arrow::datatypes::{DataType, Field as ArrowField, Schema as ArrowSchema};
use arrow::record_batch::RecordBatch;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use bytes::Bytes;
use chrono::Utc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::arrow_writer::ArrowWriter;
use rusqlite::Connection;
use serde_json::Value as SerdeJsonValue;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::artifact::FvcArtifact;
use super::codegen::{Constant, Opcode};
use crate::middle::ir::TypeMeta;
use crate::value::Value;

thread_local! {
    /// When set to `true`, all `IO.println` / `IO.print` output is silently
    /// discarded.  Used by `cmd_test` when `--no-capture` is NOT given so that
    /// test bodies don't pollute the test-runner output.
    static SUPPRESS_IO_OUTPUT: Cell<bool> = const { Cell::new(false) };

    /// Coverage tracking: `Some(set)` when coverage is enabled, `None` otherwise.
    static COVERED_LINES: RefCell<Option<HashSet<u32>>> = RefCell::new(None);

    /// IO capture buffer: when `Some`, IO output is appended here instead of
    /// being printed to stdout.  Used by integration tests to inspect output.
    static IO_CAPTURE: RefCell<Option<String>> = RefCell::new(None);

    /// DB connection store: maps handle ID → connection wrapper.
    /// Transactions are tracked as (conn_id, in_tx flag).
    static DB_CONNECTIONS: RefCell<HashMap<u64, DbConnWrapper>> = RefCell::new(HashMap::new());
    static DB_NEXT_ID: Cell<u64> = const { Cell::new(1) };

    /// Seeded RNG for deterministic generation (v3.5.0).
    /// When `Some`, Random.int / Random.float / Gen.* use this instead of thread_rng.
    static SEEDED_RNG: RefCell<Option<rand::rngs::SmallRng>> = const { RefCell::new(None) };

    static CHECKPOINT_BACKEND: RefCell<CheckpointBackend> = RefCell::new(CheckpointBackend::File {
        dir: PathBuf::from(".fav_checkpoints"),
    });
}

/// Internal DB connection wrapper.
struct DbConnWrapper {
    conn: rusqlite::Connection,
    in_tx: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointBackend {
    File { dir: PathBuf },
    Sqlite { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointMetaRecord {
    pub name: String,
    pub value: String,
    pub updated_at: String,
}

/// Enable coverage tracking for the current thread.
pub fn enable_coverage() {
    COVERED_LINES.with(|c| *c.borrow_mut() = Some(HashSet::new()));
}

/// Disable coverage tracking and return the set of covered line numbers.
pub fn take_coverage() -> HashSet<u32> {
    COVERED_LINES.with(|c| c.borrow_mut().take().unwrap_or_default())
}

/// Start capturing IO output to an in-memory buffer.
/// All subsequent `IO.println` / `IO.print` calls append to the buffer.
#[allow(dead_code)]
pub fn start_io_capture() {
    IO_CAPTURE.with(|c| *c.borrow_mut() = Some(String::new()));
}

/// Stop capturing and return the accumulated output.
#[allow(dead_code)]
pub fn take_io_captured() -> String {
    IO_CAPTURE.with(|c| c.borrow_mut().take().unwrap_or_default())
}

/// Set whether IO output should be suppressed for the current thread.
/// Call `set_suppress_io(true)` before running tests, `set_suppress_io(false)`
/// after (or in a drop guard).
pub fn set_suppress_io(suppress: bool) {
    SUPPRESS_IO_OUTPUT.with(|c| c.set(suppress));
}

pub fn set_checkpoint_backend(backend: CheckpointBackend) {
    CHECKPOINT_BACKEND.with(|cell| {
        *cell.borrow_mut() = backend;
    });
}

pub fn checkpoint_meta(name: &str) -> Result<CheckpointMetaRecord, String> {
    checkpoint_meta_impl(name)
}

pub fn checkpoint_list() -> Result<Vec<CheckpointMetaRecord>, String> {
    checkpoint_list_impl()
}

pub fn checkpoint_save_direct(name: &str, value: &str) -> Result<(), String> {
    checkpoint_save_impl(name, value)
}

pub fn checkpoint_reset_direct(name: &str) -> Result<(), String> {
    checkpoint_reset_impl(name)
}

fn current_timestamp_string() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn with_checkpoint_backend<T>(
    f: impl FnOnce(&CheckpointBackend) -> Result<T, String>,
) -> Result<T, String> {
    CHECKPOINT_BACKEND.with(|cell| {
        let backend = cell.borrow().clone();
        f(&backend)
    })
}

fn checkpoint_value_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.txt"))
}

fn checkpoint_meta_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.meta.txt"))
}

fn checkpoint_meta_default(name: &str) -> CheckpointMetaRecord {
    CheckpointMetaRecord {
        name: name.to_string(),
        value: String::new(),
        updated_at: String::new(),
    }
}

fn ensure_checkpoint_dir(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| {
        format!(
            "checkpoint backend failed to create `{}`: {}",
            dir.display(),
            e
        )
    })
}

fn write_checkpoint_meta_file(dir: &Path, meta: &CheckpointMetaRecord) -> Result<(), String> {
    ensure_checkpoint_dir(dir)?;
    let path = checkpoint_meta_path(dir, &meta.name);
    let body = serde_json::json!({
        "name": meta.name,
        "value": meta.value,
        "updated_at": meta.updated_at,
    })
    .to_string();
    std::fs::write(&path, body)
        .map_err(|e| format!("checkpoint write failed for `{}`: {}", path.display(), e))
}

fn read_checkpoint_meta_file(dir: &Path, name: &str) -> Result<CheckpointMetaRecord, String> {
    let path = checkpoint_meta_path(dir, name);
    if !path.exists() {
        return Ok(checkpoint_meta_default(name));
    }
    let body = std::fs::read_to_string(&path)
        .map_err(|e| format!("checkpoint read failed for `{}`: {}", path.display(), e))?;
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
        format!(
            "checkpoint meta parse failed for `{}`: {}",
            path.display(),
            e
        )
    })?;
    Ok(CheckpointMetaRecord {
        name: json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(name)
            .to_string(),
        value: json
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        updated_at: json
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

fn ensure_checkpoint_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _fav_checkpoints (
            name TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .map(|_| ())
    .map_err(|e| format!("checkpoint sqlite setup failed: {}", e))
}

fn open_checkpoint_sqlite(path: &Path) -> Result<rusqlite::Connection, String> {
    let conn = rusqlite::Connection::open(path).map_err(|e| {
        format!(
            "checkpoint sqlite open failed for `{}`: {}",
            path.display(),
            e
        )
    })?;
    ensure_checkpoint_table(&conn)?;
    Ok(conn)
}

fn checkpoint_last_impl(name: &str) -> Result<Option<String>, String> {
    with_checkpoint_backend(|backend| match backend {
        CheckpointBackend::File { dir } => {
            ensure_checkpoint_dir(dir)?;
            let path = checkpoint_value_path(dir, name);
            if !path.exists() {
                return Ok(None);
            }
            let value = std::fs::read_to_string(&path)
                .map_err(|e| format!("checkpoint read failed for `{}`: {}", path.display(), e))?;
            Ok(Some(value))
        }
        CheckpointBackend::Sqlite { path } => {
            let conn = open_checkpoint_sqlite(path)?;
            let mut stmt = conn
                .prepare("SELECT value FROM _fav_checkpoints WHERE name = ?1")
                .map_err(|e| format!("checkpoint sqlite query prepare failed: {}", e))?;
            let mut rows = stmt
                .query([name])
                .map_err(|e| format!("checkpoint sqlite query failed: {}", e))?;
            match rows
                .next()
                .map_err(|e| format!("checkpoint sqlite row fetch failed: {}", e))?
            {
                Some(row) => {
                    let value: String = row
                        .get(0)
                        .map_err(|e| format!("checkpoint sqlite value decode failed: {}", e))?;
                    Ok(Some(value))
                }
                None => Ok(None),
            }
        }
    })
}

fn checkpoint_save_impl(name: &str, value: &str) -> Result<(), String> {
    let now = current_timestamp_string();
    with_checkpoint_backend(|backend| match backend {
        CheckpointBackend::File { dir } => {
            ensure_checkpoint_dir(dir)?;
            let value_path = checkpoint_value_path(dir, name);
            std::fs::write(&value_path, value).map_err(|e| {
                format!(
                    "checkpoint write failed for `{}`: {}",
                    value_path.display(),
                    e
                )
            })?;
            write_checkpoint_meta_file(
                dir,
                &CheckpointMetaRecord {
                    name: name.to_string(),
                    value: value.to_string(),
                    updated_at: now,
                },
            )
        }
        CheckpointBackend::Sqlite { path } => {
            let conn = open_checkpoint_sqlite(path)?;
            conn.execute(
                "INSERT INTO _fav_checkpoints(name, value, updated_at)
                 VALUES(?1, ?2, ?3)
                 ON CONFLICT(name) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                rusqlite::params![name, value, now],
            )
            .map(|_| ())
            .map_err(|e| format!("checkpoint sqlite save failed: {}", e))
        }
    })
}

fn checkpoint_reset_impl(name: &str) -> Result<(), String> {
    with_checkpoint_backend(|backend| match backend {
        CheckpointBackend::File { dir } => {
            ensure_checkpoint_dir(dir)?;
            let value_path = checkpoint_value_path(dir, name);
            if value_path.exists() {
                std::fs::remove_file(&value_path).map_err(|e| {
                    format!(
                        "checkpoint reset failed for `{}`: {}",
                        value_path.display(),
                        e
                    )
                })?;
            }
            let meta_path = checkpoint_meta_path(dir, name);
            if meta_path.exists() {
                std::fs::remove_file(&meta_path).map_err(|e| {
                    format!(
                        "checkpoint reset failed for `{}`: {}",
                        meta_path.display(),
                        e
                    )
                })?;
            }
            Ok(())
        }
        CheckpointBackend::Sqlite { path } => {
            let conn = open_checkpoint_sqlite(path)?;
            conn.execute("DELETE FROM _fav_checkpoints WHERE name = ?1", [name])
                .map(|_| ())
                .map_err(|e| format!("checkpoint sqlite reset failed: {}", e))
        }
    })
}

fn checkpoint_meta_impl(name: &str) -> Result<CheckpointMetaRecord, String> {
    with_checkpoint_backend(|backend| match backend {
        CheckpointBackend::File { dir } => read_checkpoint_meta_file(dir, name),
        CheckpointBackend::Sqlite { path } => {
            let conn = open_checkpoint_sqlite(path)?;
            let mut stmt = conn
                .prepare("SELECT value, updated_at FROM _fav_checkpoints WHERE name = ?1")
                .map_err(|e| format!("checkpoint sqlite query prepare failed: {}", e))?;
            let mut rows = stmt
                .query([name])
                .map_err(|e| format!("checkpoint sqlite query failed: {}", e))?;
            match rows
                .next()
                .map_err(|e| format!("checkpoint sqlite row fetch failed: {}", e))?
            {
                Some(row) => {
                    let value: String = row
                        .get(0)
                        .map_err(|e| format!("checkpoint sqlite value decode failed: {}", e))?;
                    let updated_at: String = row.get(1).map_err(|e| {
                        format!("checkpoint sqlite updated_at decode failed: {}", e)
                    })?;
                    Ok(CheckpointMetaRecord {
                        name: name.to_string(),
                        value,
                        updated_at,
                    })
                }
                None => Ok(checkpoint_meta_default(name)),
            }
        }
    })
}

fn checkpoint_list_impl() -> Result<Vec<CheckpointMetaRecord>, String> {
    with_checkpoint_backend(|backend| match backend {
        CheckpointBackend::File { dir } => {
            ensure_checkpoint_dir(dir)?;
            let mut metas = Vec::new();
            let rd = std::fs::read_dir(dir)
                .map_err(|e| format!("checkpoint list failed for `{}`: {}", dir.display(), e))?;
            for entry in rd {
                let entry =
                    entry.map_err(|e| format!("checkpoint list entry read failed: {}", e))?;
                let path = entry.path();
                let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                if let Some(name) = file_name.strip_suffix(".meta.txt") {
                    metas.push(read_checkpoint_meta_file(dir, name)?);
                }
            }
            metas.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(metas)
        }
        CheckpointBackend::Sqlite { path } => {
            let conn = open_checkpoint_sqlite(path)?;
            let mut stmt = conn
                .prepare("SELECT name, value, updated_at FROM _fav_checkpoints ORDER BY name")
                .map_err(|e| format!("checkpoint sqlite list prepare failed: {}", e))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(CheckpointMetaRecord {
                        name: row.get(0)?,
                        value: row.get(1)?,
                        updated_at: row.get(2)?,
                    })
                })
                .map_err(|e| format!("checkpoint sqlite list failed: {}", e))?;
            let mut metas = Vec::new();
            for row in rows {
                metas.push(
                    row.map_err(|e| format!("checkpoint sqlite list row decode failed: {}", e))?,
                );
            }
            Ok(metas)
        }
    })
}

pub struct SuppressIoGuard {
    prev: bool,
}

impl SuppressIoGuard {
    pub fn new(suppress: bool) -> Self {
        let prev = is_io_suppressed();
        set_suppress_io(suppress);
        Self { prev }
    }
}

impl Drop for SuppressIoGuard {
    fn drop(&mut self) {
        set_suppress_io(self.prev);
    }
}

#[inline]
fn is_io_suppressed() -> bool {
    SUPPRESS_IO_OUTPUT.with(|c| c.get())
}

#[cfg(test)]
pub fn io_output_suppressed_for_tests() -> bool {
    is_io_suppressed()
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceFrame {
    pub fn_name: String,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VMError {
    pub message: String,
    pub fn_name: String,
    pub ip: usize,
    pub stack_trace: Vec<TraceFrame>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallFrame {
    pub fn_idx: usize,
    pub ip: usize,
    pub base: usize,
    pub n_locals: usize,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub struct VM {
    globals: Vec<VMValue>,
    stack: Vec<VMValue>,
    frames: Vec<CallFrame>,
    collect_frames: Vec<Vec<VMValue>>,
    emit_log: Vec<VMValue>,
    db_path: Option<String>,
    source_file: String,
    type_metas: HashMap<String, TypeMeta>,
}

static SHARED_DBS: Mutex<Vec<(String, Connection)>> = Mutex::new(Vec::new());

/// Lazy stream representation for `Stream<T>` (v2.9.0)
#[derive(Debug, Clone)]
enum VMStream {
    /// Infinite: generates next value from current seed using next_fn
    Gen { seed: VMValue, next_fn: VMValue },
    /// Finite: converted from a list
    Of(Vec<VMValue>),
    /// Lazy map: apply map_fn to each element on collect
    Map {
        inner: Box<VMStream>,
        map_fn: VMValue,
    },
    /// Lazy filter: apply pred_fn to each element on collect
    Filter {
        inner: Box<VMStream>,
        pred_fn: VMValue,
    },
    /// Finite prefix of an inner stream
    Take { inner: Box<VMStream>, n: i64 },
}

#[derive(Debug, Clone)]
enum VMValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Unit,
    List(Vec<VMValue>),
    Record(HashMap<String, VMValue>),
    Variant(String, Option<Box<VMValue>>),
    VariantCtor(String),
    CompiledFn(usize),
    Closure(usize, Vec<VMValue>),
    Builtin(String),
    /// `Stream<T>` lazy sequence (v2.9.0)
    Stream(Box<VMStream>),
    /// Opaque DB connection handle (v3.3.0)
    DbHandle(u64),
    /// Opaque DB transaction handle (v3.3.0)
    TxHandle(u64),
}

impl PartialEq for VMValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VMValue::Bool(a), VMValue::Bool(b)) => a == b,
            (VMValue::Int(a), VMValue::Int(b)) => a == b,
            (VMValue::Float(a), VMValue::Float(b)) => a == b,
            (VMValue::Str(a), VMValue::Str(b)) => a == b,
            (VMValue::Unit, VMValue::Unit) => true,
            (VMValue::List(a), VMValue::List(b)) => a == b,
            (VMValue::Record(a), VMValue::Record(b)) => a == b,
            (VMValue::Variant(n1, p1), VMValue::Variant(n2, p2)) => n1 == n2 && p1 == p2,
            (VMValue::VariantCtor(a), VMValue::VariantCtor(b)) => a == b,
            (VMValue::CompiledFn(a), VMValue::CompiledFn(b)) => a == b,
            (VMValue::Closure(a, ca), VMValue::Closure(b, cb)) => a == b && ca == cb,
            (VMValue::Builtin(a), VMValue::Builtin(b)) => a == b,
            (VMValue::Stream(_), VMValue::Stream(_)) => false, // streams are not comparable
            (VMValue::DbHandle(a), VMValue::DbHandle(b)) => a == b,
            (VMValue::TxHandle(a), VMValue::TxHandle(b)) => a == b,
            _ => false,
        }
    }
}

impl VM {
    #[allow(dead_code)]
    pub fn new(artifact: &FvcArtifact) -> VM {
        Self::new_with_db_path(artifact, None)
    }

    pub fn new_with_db_path(artifact: &FvcArtifact, db_path: Option<String>) -> VM {
        let globals = artifact
            .globals
            .iter()
            .map(|g| match g.kind {
                0 => VMValue::CompiledFn(g.fn_idx as usize),
                1 => {
                    let name = artifact
                        .str_table
                        .get(g.name_idx as usize)
                        .cloned()
                        .unwrap_or_else(|| "<builtin>".to_string());
                    VMValue::Builtin(name)
                }
                2 => {
                    let name = artifact
                        .str_table
                        .get(g.name_idx as usize)
                        .cloned()
                        .unwrap_or_else(|| "<variant>".to_string());
                    VMValue::VariantCtor(name)
                }
                _ => VMValue::Unit,
            })
            .collect();
        VM {
            globals,
            stack: Vec::new(),
            frames: Vec::new(),
            collect_frames: Vec::new(),
            emit_log: Vec::new(),
            db_path,
            source_file: String::new(),
            type_metas: artifact.type_metas.clone(),
        }
    }

    pub fn set_source_file(&mut self, source_file: &str) {
        self.source_file = source_file.to_string();
    }

    #[allow(dead_code)]
    pub fn run(artifact: &FvcArtifact, fn_idx: usize, args: Vec<Value>) -> Result<Value, VMError> {
        Self::run_with_db_path(artifact, fn_idx, args, None).map(|(value, _)| value)
    }

    pub fn run_with_db_path(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
        db_path: Option<&str>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        Self::run_with_emits_and_db_path(artifact, fn_idx, args, db_path)
    }

    #[allow(dead_code)]
    pub fn run_with_emits(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        Self::run_with_emits_and_db_path(artifact, fn_idx, args, None)
    }

    pub fn run_with_emits_and_db_path(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
        db_path: Option<&str>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        Self::run_with_emits_db_path_and_source_file(artifact, fn_idx, args, db_path, None)
    }

    pub fn run_with_emits_db_path_and_source_file(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
        db_path: Option<&str>,
        source_file: Option<&str>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        let (value, emits) = Self::run_with_vmvalues(
            artifact,
            fn_idx,
            args.into_iter().map(VMValue::from).collect(),
            db_path.map(|s| s.to_string()),
            source_file.map(|s| s.to_string()),
        )?;
        Ok((
            Value::from(value),
            emits.into_iter().map(Value::from).collect(),
        ))
    }

    fn run_with_vmvalues(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<VMValue>,
        db_path: Option<String>,
        source_file: Option<String>,
    ) -> Result<(VMValue, Vec<VMValue>), VMError> {
        let mut vm = VM::new_with_db_path(artifact, db_path);
        if let Some(source_file) = source_file {
            vm.set_source_file(&source_file);
        }
        let ret = vm.invoke_function(artifact, fn_idx, args)?;
        Ok((ret, vm.emit_log))
    }

    fn invoke_function(
        &mut self,
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        let function = artifact.functions.get(fn_idx).ok_or_else(|| VMError {
            message: format!("unknown function index: {fn_idx}"),
            fn_name: "<invalid>".to_string(),
            ip: 0,
            stack_trace: vec![],
        })?;

        let caller_depth = self.frames.len();
        let base = self.stack.len();
        self.stack.extend(args);
        let required = function.local_count as usize;
        while self.stack.len() < base + required {
            self.stack.push(VMValue::Unit);
        }
        self.frames.push(CallFrame {
            fn_idx,
            ip: 0,
            base,
            n_locals: required,
            line: 0,
        });

        self.resume(artifact, caller_depth)
    }

    fn resume(&mut self, artifact: &FvcArtifact, caller_depth: usize) -> Result<VMValue, VMError> {
        let vm = self;
        loop {
            let Some(frame) = vm.frames.last_mut() else {
                return Ok(VMValue::Unit);
            };
            let function = &artifact.functions[frame.fn_idx];
            if frame.ip >= function.code.len() {
                return Err(vm.error(artifact, "instruction pointer out of bounds"));
            }
            let opcode = function.code[frame.ip];
            frame.ip += 1;

            match opcode {
                x if x == Opcode::Const as u8 => {
                    let idx = Self::read_u16(function, frame)? as usize;
                    let constant = function
                        .constants
                        .get(idx)
                        .ok_or_else(|| vm.error(artifact, "constant index out of bounds"))?;
                    vm.stack.push(constant_to_value(constant.clone()));
                }
                x if x == Opcode::ConstUnit as u8 => vm.stack.push(VMValue::Unit),
                x if x == Opcode::ConstTrue as u8 => vm.stack.push(VMValue::Bool(true)),
                x if x == Opcode::ConstFalse as u8 => vm.stack.push(VMValue::Bool(false)),
                x if x == Opcode::LoadLocal as u8 => {
                    let slot = Self::read_u16(function, frame)? as usize;
                    let idx = frame.base + slot;
                    let value = vm
                        .stack
                        .get(idx)
                        .cloned()
                        .ok_or_else(|| vm.error(artifact, "local slot out of bounds"))?;
                    vm.stack.push(value);
                }
                x if x == Opcode::StoreLocal as u8 => {
                    let slot = Self::read_u16(function, frame)? as usize;
                    let idx = frame.base + slot;
                    let value = vm
                        .stack
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on store"))?;
                    if idx >= vm.stack.len() {
                        vm.stack.resize(idx + 1, VMValue::Unit);
                    }
                    vm.stack[idx] = value;
                }
                x if x == Opcode::LoadGlobal as u8 => {
                    let idx = Self::read_u16(function, frame)? as usize;
                    let value = vm
                        .globals
                        .get(idx)
                        .cloned()
                        .ok_or_else(|| vm.error(artifact, "global index out of bounds"))?;
                    vm.stack.push(value);
                }
                x if x == Opcode::Pop as u8 => {
                    vm.stack
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on pop"))?;
                }
                x if x == Opcode::Dup as u8 => {
                    let value = vm
                        .stack
                        .last()
                        .cloned()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on dup"))?;
                    vm.stack.push(value);
                }
                x if x == Opcode::Jump as u8 => {
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(next_ip) = frame.ip.checked_add(offset) else {
                        return Err(vm.error(artifact, "jump overflow"));
                    };
                    frame.ip = next_ip;
                }
                x if x == Opcode::JumpIfFalse as u8 => {
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(cond) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on conditional jump"));
                    };
                    match cond {
                        VMValue::Bool(false) => {
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        VMValue::Bool(true) => {}
                        _ => return Err(vm.error(artifact, "conditional jump requires a Bool")),
                    }
                }
                x if x == Opcode::MatchFail as u8 => {
                    return Err(vm.error(artifact, "non-exhaustive match"));
                }
                x if x == Opcode::ChainCheck as u8 => {
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on chain_check"));
                    };
                    match value {
                        VMValue::Variant(tag, payload) if tag == "ok" || tag == "some" => {
                            let unwrapped = payload.map(|inner| *inner).ok_or_else(|| {
                                vm.error(artifact, "chain_check expected payload")
                            })?;
                            vm.stack.push(unwrapped);
                        }
                        VMValue::Variant(tag, payload) if tag == "err" => {
                            vm.stack.push(VMValue::Variant(tag, payload));
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        VMValue::Variant(tag, None) if tag == "none" => {
                            vm.stack.push(VMValue::Variant(tag, None));
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        other => {
                            return Err(vm.error(
                                artifact,
                                &format!(
                                    "chain_check requires ok/some/err/none variant, got {other:?}"
                                ),
                            ));
                        }
                    }
                }
                x if x == Opcode::JumpIfNotVariant as u8 => {
                    let name_idx = Self::read_u16(function, frame)? as usize;
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(expected) = artifact.str_table.get(name_idx).cloned() else {
                        return Err(vm.error(artifact, "variant name index out of bounds"));
                    };
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on variant check"));
                    };
                    match value {
                        VMValue::Variant(tag, payload) if tag == expected => {
                            vm.stack.push(VMValue::Variant(tag, payload));
                        }
                        other => {
                            vm.stack.push(other);
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                    }
                }
                x if x == Opcode::GetField as u8 => {
                    let idx = Self::read_u16(function, frame)? as usize;
                    let field_name = artifact
                        .str_table
                        .get(idx)
                        .cloned()
                        .ok_or_else(|| vm.error(artifact, "field name index out of bounds"))?;
                    let value = vm
                        .stack
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on get_field"))?;
                    match value {
                        VMValue::Record(map) => {
                            let field = map.get(&field_name).cloned().ok_or_else(|| {
                                vm.error(artifact, &format!("missing record field `{field_name}`"))
                            })?;
                            vm.stack.push(field);
                        }
                        VMValue::Builtin(ns) => {
                            let full = format!("{}.{}", ns, field_name);
                            // 0-arg numeric constants: evaluate immediately so `Math.pi`
                            // can be used as a bare expression without parentheses.
                            let value = match full.as_str() {
                                "Math.pi" => VMValue::Float(std::f64::consts::PI),
                                "Math.e" => VMValue::Float(std::f64::consts::E),
                                _ => VMValue::Builtin(full),
                            };
                            vm.stack.push(value);
                        }
                        _ => return Err(vm.error(artifact, "get_field requires a record value")),
                    }
                }
                x if x == Opcode::BuildRecord as u8 => {
                    let field_count = Self::read_u16(function, frame)? as usize;
                    let names_idx = Self::read_u16(function, frame)? as usize;
                    let names = artifact.str_table.get(names_idx).cloned().ok_or_else(|| {
                        vm.error(artifact, "record field names index out of bounds")
                    })?;
                    let field_names: Vec<&str> = if names.is_empty() {
                        Vec::new()
                    } else {
                        names.split('\u{1f}').collect()
                    };
                    if field_names.len() != field_count {
                        return Err(vm.error(artifact, "record field name count mismatch"));
                    }
                    let mut values = Vec::with_capacity(field_count);
                    for _ in 0..field_count {
                        values.push(vm.stack.pop().ok_or_else(|| {
                            vm.error(artifact, "stack underflow on build_record")
                        })?);
                    }
                    values.reverse();
                    let mut map = HashMap::with_capacity(field_count);
                    for (name, value) in field_names.into_iter().zip(values.into_iter()) {
                        map.insert(name.to_string(), value);
                    }
                    vm.stack.push(VMValue::Record(map));
                }
                x if x == Opcode::MakeClosure as u8 => {
                    let global_idx = Self::read_u16(function, frame)? as usize;
                    let capture_count = Self::read_u16(function, frame)? as usize;
                    let mut captures = Vec::with_capacity(capture_count);
                    for _ in 0..capture_count {
                        captures.push(vm.stack.pop().ok_or_else(|| {
                            vm.error(artifact, "stack underflow on make_closure")
                        })?);
                    }
                    captures.reverse();
                    let target =
                        vm.globals.get(global_idx).cloned().ok_or_else(|| {
                            vm.error(artifact, "closure global index out of bounds")
                        })?;
                    match target {
                        VMValue::CompiledFn(fn_idx) => {
                            vm.stack.push(VMValue::Closure(fn_idx, captures))
                        }
                        _ => {
                            return Err(vm.error(
                                artifact,
                                "make_closure requires a function global target",
                            ));
                        }
                    }
                }
                x if x == Opcode::GetVariantPayload as u8 => {
                    let value = vm.stack.pop().ok_or_else(|| {
                        vm.error(artifact, "stack underflow on get_variant_payload")
                    })?;
                    match value {
                        VMValue::Variant(_, Some(payload)) => vm.stack.push(*payload),
                        VMValue::Variant(_, None) => {
                            return Err(vm.error(artifact, "variant has no payload"));
                        }
                        _ => {
                            return Err(
                                vm.error(artifact, "get_variant_payload requires a variant")
                            );
                        }
                    }
                }
                x if x == Opcode::CollectBegin as u8 => {
                    vm.collect_frames.push(Vec::new());
                }
                x if x == Opcode::CollectEnd as u8 => {
                    let values = vm
                        .collect_frames
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "collect_end without collect_begin"))?;
                    vm.stack.push(VMValue::List(values));
                }
                x if x == Opcode::YieldValue as u8 => {
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on yield"));
                    };
                    let Some(collect_frame) = vm.collect_frames.last_mut() else {
                        return Err(vm.error(artifact, "yield outside collect"));
                    };
                    collect_frame.push(value);
                }
                x if x == Opcode::EmitEvent as u8 => {
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on emit"));
                    };
                    vm.emit_log.push(value);
                    vm.stack.push(VMValue::Unit);
                }
                x if x == Opcode::Call as u8 => {
                    let arg_count = Self::read_u16(function, frame)? as usize;
                    let callee_pos = vm
                        .stack
                        .len()
                        .checked_sub(arg_count + 1)
                        .ok_or_else(|| vm.error(artifact, "stack underflow on call"))?;
                    let callee = vm.stack[callee_pos].clone();
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(
                            vm.stack
                                .pop()
                                .ok_or_else(|| vm.error(artifact, "stack underflow on call"))?,
                        );
                    }
                    args.reverse();
                    vm.stack.remove(callee_pos);
                    let result = vm.call_value(artifact, callee, args)?;
                    vm.stack.push(result);
                }
                x if x == Opcode::Add as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(apply_numeric_binop(
                        left,
                        right,
                        |a, b| a + b,
                        |a, b| a + b,
                        "add",
                        artifact,
                        &vm.frames,
                    )?);
                }
                x if x == Opcode::Sub as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(apply_numeric_binop(
                        left,
                        right,
                        |a, b| a - b,
                        |a, b| a - b,
                        "sub",
                        artifact,
                        &vm.frames,
                    )?);
                }
                x if x == Opcode::Mul as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(apply_numeric_binop(
                        left,
                        right,
                        |a, b| a * b,
                        |a, b| a * b,
                        "mul",
                        artifact,
                        &vm.frames,
                    )?);
                }
                x if x == Opcode::Div as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    let division_by_zero = match (&left, &right) {
                        (VMValue::Int(_), VMValue::Int(0)) => true,
                        (VMValue::Float(_), VMValue::Float(v)) => *v == 0.0,
                        (VMValue::Int(_), VMValue::Float(v)) => *v == 0.0,
                        (VMValue::Float(_), VMValue::Int(0)) => true,
                        _ => false,
                    };
                    if division_by_zero {
                        return Err(vm_error_from_frames(
                            artifact,
                            &vm.frames,
                            "division by zero".to_string(),
                        ));
                    }
                    vm.stack.push(apply_numeric_binop(
                        left,
                        right,
                        |a, b| a / b,
                        |a, b| a / b,
                        "div",
                        artifact,
                        &vm.frames,
                    )?);
                }
                x if x == Opcode::And as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    match (left, right) {
                        (VMValue::Bool(a), VMValue::Bool(b)) => {
                            vm.stack.push(VMValue::Bool(a && b))
                        }
                        (left, right) => {
                            return Err(vm.error(
                                artifact,
                                &format!(
                                    "logical and requires Bool operands, got {} and {}",
                                    vmvalue_type_name(&left),
                                    vmvalue_type_name(&right)
                                ),
                            ));
                        }
                    }
                }
                x if x == Opcode::Or as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    match (left, right) {
                        (VMValue::Bool(a), VMValue::Bool(b)) => {
                            vm.stack.push(VMValue::Bool(a || b))
                        }
                        (left, right) => {
                            return Err(vm.error(
                                artifact,
                                &format!(
                                    "logical or requires Bool operands, got {} and {}",
                                    vmvalue_type_name(&left),
                                    vmvalue_type_name(&right)
                                ),
                            ));
                        }
                    }
                }
                x if x == Opcode::Eq as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(VMValue::Bool(left == right));
                }
                x if x == Opcode::Ne as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(VMValue::Bool(left != right));
                }
                x if x == Opcode::Lt as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack
                        .push(compare_pair(pair, |a, b| a < b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Le as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack
                        .push(compare_pair(pair, |a, b| a <= b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Gt as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack
                        .push(compare_pair(pair, |a, b| a > b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Ge as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack
                        .push(compare_pair(pair, |a, b| a >= b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Return as u8 => {
                    let ret = vm.stack.pop().unwrap_or(VMValue::Unit);
                    let frame = vm.frames.pop().expect("frame exists");
                    vm.stack.truncate(frame.base);
                    if vm.frames.len() == caller_depth {
                        return Ok(ret);
                    }
                    vm.stack.push(ret);
                }
                x if x == Opcode::TrackLine as u8 => {
                    if frame.ip + 3 >= function.code.len() {
                        return Err(vm.error(artifact, "TrackLine: unexpected end of bytecode"));
                    }
                    let b0 = function.code[frame.ip];
                    let b1 = function.code[frame.ip + 1];
                    let b2 = function.code[frame.ip + 2];
                    let b3 = function.code[frame.ip + 3];
                    frame.ip += 4;
                    let line = u32::from_le_bytes([b0, b1, b2, b3]);
                    frame.line = line;
                    COVERED_LINES.with(|c| {
                        if let Some(set) = c.borrow_mut().as_mut() {
                            set.insert(line);
                        }
                    });
                }
                other => {
                    return Err(vm.error(artifact, &format!("unsupported opcode: 0x{other:02x}")));
                }
            }
        }
    }

    fn read_u16(
        function: &crate::backend::artifact::FvcFunction,
        frame: &mut CallFrame,
    ) -> Result<u16, VMError> {
        if frame.ip + 1 >= function.code.len() {
            return Err(VMError {
                message: "unexpected end of bytecode".to_string(),
                fn_name: "<decode>".to_string(),
                ip: frame.ip,
                stack_trace: vec![],
            });
        }
        let lo = function.code[frame.ip];
        let hi = function.code[frame.ip + 1];
        frame.ip += 2;
        Ok(u16::from_le_bytes([lo, hi]))
    }

    fn error(&self, artifact: &FvcArtifact, message: &str) -> VMError {
        if let Some(frame) = self.frames.last() {
            let function = &artifact.functions[frame.fn_idx];
            let fn_name = artifact
                .str_table
                .get(function.name_idx as usize)
                .cloned()
                .unwrap_or_else(|| "<unknown>".to_string());
            VMError {
                message: message.to_string(),
                fn_name,
                ip: frame.ip,
                stack_trace: build_stack_trace(artifact, &self.frames),
            }
        } else {
            VMError {
                message: message.to_string(),
                fn_name: "<none>".to_string(),
                ip: 0,
                stack_trace: vec![],
            }
        }
    }

    fn call_value(
        &mut self,
        artifact: &FvcArtifact,
        callee: VMValue,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        match callee {
            VMValue::CompiledFn(target_idx) => self.invoke_function(artifact, target_idx, args),
            VMValue::Closure(target_idx, captures) => {
                let mut full_args = captures;
                full_args.extend(args);
                self.invoke_function(artifact, target_idx, full_args)
            }
            VMValue::VariantCtor(name) => {
                let payload = match args.len() {
                    0 => None,
                    1 => Some(Box::new(args.into_iter().next().expect("single payload"))),
                    _ => {
                        return Err(self
                            .error(artifact, "variant constructor call expects 0 or 1 argument"));
                    }
                };
                Ok(VMValue::Variant(name, payload))
            }
            VMValue::Builtin(name) => self.call_builtin(artifact, &name, args),
            _ => Err(self.error(artifact, "attempted to call a non-function value")),
        }
    }

    fn call_builtin(
        &mut self,
        artifact: &FvcArtifact,
        name: &str,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        match name {
            "List.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(xs) => {
                        let mut out = Vec::with_capacity(xs.len());
                        for x in xs {
                            out.push(self.call_value(artifact, func.clone(), vec![x])?);
                        }
                        Ok(VMValue::List(out))
                    }
                    _ => Err(self.error(artifact, "List.map requires a List as first argument")),
                }
            }
            "List.filter" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.filter requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(xs) => {
                        let mut out = Vec::new();
                        for x in xs {
                            let keep = self.call_value(artifact, func.clone(), vec![x.clone()])?;
                            match keep {
                                VMValue::Bool(true) => out.push(x),
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.filter predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::List(out))
                    }
                    _ => Err(self.error(artifact, "List.filter requires a List as first argument")),
                }
            }
            "List.fold" => {
                if args.len() != 3 {
                    return Err(self.error(artifact, "List.fold requires 3 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let mut acc = it.next().expect("init");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(xs) => {
                        for x in xs {
                            acc = self.call_value(artifact, func.clone(), vec![acc, x])?;
                        }
                        Ok(acc)
                    }
                    _ => Err(self.error(artifact, "List.fold requires a List as first argument")),
                }
            }
            "List.flat_map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.flat_map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(xs) => {
                        let mut out: Vec<VMValue> = Vec::new();
                        for x in xs {
                            match self.call_value(artifact, func.clone(), vec![x])? {
                                VMValue::List(inner) => out.extend(inner),
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.flat_map: callback must return List, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::List(out))
                    }
                    _ => {
                        Err(self.error(artifact, "List.flat_map requires a List as first argument"))
                    }
                }
            }
            "List.sort" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.sort requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let cmp = it.next().expect("cmp");
                match list {
                    VMValue::List(mut xs) => {
                        let mut sort_err: Option<VMError> = None;
                        xs.sort_by(|a, b| {
                            if sort_err.is_some() {
                                return std::cmp::Ordering::Equal;
                            }
                            match self.call_value(artifact, cmp.clone(), vec![a.clone(), b.clone()])
                            {
                                Ok(VMValue::Int(n)) => {
                                    if n < 0 {
                                        std::cmp::Ordering::Less
                                    } else if n > 0 {
                                        std::cmp::Ordering::Greater
                                    } else {
                                        std::cmp::Ordering::Equal
                                    }
                                }
                                Ok(other) => {
                                    sort_err = Some(self.error(
                                        artifact,
                                        &format!(
                                            "List.sort: comparator must return Int, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                    std::cmp::Ordering::Equal
                                }
                                Err(e) => {
                                    sort_err = Some(e);
                                    std::cmp::Ordering::Equal
                                }
                            }
                        });
                        if let Some(e) = sort_err {
                            return Err(e);
                        }
                        Ok(VMValue::List(xs))
                    }
                    _ => Err(self.error(artifact, "List.sort requires a List as first argument")),
                }
            }
            "List.find" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.find requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(xs) => {
                        for x in xs {
                            match self.call_value(artifact, pred.clone(), vec![x.clone()])? {
                                VMValue::Bool(true) => {
                                    return Ok(VMValue::Variant("some".into(), Some(Box::new(x))));
                                }
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.find predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Variant("none".into(), None))
                    }
                    _ => Err(self.error(artifact, "List.find requires a List as first argument")),
                }
            }
            "List.any" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.any requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(xs) => {
                        for x in xs {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(true) => return Ok(VMValue::Bool(true)),
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.any predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Bool(false))
                    }
                    _ => Err(self.error(artifact, "List.any requires a List as first argument")),
                }
            }
            "List.all" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.all requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(xs) => {
                        for x in xs {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(false) => return Ok(VMValue::Bool(false)),
                                VMValue::Bool(true) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.all predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Bool(true))
                    }
                    _ => Err(self.error(artifact, "List.all requires a List as first argument")),
                }
            }
            "List.count" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.count requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(xs) => {
                        let mut count = 0i64;
                        for x in xs {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(true) => count += 1,
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.count predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Int(count))
                    }
                    _ => Err(self.error(artifact, "List.count requires a List as first argument")),
                }
            }
            "List.index_of" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.index_of requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(xs) => {
                        for (i, x) in xs.into_iter().enumerate() {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(true) => {
                                    return Ok(VMValue::Variant(
                                        "some".into(),
                                        Some(Box::new(VMValue::Int(i as i64))),
                                    ));
                                }
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.index_of predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Variant("none".into(), None))
                    }
                    _ => {
                        Err(self.error(artifact, "List.index_of requires a List as first argument"))
                    }
                }
            }
            "Map.map_values" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Map.map_values requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let map = it.next().expect("map");
                let func = it.next().expect("func");
                match map {
                    VMValue::Record(m) => {
                        let mut out = HashMap::with_capacity(m.len());
                        for (k, v) in m {
                            let mapped = self.call_value(artifact, func.clone(), vec![v])?;
                            out.insert(k, mapped);
                        }
                        Ok(VMValue::Record(out))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Map.map_values requires a Map as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Map.filter_values" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Map.filter_values requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let map = it.next().expect("map");
                let func = it.next().expect("func");
                match map {
                    VMValue::Record(m) => {
                        let mut out = HashMap::new();
                        for (k, v) in m {
                            let keep = self.call_value(artifact, func.clone(), vec![v.clone()])?;
                            match keep {
                                VMValue::Bool(true) => {
                                    out.insert(k, v);
                                }
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "Map.filter_values predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Record(out))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Map.filter_values requires a Map as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let func = it.next().expect("func");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Option.map expected payload for some")
                        })?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        Ok(VMValue::Variant("some".to_string(), Some(Box::new(mapped))))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        Ok(VMValue::Variant("none".to_string(), None))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.map requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.and_then" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.and_then requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let func = it.next().expect("func");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Option.and_then expected payload for some")
                        })?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        match mapped {
                            VMValue::Variant(tag, payload) if tag == "some" || tag == "none" => {
                                Ok(VMValue::Variant(tag, payload))
                            }
                            other => Err(self.error(
                                artifact,
                                &format!(
                                    "Option.and_then callback must return Option, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            )),
                        }
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        Ok(VMValue::Variant("none".to_string(), None))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.and_then requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.unwrap_or" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.unwrap_or requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let default = it.next().expect("default");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Option.unwrap_or expected payload for some")
                        })
                    }
                    VMValue::Variant(tag, None) if tag == "none" => Ok(default),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.unwrap_or requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.or_else" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.or_else requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let func = it.next().expect("func");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        Ok(VMValue::Variant(tag, payload))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        let mapped = self.call_value(artifact, func, vec![])?;
                        match mapped {
                            VMValue::Variant(tag, payload) if tag == "some" || tag == "none" => {
                                Ok(VMValue::Variant(tag, payload))
                            }
                            other => Err(self.error(
                                artifact,
                                &format!(
                                    "Option.or_else callback must return Option, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            )),
                        }
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.or_else requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.is_some" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Option.is_some requires 1 argument"));
                }
                match args.into_iter().next().expect("option") {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        Ok(VMValue::Bool(payload.is_some()))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => Ok(VMValue::Bool(false)),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.is_some requires an Option argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.is_none" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Option.is_none requires 1 argument"));
                }
                match args.into_iter().next().expect("option") {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        Ok(VMValue::Bool(payload.is_none()))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => Ok(VMValue::Bool(true)),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.is_none requires an Option argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.to_result" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.to_result requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let err = it.next().expect("err");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Option.to_result expected payload for some")
                        })?;
                        Ok(VMValue::Variant("ok".to_string(), Some(Box::new(inner))))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        Ok(VMValue::Variant("err".to_string(), Some(Box::new(err))))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.to_result requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let func = it.next().expect("func");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.map expected payload for ok")
                        })?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        Ok(VMValue::Variant("ok".to_string(), Some(Box::new(mapped))))
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        Ok(VMValue::Variant(tag, payload))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.map requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.map_err" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.map_err requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let func = it.next().expect("func");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        Ok(VMValue::Variant(tag, payload))
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.map_err expected payload for err")
                        })?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        Ok(VMValue::Variant("err".to_string(), Some(Box::new(mapped))))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.map_err requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.and_then" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.and_then requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let func = it.next().expect("func");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.and_then expected payload for ok")
                        })?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        match mapped {
                            VMValue::Variant(tag, payload) if tag == "ok" || tag == "err" => {
                                Ok(VMValue::Variant(tag, payload))
                            }
                            other => Err(self.error(
                                artifact,
                                &format!(
                                    "Result.and_then callback must return Result, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            )),
                        }
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        Ok(VMValue::Variant(tag, payload))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.and_then requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.unwrap_or" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.unwrap_or requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let default = it.next().expect("default");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.unwrap_or expected payload for ok")
                        })
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        let _ = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.unwrap_or expected payload for err")
                        })?;
                        Ok(default)
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.unwrap_or requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.is_ok" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Result.is_ok requires 1 argument"));
                }
                match args.into_iter().next().expect("result") {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        Ok(VMValue::Bool(payload.is_some()))
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        Ok(VMValue::Bool(false && payload.is_some()))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.is_ok requires a Result argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.is_err" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Result.is_err requires 1 argument"));
                }
                match args.into_iter().next().expect("result") {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        Ok(VMValue::Bool(false && payload.is_some()))
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        Ok(VMValue::Bool(payload.is_some()))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.is_err requires a Result argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.to_option" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Result.to_option requires 1 argument"));
                }
                match args.into_iter().next().expect("result") {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        Ok(VMValue::Variant("some".to_string(), payload))
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        let _ = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.to_option expected payload for err")
                        })?;
                        Ok(VMValue::Variant("none".to_string(), None))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.to_option requires a Result argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            // Stream builtins (v2.9.0)
            "Stream.from" | "Stream.of" => {
                let mut it = args.into_iter();
                let list = it
                    .next()
                    .ok_or_else(|| self.error(artifact, "Stream.from requires 1 argument"))?;
                match list {
                    VMValue::List(xs) => Ok(VMValue::Stream(Box::new(VMStream::Of(xs)))),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.from requires a List argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.gen" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Stream.gen requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let seed = it.next().expect("seed");
                let next_fn = it.next().expect("next_fn");
                Ok(VMValue::Stream(Box::new(VMStream::Gen { seed, next_fn })))
            }
            "Stream.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Stream.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let stream = it.next().expect("stream");
                let map_fn = it.next().expect("map_fn");
                match stream {
                    VMValue::Stream(inner) => Ok(VMValue::Stream(Box::new(VMStream::Map {
                        inner: inner,
                        map_fn,
                    }))),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.map requires a Stream as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.filter" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Stream.filter requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let stream = it.next().expect("stream");
                let pred_fn = it.next().expect("pred_fn");
                match stream {
                    VMValue::Stream(inner) => Ok(VMValue::Stream(Box::new(VMStream::Filter {
                        inner: inner,
                        pred_fn,
                    }))),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.filter requires a Stream as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.take" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Stream.take requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let stream = it.next().expect("stream");
                let n_val = it.next().expect("n");
                match (stream, n_val) {
                    (VMValue::Stream(inner), VMValue::Int(n)) => {
                        Ok(VMValue::Stream(Box::new(VMStream::Take {
                            inner: inner,
                            n,
                        })))
                    }
                    (VMValue::Stream(_), other) => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.take second argument must be Int, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                    (other, _) => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.take requires a Stream as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.to_list" => {
                let mut it = args.into_iter();
                let stream = it
                    .next()
                    .ok_or_else(|| self.error(artifact, "Stream.to_list requires 1 argument"))?;
                match stream {
                    VMValue::Stream(s) => {
                        let items = self.materialize_stream(artifact, *s)?;
                        Ok(VMValue::List(items))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.to_list requires a Stream argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Http.serve_raw" => {
                if args.len() != 3 {
                    return Err(self.error(artifact, "Http.serve_raw requires 3 arguments"));
                }
                let mut it = args.into_iter();
                let port = match it.next().expect("port") {
                    VMValue::Int(port) => port,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Http.serve_raw expects Int port, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let routes = match it.next().expect("routes") {
                    VMValue::List(routes) => routes,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Http.serve_raw expects List<Map<String,String>>, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let handler_name = match it.next().expect("handler_name") {
                    VMValue::Str(name) => name,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Http.serve_raw expects String handler_name, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let server = tiny_http::Server::http(format!("0.0.0.0:{port}")).map_err(|e| {
                    self.error(artifact, &format!("Http.serve_raw bind failed: {}", e))
                })?;
                let mut request = server.recv().map_err(|e| {
                    self.error(artifact, &format!("Http.serve_raw recv failed: {}", e))
                })?;
                let method = request.method().as_str().to_string();
                let path = request.url().to_string();
                let mut body = String::new();
                let mut reader = request.as_reader();
                std::io::Read::read_to_string(&mut reader, &mut body).map_err(|e| {
                    self.error(artifact, &format!("Http.serve_raw body read failed: {}", e))
                })?;

                let route_allowed = routes.into_iter().any(|route| match route {
                    VMValue::Record(map) => {
                        let route_method = map.get("method").map(vm_scalar_to_plain_string);
                        let route_path = map.get("path").map(vm_scalar_to_plain_string);
                        route_method.as_deref().unwrap_or("") == method
                            && route_path.as_deref().unwrap_or("") == path
                    }
                    _ => false,
                });

                let response_value = if route_allowed {
                    let fn_idx = artifact.fn_idx_by_name(&handler_name).ok_or_else(|| {
                        self.error(
                            artifact,
                            &format!("Http.serve_raw unknown handler `{}`", handler_name),
                        )
                    })?;
                    let function = &artifact.functions[fn_idx];
                    let args = match function.param_count {
                        0 => vec![],
                        1 => {
                            let mut req = HashMap::new();
                            req.insert("method".to_string(), VMValue::Str(method.clone()));
                            req.insert("path".to_string(), VMValue::Str(path.clone()));
                            req.insert("body".to_string(), VMValue::Str(body.clone()));
                            vec![VMValue::Record(req)]
                        }
                        3 => vec![
                            VMValue::Str(method.clone()),
                            VMValue::Str(path.clone()),
                            VMValue::Str(body.clone()),
                        ],
                        other => {
                            return Err(self.error(
                                artifact,
                                &format!(
                                    "Http.serve_raw handler `{}` must take 0, 1, or 3 args, got {}",
                                    handler_name, other
                                ),
                            ));
                        }
                    };
                    self.invoke_function(artifact, fn_idx, args)?
                } else {
                    http_response_vm(404, "not found".to_string(), "text/plain".to_string())
                };

                let (status, resp_body, content_type) = match response_value {
                    VMValue::Record(map) => {
                        let status = match map.get("status") {
                            Some(VMValue::Int(n)) => *n as u16,
                            _ => 200,
                        };
                        let body = map
                            .get("body")
                            .map(vm_scalar_to_plain_string)
                            .unwrap_or_default();
                        let content_type = map
                            .get("content_type")
                            .map(vm_scalar_to_plain_string)
                            .unwrap_or_else(|| "text/plain".to_string());
                        (status, body, content_type)
                    }
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Http.serve_raw handler must return HttpResponse record, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let response = tiny_http::Response::from_string(resp_body)
                    .with_status_code(status)
                    .with_header(
                        tiny_http::Header::from_bytes(
                            b"Content-Type".as_slice(),
                            content_type.as_bytes(),
                        )
                        .map_err(|_| {
                            self.error(artifact, "Http.serve_raw invalid Content-Type header")
                        })?,
                    );
                request.respond(response).map_err(|e| {
                    self.error(artifact, &format!("Http.serve_raw respond failed: {}", e))
                })?;
                Ok(VMValue::Unit)
            }
            "Grpc.serve_raw" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Grpc.serve_raw requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let port = match it.next().expect("port") {
                    VMValue::Int(port) => port,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Grpc.serve_raw expects Int port, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let _service_name = match it.next().expect("service_name") {
                    VMValue::Str(name) => name,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Grpc.serve_raw expects String service_name, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let (req_tx, req_rx) =
                    std::sync::mpsc::channel::<GrpcRequestMsg>();
                grpc_serve_impl(port, req_tx)
                    .map_err(|e| self.error(artifact, &format!("Grpc.serve_raw failed: {}", e)))?;
                loop {
                    let (handler_name, proto_bytes, res_tx) = match req_rx.recv() {
                        Ok(msg) => msg,
                        Err(_) => break,
                    };
                    let fn_idx = match artifact.fn_idx_by_name(&handler_name) {
                        Some(idx) => idx,
                        None => {
                            let _ = res_tx.send(Err(format!(
                                "Grpc.serve_raw: unknown handler `{}`",
                                handler_name
                            )));
                            continue;
                        }
                    };
                    let req_value = match proto_bytes_to_string_map(&proto_bytes) {
                        Ok(row) => VMValue::Record(
                            row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
                        ),
                        Err(e) => {
                            let _ = res_tx.send(Err(format!("proto decode failed: {}", e)));
                            continue;
                        }
                    };
                    let result = self.invoke_function(artifact, fn_idx, vec![req_value]);
                    let resp =
                        grpc_vm_value_to_proto_bytes(result.map_err(|e| e.message));
                    let _ = res_tx.send(resp.map(|b| encode_grpc_frame(&b)));
                }
                Ok(VMValue::Unit)
            }
            "Grpc.serve_stream_raw" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Grpc.serve_stream_raw requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let port = match it.next().expect("port") {
                    VMValue::Int(port) => port,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Grpc.serve_stream_raw expects Int port, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let _service_name = match it.next().expect("service_name") {
                    VMValue::Str(name) => name,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Grpc.serve_stream_raw expects String service_name, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let (req_tx, req_rx) =
                    std::sync::mpsc::channel::<GrpcRequestMsg>();
                grpc_serve_impl(port, req_tx).map_err(|e| {
                    self.error(artifact, &format!("Grpc.serve_stream_raw failed: {}", e))
                })?;
                loop {
                    let (handler_name, proto_bytes, res_tx) = match req_rx.recv() {
                        Ok(msg) => msg,
                        Err(_) => break,
                    };
                    let fn_idx = match artifact.fn_idx_by_name(&handler_name) {
                        Some(idx) => idx,
                        None => {
                            let _ = res_tx.send(Err(format!(
                                "Grpc.serve_stream_raw: unknown handler `{}`",
                                handler_name
                            )));
                            continue;
                        }
                    };
                    let req_value = match proto_bytes_to_string_map(&proto_bytes) {
                        Ok(row) => VMValue::Record(
                            row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
                        ),
                        Err(e) => {
                            let _ = res_tx.send(Err(format!("proto decode failed: {}", e)));
                            continue;
                        }
                    };
                    let result = self.invoke_function(artifact, fn_idx, vec![req_value]);
                    let frames = match result {
                        Ok(VMValue::List(items)) => {
                            let mut combined: Vec<u8> = Vec::new();
                            let mut ok = true;
                            for item in items {
                                match grpc_vm_value_to_proto_bytes(Ok(item)) {
                                    Ok(b) => {
                                        combined.extend_from_slice(&encode_grpc_frame(&b));
                                    }
                                    Err(e) => {
                                        let _ = res_tx.send(Err(e));
                                        ok = false;
                                        break;
                                    }
                                }
                            }
                            if !ok {
                                continue;
                            }
                            Ok(combined)
                        }
                        Ok(other) => Err(format!(
                            "Grpc.serve_stream_raw handler must return List, got {}",
                            vmvalue_type_name(&other)
                        )),
                        Err(e) => Err(e.message),
                    };
                    let _ = res_tx.send(frames);
                }
                Ok(VMValue::Unit)
            }
            _ => {
                if let Some(target_idx) = artifact.globals.iter().position(|g| {
                    g.kind == 0
                        && artifact
                            .str_table
                            .get(g.name_idx as usize)
                            .is_some_and(|n| n == name)
                }) {
                    return self.call_value(
                        artifact,
                        VMValue::CompiledFn(artifact.globals[target_idx].fn_idx as usize),
                        args,
                    );
                }
                vm_call_builtin(
                    name,
                    args,
                    &mut self.emit_log,
                    self.db_path.as_deref(),
                    &self.type_metas,
                )
                .map_err(|e| self.error(artifact, &e))
            }
        }
    }

    /// Materialize a lazy `VMStream` into a `Vec<VMValue>`.
    fn materialize_stream(
        &mut self,
        artifact: &FvcArtifact,
        stream: VMStream,
    ) -> Result<Vec<VMValue>, VMError> {
        match stream {
            VMStream::Of(items) => Ok(items),
            VMStream::Gen { .. } => Err(self.error(
                artifact,
                "cannot collect an infinite stream without Stream.take",
            )),
            VMStream::Map { inner, map_fn } => {
                let items = self.materialize_stream(artifact, *inner)?;
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.call_value(artifact, map_fn.clone(), vec![item])?);
                }
                Ok(out)
            }
            VMStream::Filter { inner, pred_fn } => {
                let items = self.materialize_stream(artifact, *inner)?;
                let mut out = Vec::new();
                for item in items {
                    let keep = self.call_value(artifact, pred_fn.clone(), vec![item.clone()])?;
                    match keep {
                        VMValue::Bool(true) => out.push(item),
                        VMValue::Bool(false) => {}
                        other => {
                            return Err(self.error(
                                artifact,
                                &format!(
                                    "Stream.filter predicate must return Bool, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            ));
                        }
                    }
                }
                Ok(out)
            }
            VMStream::Take { inner, n } => {
                let n_usize = if n < 0 { 0 } else { n as usize };
                match *inner {
                    VMStream::Gen { seed, next_fn } => {
                        let mut result = Vec::with_capacity(n_usize);
                        let mut current = seed;
                        for _ in 0..n_usize {
                            result.push(current.clone());
                            current = self.call_value(artifact, next_fn.clone(), vec![current])?;
                        }
                        Ok(result)
                    }
                    other => {
                        let items = self.materialize_stream(artifact, other)?;
                        Ok(items.into_iter().take(n_usize).collect())
                    }
                }
            }
        }
    }

    fn pop_pair(&mut self, artifact: &FvcArtifact) -> Result<(VMValue, VMValue), VMError> {
        let right = self
            .stack
            .pop()
            .ok_or_else(|| self.error(artifact, "stack underflow"))?;
        let left = self
            .stack
            .pop()
            .ok_or_else(|| self.error(artifact, "stack underflow"))?;
        Ok((left, right))
    }
}

fn constant_to_value(constant: Constant) -> VMValue {
    match constant {
        Constant::Int(v) => VMValue::Int(v),
        Constant::Float(v) => VMValue::Float(v),
        Constant::Str(v) => VMValue::Str(v),
        Constant::Name(v) => VMValue::Str(v),
    }
}

impl From<Value> for VMValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(v) => VMValue::Bool(v),
            Value::Int(v) => VMValue::Int(v),
            Value::Float(v) => VMValue::Float(v),
            Value::Str(v) => VMValue::Str(v),
            Value::Unit => VMValue::Unit,
            Value::List(values) => VMValue::List(values.into_iter().map(VMValue::from).collect()),
            Value::Record(map) => VMValue::Record(
                map.into_iter()
                    .map(|(k, v)| (k, VMValue::from(v)))
                    .collect(),
            ),
            Value::Variant(tag, payload) => {
                VMValue::Variant(tag, payload.map(|inner| Box::new(VMValue::from(*inner))))
            }
            other => panic!("unsupported VM argument value: {other:?}"),
        }
    }
}

impl From<VMValue> for Value {
    fn from(value: VMValue) -> Self {
        match value {
            VMValue::Bool(v) => Value::Bool(v),
            VMValue::Int(v) => Value::Int(v),
            VMValue::Float(v) => Value::Float(v),
            VMValue::Str(v) => Value::Str(v),
            VMValue::Unit => Value::Unit,
            VMValue::List(values) => Value::List(values.into_iter().map(Value::from).collect()),
            VMValue::Record(map) => {
                Value::Record(map.into_iter().map(|(k, v)| (k, Value::from(v))).collect())
            }
            VMValue::Variant(tag, payload) => {
                Value::Variant(tag, payload.map(|inner| Box::new(Value::from(*inner))))
            }
            VMValue::VariantCtor(name) => Value::Variant(name, None),
            VMValue::CompiledFn(idx) => Value::Str(format!("<fn:{idx}>")),
            VMValue::Closure(idx, captures) => {
                Value::Str(format!("<closure:{idx};captures={}>", captures.len()))
            }
            VMValue::Builtin(name) => Value::Str(format!("<builtin:{name}>")),
            VMValue::Stream(_) => Value::Str("<stream>".to_string()),
            VMValue::DbHandle(id) => Value::Str(format!("<db:{id}>")),
            VMValue::TxHandle(id) => Value::Str(format!("<tx:{id}>")),
        }
    }
}

fn apply_numeric_binop(
    left: VMValue,
    right: VMValue,
    int_op: impl FnOnce(i64, i64) -> i64,
    float_op: impl FnOnce(f64, f64) -> f64,
    op_name: &str,
    artifact: &FvcArtifact,
    frames: &[CallFrame],
) -> Result<VMValue, VMError> {
    match (left, right) {
        (VMValue::Int(a), VMValue::Int(b)) => Ok(VMValue::Int(int_op(a, b))),
        (VMValue::Float(a), VMValue::Float(b)) => Ok(VMValue::Float(float_op(a, b))),
        (VMValue::Int(a), VMValue::Float(b)) => Ok(VMValue::Float(float_op(a as f64, b))),
        (VMValue::Float(a), VMValue::Int(b)) => Ok(VMValue::Float(float_op(a, b as f64))),
        _ => Err(vm_error_from_frames(
            artifact,
            frames,
            format!("type error in {op_name}: numeric operands required"),
        )),
    }
}

fn compare_pair(
    pair: (VMValue, VMValue),
    cmp: impl FnOnce(f64, f64) -> bool,
    artifact: &FvcArtifact,
    frames: &[CallFrame],
) -> Result<VMValue, VMError> {
    match pair {
        (VMValue::Int(a), VMValue::Int(b)) => Ok(VMValue::Bool(cmp(a as f64, b as f64))),
        (VMValue::Float(a), VMValue::Float(b)) => Ok(VMValue::Bool(cmp(a, b))),
        (VMValue::Int(a), VMValue::Float(b)) => Ok(VMValue::Bool(cmp(a as f64, b))),
        (VMValue::Float(a), VMValue::Int(b)) => Ok(VMValue::Bool(cmp(a, b as f64))),
        _ => Err(vm_error_from_frames(
            artifact,
            frames,
            "type error in comparison: numeric operands required".to_string(),
        )),
    }
}

fn build_stack_trace(artifact: &FvcArtifact, frames: &[CallFrame]) -> Vec<TraceFrame> {
    frames
        .iter()
        .rev()
        .map(|frame| {
            let function = &artifact.functions[frame.fn_idx];
            let fn_name = artifact
                .str_table
                .get(function.name_idx as usize)
                .cloned()
                .unwrap_or_else(|| "<unknown>".to_string());
            TraceFrame {
                fn_name,
                line: frame.line,
            }
        })
        .collect()
}

fn vm_error_from_frames(artifact: &FvcArtifact, frames: &[CallFrame], message: String) -> VMError {
    let stack_trace = build_stack_trace(artifact, frames);
    if let Some(frame) = frames.last() {
        let top = stack_trace.first().cloned().unwrap_or(TraceFrame {
            fn_name: "<unknown>".to_string(),
            line: 0,
        });
        VMError {
            message,
            fn_name: top.fn_name,
            ip: frame.ip,
            stack_trace,
        }
    } else {
        VMError {
            message,
            fn_name: "<none>".to_string(),
            ip: 0,
            stack_trace,
        }
    }
}

fn vmvalue_repr(v: &VMValue) -> String {
    match v {
        VMValue::Bool(b) => b.to_string(),
        VMValue::Int(n) => n.to_string(),
        VMValue::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.1}", f)
            } else {
                f.to_string()
            }
        }
        VMValue::Str(s) => format!("\"{}\"", s),
        VMValue::Unit => "()".to_string(),
        VMValue::List(vs) => {
            let items: Vec<_> = vs.iter().map(vmvalue_repr).collect();
            format!("[{}]", items.join(", "))
        }
        VMValue::Record(m) => {
            let mut pairs: Vec<_> = m
                .iter()
                .map(|(k, v)| format!("{}: {}", k, vmvalue_repr(v)))
                .collect();
            pairs.sort();
            format!("{{ {} }}", pairs.join(", "))
        }
        VMValue::Variant(name, None) => name.clone(),
        VMValue::Variant(name, Some(payload)) => format!("{}({})", name, vmvalue_repr(payload)),
        VMValue::CompiledFn(idx) => format!("<fn:{}>", idx),
        VMValue::Closure(idx, caps) => format!("<closure:{};captures={}>", idx, caps.len()),
        VMValue::VariantCtor(name) => format!("<ctor:{}>", name),
        VMValue::Builtin(name) => format!("<builtin:{}>", name),
        VMValue::Stream(_) => "<stream>".to_string(),
        VMValue::DbHandle(id) => format!("<db:{}>", id),
        VMValue::TxHandle(id) => format!("<tx:{}>", id),
    }
}

fn vmvalue_type_name(v: &VMValue) -> &'static str {
    match v {
        VMValue::Bool(_) => "Bool",
        VMValue::Int(_) => "Int",
        VMValue::Float(_) => "Float",
        VMValue::Str(_) => "String",
        VMValue::Unit => "Unit",
        VMValue::List(_) => "List",
        VMValue::Record(_) => "Record",
        VMValue::Variant(_, _) => "Variant",
        VMValue::VariantCtor(_) => "VariantCtor",
        VMValue::CompiledFn(_) => "CompiledFn",
        VMValue::Closure(_, _) => "Closure",
        VMValue::Builtin(_) => "Builtin",
        VMValue::Stream(_) => "Stream",
        VMValue::DbHandle(_) => "DbHandle",
        VMValue::TxHandle(_) => "TxHandle",
    }
}

fn json_variant_vm(name: &str, payload: Option<VMValue>) -> VMValue {
    VMValue::Variant(name.to_string(), payload.map(Box::new))
}

fn serde_to_vm_json(value: SerdeJsonValue) -> VMValue {
    match value {
        SerdeJsonValue::Null => json_variant_vm("json_null", None),
        SerdeJsonValue::Bool(b) => json_variant_vm("json_bool", Some(VMValue::Bool(b))),
        SerdeJsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                json_variant_vm("json_int", Some(VMValue::Int(i)))
            } else {
                json_variant_vm(
                    "json_float",
                    Some(VMValue::Float(n.as_f64().unwrap_or(0.0))),
                )
            }
        }
        SerdeJsonValue::String(s) => json_variant_vm("json_str", Some(VMValue::Str(s))),
        SerdeJsonValue::Array(items) => json_variant_vm(
            "json_array",
            Some(VMValue::List(
                items.into_iter().map(serde_to_vm_json).collect(),
            )),
        ),
        SerdeJsonValue::Object(map) => {
            let mut fields = HashMap::new();
            for (k, v) in map {
                fields.insert(k, serde_to_vm_json(v));
            }
            json_variant_vm("json_object", Some(VMValue::Record(fields)))
        }
    }
}

fn vm_json_to_serde(value: &VMValue) -> Option<SerdeJsonValue> {
    match value {
        VMValue::Variant(tag, None) if tag == "json_null" => Some(SerdeJsonValue::Null),
        VMValue::Variant(tag, Some(payload)) if tag == "json_bool" => match payload.as_ref() {
            VMValue::Bool(b) => Some(SerdeJsonValue::Bool(*b)),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_int" => match payload.as_ref() {
            VMValue::Int(i) => Some(SerdeJsonValue::Number((*i).into())),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_float" => match payload.as_ref() {
            VMValue::Float(f) => serde_json::Number::from_f64(*f).map(SerdeJsonValue::Number),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_str" => match payload.as_ref() {
            VMValue::Str(s) => Some(SerdeJsonValue::String(s.clone())),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_array" => match payload.as_ref() {
            VMValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(vm_json_to_serde(item)?);
                }
                Some(SerdeJsonValue::Array(out))
            }
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_object" => match payload.as_ref() {
            VMValue::Record(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    out.insert(k.clone(), vm_json_to_serde(v)?);
                }
                Some(SerdeJsonValue::Object(out))
            }
            _ => None,
        },
        _ => None,
    }
}

fn vm_string(value: VMValue, context: &str) -> Result<String, String> {
    match value {
        VMValue::Str(s) => Ok(s),
        other => Err(format!(
            "{} expects String, got {}",
            context,
            vmvalue_type_name(&other)
        )),
    }
}

fn vm_int(value: VMValue, context: &str) -> Result<i64, String> {
    match value {
        VMValue::Int(n) => Ok(n),
        other => Err(format!(
            "{} expects Int, got {}",
            context,
            vmvalue_type_name(&other)
        )),
    }
}

fn vm_float(value: VMValue, context: &str) -> Result<f64, String> {
    match value {
        VMValue::Float(f) => Ok(f),
        VMValue::Int(n) => Ok(n as f64),
        other => Err(format!(
            "{} expects Float, got {}",
            context,
            vmvalue_type_name(&other)
        )),
    }
}

fn vm_string_list(value: VMValue, context: &str) -> Result<Vec<String>, String> {
    match value {
        VMValue::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(vm_string(item, context)?);
            }
            Ok(out)
        }
        other => Err(format!(
            "{} expects List<String>, got {}",
            context,
            vmvalue_type_name(&other)
        )),
    }
}

fn schema_error_vm(
    field: impl Into<String>,
    expected: impl Into<String>,
    got: impl Into<String>,
) -> VMValue {
    let mut map = HashMap::new();
    map.insert("field".to_string(), VMValue::Str(field.into()));
    map.insert("expected".to_string(), VMValue::Str(expected.into()));
    map.insert("got".to_string(), VMValue::Str(got.into()));
    VMValue::Record(map)
}

fn ok_vm(value: VMValue) -> VMValue {
    VMValue::Variant("ok".to_string(), Some(Box::new(value)))
}

fn err_vm(value: VMValue) -> VMValue {
    VMValue::Variant("err".to_string(), Some(Box::new(value)))
}

fn stringify_json_scalar(value: &SerdeJsonValue) -> Option<String> {
    match value {
        SerdeJsonValue::Null => Some(String::new()),
        SerdeJsonValue::Bool(v) => Some(if *v { "true".into() } else { "false".into() }),
        SerdeJsonValue::Number(v) => Some(v.to_string()),
        SerdeJsonValue::String(v) => Some(v.clone()),
        SerdeJsonValue::Array(_) | SerdeJsonValue::Object(_) => None,
    }
}

fn parse_json_object_raw(text: &str) -> Result<HashMap<String, VMValue>, String> {
    let value: SerdeJsonValue =
        serde_json::from_str(text).map_err(|e| format!("json parse error: {}", e))?;
    let SerdeJsonValue::Object(map) = value else {
        return Err("json parse error: expected object".to_string());
    };
    let mut out = HashMap::new();
    for (key, value) in map {
        let scalar = stringify_json_scalar(&value).ok_or_else(|| {
            "json parse error: nested arrays/objects are not supported".to_string()
        })?;
        out.insert(key, VMValue::Str(scalar));
    }
    Ok(out)
}

fn parse_json_array_raw(text: &str) -> Result<Vec<VMValue>, String> {
    let value: SerdeJsonValue =
        serde_json::from_str(text).map_err(|e| format!("json parse error: {}", e))?;
    let SerdeJsonValue::Array(items) = value else {
        return Err("json parse error: expected array".to_string());
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let SerdeJsonValue::Object(map) = item else {
            return Err("json parse error: expected array of objects".to_string());
        };
        let mut row = HashMap::new();
        for (key, value) in map {
            let scalar = stringify_json_scalar(&value).ok_or_else(|| {
                "json parse error: nested arrays/objects are not supported".to_string()
            })?;
            row.insert(key, VMValue::Str(scalar));
        }
        out.push(VMValue::Record(row));
    }
    Ok(out)
}

fn parse_bool_like(raw: &str) -> Option<bool> {
    match raw {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

fn parse_schema_value(raw: &str, ty: &str, field: &str) -> Result<VMValue, VMValue> {
    if let Some(inner) = ty.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
        if raw.is_empty() {
            return Ok(VMValue::Variant("none".to_string(), None));
        }
        let inner_value = parse_schema_value(raw, inner, field)?;
        return Ok(VMValue::Variant(
            "some".to_string(),
            Some(Box::new(inner_value)),
        ));
    }
    if let Some(inner) = ty.strip_suffix('?') {
        if raw.is_empty() {
            return Ok(VMValue::Variant("none".to_string(), None));
        }
        let inner_value = parse_schema_value(raw, inner, field)?;
        return Ok(VMValue::Variant(
            "some".to_string(),
            Some(Box::new(inner_value)),
        ));
    }

    match ty {
        "Int" => raw
            .parse::<i64>()
            .map(VMValue::Int)
            .map_err(|_| schema_error_vm(field, "Int", raw)),
        "Float" => raw
            .parse::<f64>()
            .map(VMValue::Float)
            .map_err(|_| schema_error_vm(field, "Float", raw)),
        "Bool" => parse_bool_like(raw)
            .map(VMValue::Bool)
            .ok_or_else(|| schema_error_vm(field, "Bool", raw)),
        "String" => Ok(VMValue::Str(raw.to_string())),
        other => Err(schema_error_vm(field, other, raw)),
    }
}

fn schema_rows_from_vm(
    value: VMValue,
    context: &str,
) -> Result<Vec<HashMap<String, VMValue>>, String> {
    match value {
        VMValue::List(rows) => rows
            .into_iter()
            .map(|row| match row {
                VMValue::Record(map) => Ok(map),
                other => Err(format!(
                    "{} expects List<Map<String,String>>, got {}",
                    context,
                    vmvalue_type_name(&other)
                )),
            })
            .collect(),
        other => Err(format!(
            "{} expects List<Map<String,String>>, got {}",
            context,
            vmvalue_type_name(&other)
        )),
    }
}

fn schema_record_to_string_map(record: &HashMap<String, VMValue>) -> HashMap<String, String> {
    record
        .iter()
        .map(|(k, v)| {
            let value = vm_scalar_to_plain_string(v);
            (k.clone(), value)
        })
        .collect()
}

fn vm_scalar_to_plain_string(value: &VMValue) -> String {
    match value {
        VMValue::Str(s) => s.clone(),
        VMValue::Int(n) => n.to_string(),
        VMValue::Float(f) => f.to_string(),
        VMValue::Bool(b) => b.to_string(),
        VMValue::Unit => String::new(),
        VMValue::Variant(tag, None) if tag == "none" => String::new(),
        VMValue::Variant(tag, Some(payload)) if tag == "some" => vm_scalar_to_plain_string(payload),
        other => vmvalue_repr(other),
    }
}

fn schema_adapt_rows(
    rows: Vec<HashMap<String, VMValue>>,
    type_name: &str,
    type_metas: &HashMap<String, TypeMeta>,
) -> VMValue {
    let Some(meta) = type_metas.get(type_name) else {
        return err_vm(schema_error_vm(
            "",
            format!("known type {}", type_name),
            type_name,
        ));
    };
    let positional = meta.fields.iter().any(|field| field.col_index.is_some());
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut record = HashMap::new();
        for field in &meta.fields {
            let lookup_key = if positional {
                field
                    .col_index
                    .map(|idx| idx.to_string())
                    .unwrap_or_else(|| field.name.clone())
            } else {
                field.name.clone()
            };
            let raw = match row.get(&lookup_key) {
                Some(VMValue::Str(s)) => s.clone(),
                Some(other) => vmvalue_repr(other),
                None => return err_vm(schema_error_vm(&field.name, &lookup_key, "missing")),
            };
            match parse_schema_value(&raw, &field.ty, &field.name) {
                Ok(value) => {
                    record.insert(field.name.clone(), value);
                }
                Err(err) => return err_vm(err),
            }
        }
        out.push(VMValue::Record(record));
    }
    ok_vm(VMValue::List(out))
}

fn schema_to_json_value(
    value: &VMValue,
    type_name: &str,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<SerdeJsonValue, String> {
    let VMValue::Record(record) = value else {
        return Err(format!("Schema expected record for `{}`", type_name));
    };
    let mut out = serde_json::Map::new();
    let ordered_fields: Vec<(String, String)> = if let Some(meta) = type_metas.get(type_name) {
        meta.fields
            .iter()
            .map(|field| (field.name.clone(), field.ty.clone()))
            .collect()
    } else {
        let mut keys: Vec<String> = record.keys().cloned().collect();
        keys.sort();
        keys.into_iter().map(|key| (key, "_".into())).collect()
    };
    for (field_name, field_ty) in ordered_fields {
        let value = record.get(&field_name).ok_or_else(|| {
            format!(
                "record missing field `{}` for schema `{}`",
                field_name, type_name
            )
        })?;
        let json = match value {
            VMValue::Int(v) => SerdeJsonValue::Number((*v).into()),
            VMValue::Float(v) => serde_json::Number::from_f64(*v)
                .map(SerdeJsonValue::Number)
                .ok_or_else(|| format!("invalid float in field `{}`", field_name))?,
            VMValue::Bool(v) => SerdeJsonValue::Bool(*v),
            VMValue::Str(v) => SerdeJsonValue::String(v.clone()),
            VMValue::Variant(tag, None) if tag == "none" => SerdeJsonValue::Null,
            VMValue::Variant(tag, Some(payload)) if tag == "some" => match payload.as_ref() {
                VMValue::Int(v) => SerdeJsonValue::Number((*v).into()),
                VMValue::Float(v) => serde_json::Number::from_f64(*v)
                    .map(SerdeJsonValue::Number)
                    .ok_or_else(|| format!("invalid float in field `{}`", field_name))?,
                VMValue::Bool(v) => SerdeJsonValue::Bool(*v),
                VMValue::Str(v) => SerdeJsonValue::String(v.clone()),
                other => {
                    return Err(format!(
                        "unsupported option payload {} for field `{}`",
                        vmvalue_type_name(other),
                        field_name
                    ));
                }
            },
            other => {
                return Err(format!(
                    "unsupported field value {} for field `{}` ({})",
                    vmvalue_type_name(other),
                    field_name,
                    field_ty
                ));
            }
        };
        out.insert(field_name, json);
    }
    Ok(SerdeJsonValue::Object(out))
}

fn vmvalue_to_sql(value: &VMValue) -> rusqlite::types::Value {
    match value {
        VMValue::Int(n) => rusqlite::types::Value::Integer(*n),
        VMValue::Float(f) => rusqlite::types::Value::Real(*f),
        VMValue::Str(s) => rusqlite::types::Value::Text(s.clone()),
        VMValue::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
        VMValue::Unit => rusqlite::types::Value::Null,
        other => rusqlite::types::Value::Text(vmvalue_repr(other)),
    }
}

fn sqlite_value_to_string(value: rusqlite::types::Value) -> String {
    match value {
        rusqlite::types::Value::Null => "null".to_string(),
        rusqlite::types::Value::Integer(n) => n.to_string(),
        rusqlite::types::Value::Real(f) => f.to_string(),
        rusqlite::types::Value::Text(s) => s,
        rusqlite::types::Value::Blob(bytes) => format!("<blob:{} bytes>", bytes.len()),
    }
}

fn with_db_path<T, F>(db_path: Option<&str>, f: F) -> Result<T, String>
where
    F: FnOnce(&Connection) -> Result<T, String>,
{
    let path =
        db_path.ok_or_else(|| "Db not initialized 窶・run with --db <path> flag".to_string())?;
    let mut dbs = SHARED_DBS
        .lock()
        .map_err(|_| "Db mutex poisoned".to_string())?;
    let entry_idx = if let Some(idx) = dbs.iter().position(|(p, _)| p == path) {
        idx
    } else {
        let conn = if path == ":memory:" {
            Connection::open_in_memory().map_err(|e| format!("Db open failed: {}", e))?
        } else {
            Connection::open(path).map_err(|e| format!("Db open failed for `{}`: {}", path, e))?
        };
        dbs.push((path.to_string(), conn));
        dbs.len() - 1
    };
    let (_, conn) = &dbs[entry_idx];
    f(conn)
}

/// Build a `DbError { code, message }` record.
fn db_error_vm(code: &str, message: &str) -> VMValue {
    let mut m = HashMap::new();
    m.insert("code".to_string(), VMValue::Str(code.to_string()));
    m.insert("message".to_string(), VMValue::Str(message.to_string()));
    VMValue::Record(m)
}

fn http_response_vm(status: i64, body: String, content_type: String) -> VMValue {
    let mut m = HashMap::new();
    m.insert("status".to_string(), VMValue::Int(status));
    m.insert("body".to_string(), VMValue::Str(body));
    m.insert("content_type".to_string(), VMValue::Str(content_type));
    VMValue::Record(m)
}

fn http_error_vm(code: i64, message: String, status: i64) -> VMValue {
    let mut m = HashMap::new();
    m.insert("code".to_string(), VMValue::Int(code));
    m.insert("message".to_string(), VMValue::Str(message));
    m.insert("status".to_string(), VMValue::Int(status));
    VMValue::Record(m)
}

fn parquet_error_vm(message: impl Into<String>) -> VMValue {
    let mut m = HashMap::new();
    m.insert("message".to_string(), VMValue::Str(message.into()));
    VMValue::Record(m)
}

fn rpc_error_vm(code: i64, message: impl Into<String>) -> VMValue {
    let mut m = HashMap::new();
    m.insert("code".to_string(), VMValue::Int(code));
    m.insert("message".to_string(), VMValue::Str(message.into()));
    VMValue::Record(m)
}

fn encode_grpc_frame(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + payload.len());
    out.push(0u8);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

fn decode_grpc_frame(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 5 {
        return Err(format!("gRPC frame too short: {} bytes", data.len()));
    }
    let len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
    if data.len() < 5 + len {
        return Err(format!(
            "gRPC frame body truncated: expected {} bytes, got {}",
            len,
            data.len().saturating_sub(5)
        ));
    }
    Ok(data[5..5 + len].to_vec())
}

fn decode_all_grpc_frames(data: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let mut frames = Vec::new();
    let mut offset = 0usize;
    while offset < data.len() {
        if data.len() - offset < 5 {
            return Err(format!(
                "gRPC trailing bytes too short for frame header: {}",
                data.len() - offset
            ));
        }
        let len = u32::from_be_bytes([
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
        ]) as usize;
        let end = offset + 5 + len;
        if end > data.len() {
            return Err(format!(
                "gRPC frame body truncated: expected {} bytes, got {}",
                len,
                data.len().saturating_sub(offset + 5)
            ));
        }
        frames.push(data[offset + 5..end].to_vec());
        offset = end;
    }
    Ok(frames)
}

#[allow(dead_code)]
fn pascal_to_snake(name: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if idx > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn proto_wire_type_for_field(ty: &str) -> u8 {
    match option_inner_type_name(ty) {
        "Int" | "Bool" => 0,
        "Float" => 1,
        _ => 2,
    }
}

fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push(((value as u8) & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn decode_varint(bytes: &[u8], pos: &mut usize) -> Result<u64, String> {
    let mut shift = 0u32;
    let mut value = 0u64;
    while *pos < bytes.len() {
        let byte = bytes[*pos];
        *pos += 1;
        value |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift > 63 {
            return Err("protobuf varint too large".to_string());
        }
    }
    Err("unexpected EOF while reading protobuf varint".to_string())
}

fn skip_proto_value(bytes: &[u8], pos: &mut usize, wire_type: u8) -> Result<(), String> {
    match wire_type {
        0 => {
            let _ = decode_varint(bytes, pos)?;
            Ok(())
        }
        1 => {
            if *pos + 8 > bytes.len() {
                return Err("unexpected EOF while reading 64-bit field".to_string());
            }
            *pos += 8;
            Ok(())
        }
        2 => {
            let len = decode_varint(bytes, pos)? as usize;
            if *pos + len > bytes.len() {
                return Err("unexpected EOF while reading length-delimited field".to_string());
            }
            *pos += len;
            Ok(())
        }
        other => Err(format!("unsupported protobuf wire type {}", other)),
    }
}

fn map_to_proto_bytes(
    type_name: &str,
    row: &HashMap<String, String>,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<Vec<u8>, String> {
    let meta = type_metas
        .get(type_name)
        .ok_or_else(|| format!("Grpc.encode_raw: unknown type `{}`", type_name))?;
    let mut out = Vec::new();
    for (idx, field) in meta.fields.iter().enumerate() {
        let Some(raw) = row.get(&field.name) else {
            continue;
        };
        if raw.is_empty() && is_option_type_name(&field.ty) {
            continue;
        }
        let field_no = (idx + 1) as u64;
        let wire_type = proto_wire_type_for_field(&field.ty) as u64;
        encode_varint((field_no << 3) | wire_type, &mut out);
        match option_inner_type_name(&field.ty) {
            "Int" => {
                let value = raw.parse::<i64>().map_err(|e| {
                    format!(
                        "Grpc.encode_raw invalid Int field `{}` value `{}`: {}",
                        field.name, raw, e
                    )
                })?;
                encode_varint(value as u64, &mut out);
            }
            "Bool" => {
                let value = parse_bool_like(raw).ok_or_else(|| {
                    format!(
                        "Grpc.encode_raw invalid Bool field `{}` value `{}`",
                        field.name, raw
                    )
                })?;
                encode_varint(if value { 1 } else { 0 }, &mut out);
            }
            "Float" => {
                let value = raw.parse::<f64>().map_err(|e| {
                    format!(
                        "Grpc.encode_raw invalid Float field `{}` value `{}`: {}",
                        field.name, raw, e
                    )
                })?;
                out.extend_from_slice(&value.to_le_bytes());
            }
            _ => {
                encode_varint(raw.len() as u64, &mut out);
                out.extend_from_slice(raw.as_bytes());
            }
        }
    }
    Ok(out)
}

fn string_map_to_proto_bytes(row: &HashMap<String, String>) -> Vec<u8> {
    let mut fields: Vec<(&String, &String)> = row.iter().collect();
    fields.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = Vec::new();
    for (idx, (_key, value)) in fields.iter().enumerate() {
        let field_no = (idx + 1) as u64;
        let tag = (field_no << 3) | 2u64;
        encode_varint(tag, &mut out);
        encode_varint(value.len() as u64, &mut out);
        out.extend_from_slice(value.as_bytes());
    }
    out
}

fn proto_bytes_to_map(
    type_name: &str,
    bytes: &[u8],
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<HashMap<String, String>, String> {
    let meta = type_metas
        .get(type_name)
        .ok_or_else(|| format!("Grpc.decode_raw: unknown type `{}`", type_name))?;
    let mut out = HashMap::new();
    let mut pos = 0usize;
    while pos < bytes.len() {
        let key = decode_varint(bytes, &mut pos)?;
        let field_no = (key >> 3) as usize;
        let wire_type = (key & 0x07) as u8;
        let Some(field) = meta.fields.get(field_no.saturating_sub(1)) else {
            skip_proto_value(bytes, &mut pos, wire_type)?;
            continue;
        };
        let value = match (option_inner_type_name(&field.ty), wire_type) {
            ("Int", 0) => decode_varint(bytes, &mut pos)?.to_string(),
            ("Bool", 0) => {
                if decode_varint(bytes, &mut pos)? == 0 {
                    "false".to_string()
                } else {
                    "true".to_string()
                }
            }
            ("Float", 1) => {
                if pos + 8 > bytes.len() {
                    return Err("unexpected EOF while reading double".to_string());
                }
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[pos..pos + 8]);
                pos += 8;
                f64::from_le_bytes(buf).to_string()
            }
            (_, 2) => {
                let len = decode_varint(bytes, &mut pos)? as usize;
                if pos + len > bytes.len() {
                    return Err("unexpected EOF while reading string field".to_string());
                }
                let value = String::from_utf8(bytes[pos..pos + len].to_vec())
                    .map_err(|e| format!("Grpc.decode_raw invalid UTF-8: {}", e))?;
                pos += len;
                value
            }
            _ => {
                skip_proto_value(bytes, &mut pos, wire_type)?;
                continue;
            }
        };
        out.insert(field.name.clone(), value);
    }
    Ok(out)
}

fn proto_bytes_to_string_map(bytes: &[u8]) -> Result<HashMap<String, String>, String> {
    let mut out = HashMap::new();
    let mut pos = 0usize;
    while pos < bytes.len() {
        let key = decode_varint(bytes, &mut pos)?;
        let field_no = (key >> 3) as usize;
        let wire_type = (key & 0x07) as u8;
        match wire_type {
            0 => {
                let value = decode_varint(bytes, &mut pos)?.to_string();
                out.insert(format!("field{}", field_no), value);
            }
            1 => {
                if pos + 8 > bytes.len() {
                    return Err("unexpected EOF while reading double".to_string());
                }
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[pos..pos + 8]);
                pos += 8;
                out.insert(
                    format!("field{}", field_no),
                    f64::from_le_bytes(buf).to_string(),
                );
            }
            2 => {
                let len = decode_varint(bytes, &mut pos)? as usize;
                if pos + len > bytes.len() {
                    return Err("unexpected EOF while reading string field".to_string());
                }
                let value = String::from_utf8(bytes[pos..pos + len].to_vec())
                    .map_err(|e| format!("Grpc raw response invalid UTF-8: {}", e))?;
                pos += len;
                out.insert(format!("field{}", field_no), value);
            }
            other => {
                skip_proto_value(bytes, &mut pos, other)?;
            }
        }
    }
    Ok(out)
}

/// Type alias for messages sent from the h2 server thread to the VM dispatch loop.
/// `(handler_fn_name, proto_bytes, response_sender)`
type GrpcRequestMsg = (
    String,
    Vec<u8>,
    std::sync::mpsc::SyncSender<Result<Vec<u8>, String>>,
);

/// Spawn a background tokio thread running an h2/gRPC server on `port`.
/// Each incoming request is forwarded to `req_tx`; the VM loop replies via the
/// per-request `SyncSender` embedded in the message.
fn grpc_serve_impl(
    port: i64,
    req_tx: std::sync::mpsc::Sender<GrpcRequestMsg>,
) -> Result<(), String> {
    let port_u16 = u16::try_from(port).map_err(|_| format!("invalid gRPC port {}", port))?;
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime build failed");
        rt.block_on(async move {
            let listener =
                tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port_u16))
                    .await
                    .expect("gRPC bind failed");
            eprintln!("Listening on 0.0.0.0:{port_u16} (gRPC / HTTP2)");
            loop {
                let (socket, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let req_tx = req_tx.clone();
                tokio::spawn(async move {
                    let mut conn = match h2::server::handshake(socket).await {
                        Ok(c) => c,
                        Err(_) => return,
                    };
                    while let Some(result) = conn.accept().await {
                        let (request, respond) = match result {
                            Ok(r) => r,
                            Err(_) => return,
                        };
                        let req_tx = req_tx.clone();
                        tokio::spawn(async move {
                            grpc_handle_h2_request(request, respond, req_tx).await;
                        });
                    }
                });
            }
        });
    });
    Ok(())
}

/// Async handler for a single h2 gRPC request: reads body, dispatches to VM
/// via channel, sends response back over h2.
async fn grpc_handle_h2_request(
    request: http::Request<h2::RecvStream>,
    mut respond: h2::server::SendResponse<Bytes>,
    req_tx: std::sync::mpsc::Sender<GrpcRequestMsg>,
) {
    // Derive handler name: "/ServiceName/MethodName" -> "handle_method_name"
    let path = request.uri().path().to_string();
    let method_part = path.rsplit('/').next().unwrap_or("unknown").to_string();
    let handler_name = format!("handle_{}", pascal_to_snake(&method_part));

    // Read all DATA frames from the request body
    let mut body = request.into_body();
    let mut body_bytes: Vec<u8> = Vec::new();
    while let Some(chunk) = body.data().await {
        match chunk {
            Ok(data) => {
                let n = data.len();
                body_bytes.extend_from_slice(&data);
                let _ = body.flow_control().release_capacity(n);
            }
            Err(_) => return,
        }
    }

    // Strip gRPC framing (5-byte prefix); fall back to raw bytes if malformed
    let proto_bytes = decode_grpc_frame(&body_bytes).unwrap_or(body_bytes);

    // Send request to VM dispatch loop and wait for response
    let (res_tx, res_rx) =
        std::sync::mpsc::sync_channel::<Result<Vec<u8>, String>>(1);
    if req_tx.send((handler_name, proto_bytes, res_tx)).is_err() {
        return;
    }
    let resp_data =
        match tokio::task::spawn_blocking(move || res_rx.recv()).await {
            Ok(Ok(Ok(b))) => b,
            _ => return,
        };

    // Send HTTP/2 response
    let http_resp = http::Response::builder()
        .status(200)
        .header("content-type", "application/grpc")
        .body(())
        .unwrap();
    let mut send = match respond.send_response(http_resp, false) {
        Ok(s) => s,
        Err(_) => return,
    };
    let _ = send.send_data(Bytes::from(resp_data), false);
    let mut trailers = http::HeaderMap::new();
    trailers.insert(
        http::header::HeaderName::from_static("grpc-status"),
        http::HeaderValue::from_static("0"),
    );
    let _ = send.send_trailers(trailers);
}

/// Convert a VM function result into proto bytes for a gRPC response.
fn grpc_vm_value_to_proto_bytes(result: Result<VMValue, String>) -> Result<Vec<u8>, String> {
    match result {
        Ok(VMValue::Record(map)) => {
            let str_map = schema_record_to_string_map(&map);
            Ok(string_map_to_proto_bytes(&str_map))
        }
        Ok(other) => Err(format!(
            "gRPC handler must return Map<String,String>, got {}",
            vmvalue_type_name(&other)
        )),
        Err(e) => Err(e),
    }
}

/// Extract the TCP address from a gRPC host string.
/// "http://host:port" -> "host:port", "host:port" -> "host:port"
fn grpc_tcp_addr(host: &str) -> String {
    if let Some(rest) = host.strip_prefix("http://") {
        rest.trim_end_matches('/').to_string()
    } else if let Some(rest) = host.strip_prefix("https://") {
        rest.trim_end_matches('/').to_string()
    } else {
        host.to_string()
    }
}

/// Build a full URI for a gRPC method call.
fn grpc_method_uri(host: &str, method: &str) -> String {
    let base = if host.starts_with("http://") || host.starts_with("https://") {
        host.trim_end_matches('/').to_string()
    } else {
        format!("http://{}", host)
    };
    format!("{}/{}", base, method.trim_start_matches('/'))
}

fn is_option_type_name(ty: &str) -> bool {
    ty.starts_with("Option<") && ty.ends_with('>')
}

fn option_inner_type_name(ty: &str) -> &str {
    if is_option_type_name(ty) {
        &ty[7..ty.len() - 1]
    } else {
        ty
    }
}

fn arrow_type_for_meta(ty: &str) -> DataType {
    match option_inner_type_name(ty) {
        "Int" => DataType::Int64,
        "Float" => DataType::Float64,
        "Bool" => DataType::Boolean,
        _ => DataType::Utf8,
    }
}

fn parquet_write_rows(
    path: &str,
    type_name: &str,
    rows: Vec<HashMap<String, VMValue>>,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<(), String> {
    let meta = type_metas
        .get(type_name)
        .ok_or_else(|| format!("Parquet.write_raw: unknown type `{}`", type_name))?;
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Parquet.write_raw failed to create directory: {}", e))?;
        }
    }

    let fields: Vec<ArrowField> = meta
        .fields
        .iter()
        .map(|field| {
            ArrowField::new(
                &field.name,
                arrow_type_for_meta(&field.ty),
                is_option_type_name(&field.ty),
            )
        })
        .collect();
    let schema = std::sync::Arc::new(ArrowSchema::new(fields));
    let mut arrays: Vec<ArrayRef> = Vec::with_capacity(meta.fields.len());

    for field in &meta.fields {
        let base_ty = option_inner_type_name(&field.ty);
        match arrow_type_for_meta(&field.ty) {
            DataType::Int64 => {
                let mut builder = Int64Builder::new();
                for row in &rows {
                    let raw = row
                        .get(&field.name)
                        .map(vm_scalar_to_plain_string)
                        .unwrap_or_default();
                    if raw.is_empty() && is_option_type_name(&field.ty) {
                        builder.append_null();
                    } else {
                        let value = raw.parse::<i64>().map_err(|e| {
                            format!(
                                "Parquet.write_raw invalid {} field `{}` value `{}`: {}",
                                base_ty, field.name, raw, e
                            )
                        })?;
                        builder.append_value(value);
                    }
                }
                arrays.push(std::sync::Arc::new(builder.finish()));
            }
            DataType::Float64 => {
                let mut builder = Float64Builder::new();
                for row in &rows {
                    let raw = row
                        .get(&field.name)
                        .map(vm_scalar_to_plain_string)
                        .unwrap_or_default();
                    if raw.is_empty() && is_option_type_name(&field.ty) {
                        builder.append_null();
                    } else {
                        let value = raw.parse::<f64>().map_err(|e| {
                            format!(
                                "Parquet.write_raw invalid {} field `{}` value `{}`: {}",
                                base_ty, field.name, raw, e
                            )
                        })?;
                        builder.append_value(value);
                    }
                }
                arrays.push(std::sync::Arc::new(builder.finish()));
            }
            DataType::Boolean => {
                let mut builder = BooleanBuilder::new();
                for row in &rows {
                    let raw = row
                        .get(&field.name)
                        .map(vm_scalar_to_plain_string)
                        .unwrap_or_default();
                    if raw.is_empty() && is_option_type_name(&field.ty) {
                        builder.append_null();
                    } else {
                        let value = match raw.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(format!(
                                    "Parquet.write_raw invalid Bool field `{}` value `{}`",
                                    field.name, raw
                                ));
                            }
                        };
                        builder.append_value(value);
                    }
                }
                arrays.push(std::sync::Arc::new(builder.finish()));
            }
            DataType::Utf8 => {
                let mut builder = StringBuilder::new();
                for row in &rows {
                    let raw = row
                        .get(&field.name)
                        .map(vm_scalar_to_plain_string)
                        .unwrap_or_default();
                    if raw.is_empty() && is_option_type_name(&field.ty) {
                        builder.append_null();
                    } else {
                        builder.append_value(raw);
                    }
                }
                arrays.push(std::sync::Arc::new(builder.finish()));
            }
            other => {
                return Err(format!(
                    "Parquet.write_raw unsupported Arrow type for `{}`: {:?}",
                    field.name, other
                ));
            }
        }
    }

    let batch = RecordBatch::try_new(schema.clone(), arrays)
        .map_err(|e| format!("Parquet.write_raw record batch failed: {}", e))?;
    let file = File::create(path).map_err(|e| format!("Parquet.write_raw open failed: {}", e))?;
    let mut writer = ArrowWriter::try_new(file, schema, None)
        .map_err(|e| format!("Parquet.write_raw writer failed: {}", e))?;
    writer
        .write(&batch)
        .map_err(|e| format!("Parquet.write_raw write failed: {}", e))?;
    writer
        .close()
        .map_err(|e| format!("Parquet.write_raw close failed: {}", e))?;
    Ok(())
}

fn parquet_read_rows(path: &str) -> Result<Vec<HashMap<String, VMValue>>, String> {
    let file = File::open(path).map_err(|e| format!("Parquet.read_raw open failed: {}", e))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| format!("Parquet.read_raw reader failed: {}", e))?;
    let reader = builder
        .build()
        .map_err(|e| format!("Parquet.read_raw build failed: {}", e))?;
    let mut rows = Vec::new();
    for batch_result in reader {
        let batch = batch_result.map_err(|e| format!("Parquet.read_raw batch failed: {}", e))?;
        let schema = batch.schema();
        for row_idx in 0..batch.num_rows() {
            let mut row = HashMap::new();
            for (col_idx, field) in schema.fields().iter().enumerate() {
                let column = batch.column(col_idx);
                let value = parquet_cell_to_string(column.as_ref(), row_idx)?;
                row.insert(field.name().clone(), VMValue::Str(value));
            }
            rows.push(row);
        }
    }
    Ok(rows)
}

fn parquet_cell_to_string(array: &dyn Array, row_idx: usize) -> Result<String, String> {
    if array.is_null(row_idx) {
        return Ok(String::new());
    }
    if let Some(arr) = array.as_any().downcast_ref::<StringArray>() {
        return Ok(arr.value(row_idx).to_string());
    }
    if let Some(arr) = array.as_any().downcast_ref::<Int64Array>() {
        return Ok(arr.value(row_idx).to_string());
    }
    if let Some(arr) = array.as_any().downcast_ref::<Float64Array>() {
        return Ok(arr.value(row_idx).to_string());
    }
    if let Some(arr) = array.as_any().downcast_ref::<BooleanArray>() {
        return Ok(arr.value(row_idx).to_string());
    }
    Err(format!(
        "Parquet.read_raw unsupported column type: {:?}",
        array.data_type()
    ))
}

/// Execute a raw SELECT and return rows as `List<Map<String,String>>`.
fn sqlite_query_raw(conn: &rusqlite::Connection, sql: &str) -> Result<Vec<VMValue>, String> {
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("E0602: db query failed: {}", e))?;
    let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let mut rows_out = Vec::new();
    let mut rows = stmt
        .query([])
        .map_err(|e| format!("E0602: db query failed: {}", e))?;
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("E0602: db query failed: {}", e))?
    {
        let mut map = HashMap::new();
        for (i, name) in col_names.iter().enumerate() {
            let val: rusqlite::types::Value = row
                .get(i)
                .map_err(|e| format!("E0602: db query failed: {}", e))?;
            map.insert(name.clone(), VMValue::Str(sqlite_value_to_string(val)));
        }
        rows_out.push(VMValue::Record(map));
    }
    Ok(rows_out)
}

/// Execute a parameterised SELECT and return rows as `List<Map<String,String>>`.
fn sqlite_query_raw_params(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[String],
) -> Result<Vec<VMValue>, String> {
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("E0602: db query failed: {}", e))?;
    let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let param_refs: Vec<&dyn rusqlite::ToSql> =
        params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    let mut rows_out = Vec::new();
    let mut rows = stmt
        .query(param_refs.as_slice())
        .map_err(|e| format!("E0602: db query failed: {}", e))?;
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("E0602: db query failed: {}", e))?
    {
        let mut map = HashMap::new();
        for (i, name) in col_names.iter().enumerate() {
            let val: rusqlite::types::Value = row
                .get(i)
                .map_err(|e| format!("E0602: db query failed: {}", e))?;
            map.insert(name.clone(), VMValue::Str(sqlite_value_to_string(val)));
        }
        rows_out.push(VMValue::Record(map));
    }
    Ok(rows_out)
}

// ── Gen helpers (v3.5.0) ─────────────────────────────────────────────────────

fn seeded_rand_int(lo: i64, hi: i64) -> i64 {
    use rand::Rng;
    SEEDED_RNG.with(|r| {
        let mut borrowed = r.borrow_mut();
        if let Some(rng) = borrowed.as_mut() {
            rng.gen_range(lo..=hi)
        } else {
            rand::thread_rng().gen_range(lo..=hi)
        }
    })
}

fn seeded_rand_float() -> f64 {
    use rand::Rng;
    SEEDED_RNG.with(|r| {
        let mut borrowed = r.borrow_mut();
        if let Some(rng) = borrowed.as_mut() {
            rng.r#gen::<f64>()
        } else {
            rand::thread_rng().r#gen::<f64>()
        }
    })
}

fn random_alphanumeric_string(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    (0..len)
        .map(|_| {
            let idx = seeded_rand_int(0, (CHARS.len() - 1) as i64) as usize;
            CHARS[idx] as char
        })
        .collect()
}

fn gen_value_for_type(ty: &str) -> String {
    if ty.starts_with("Option<") && ty.ends_with('>') {
        // 50% chance of empty (None), 50% of inner type
        if seeded_rand_int(0, 1) == 0 {
            String::new()
        } else {
            let inner = &ty[7..ty.len() - 1];
            gen_value_for_type(inner)
        }
    } else {
        match ty {
            "Int" => seeded_rand_int(-1000, 1000).to_string(),
            "Float" => format!("{:.6}", seeded_rand_float()),
            "Bool" => if seeded_rand_int(0, 1) == 0 {
                "false"
            } else {
                "true"
            }
            .to_string(),
            _ => random_alphanumeric_string(8),
        }
    }
}

fn gen_corrupt_value(ty: &str) -> String {
    // Returns a value that is intentionally invalid for the given type
    if ty.starts_with("Option<") {
        String::new() // Options become empty (None) when corrupted
    } else {
        match ty {
            "Int" | "Float" => "NaN".to_string(),
            "Bool" => "maybe".to_string(),
            _ => String::new(),
        }
    }
}

fn gen_one_row(type_name: &str, type_metas: &HashMap<String, TypeMeta>) -> Result<VMValue, String> {
    let meta = type_metas
        .get(type_name)
        .ok_or_else(|| format!("Gen.one_raw: unknown type '{type_name}'"))?;
    let mut map = HashMap::new();
    for field in &meta.fields {
        let val = gen_value_for_type(&field.ty);
        map.insert(field.name.clone(), VMValue::Str(val));
    }
    Ok(VMValue::Record(map))
}

fn is_valid_for_type(val: &str, ty: &str) -> bool {
    if ty.starts_with("Option<") && ty.ends_with('>') {
        if val.is_empty() {
            return true; // None is always valid for Option
        }
        let inner = &ty[7..ty.len() - 1];
        return is_valid_for_type(val, inner);
    }
    match ty {
        "Int" => val.parse::<i64>().is_ok(),
        "Float" => val.parse::<f64>().is_ok(),
        "Bool" => val == "true" || val == "false",
        _ => true, // String and unknown types are always valid
    }
}

fn vm_call_builtin(
    name: &str,
    args: Vec<VMValue>,
    emit_log: &mut Vec<VMValue>,
    db_path: Option<&str>,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<VMValue, String> {
    match name {
        "Math.pi" => {
            if !args.is_empty() {
                return Err("Math.pi requires 0 arguments".to_string());
            }
            Ok(VMValue::Float(std::f64::consts::PI))
        }
        "Math.e" => {
            if !args.is_empty() {
                return Err("Math.e requires 0 arguments".to_string());
            }
            Ok(VMValue::Float(std::f64::consts::E))
        }
        "Math.abs" => match args.as_slice() {
            [VMValue::Int(n)] => Ok(VMValue::Int(n.abs())),
            [_] => Err("Math.abs requires an Int argument".to_string()),
            _ => Err("Math.abs requires 1 argument".to_string()),
        },
        "Math.abs_float" => match args.as_slice() {
            [VMValue::Float(f)] => Ok(VMValue::Float(f.abs())),
            [_] => Err("Math.abs_float requires a Float argument".to_string()),
            _ => Err("Math.abs_float requires 1 argument".to_string()),
        },
        "Math.min" => match args.as_slice() {
            [VMValue::Int(a), VMValue::Int(b)] => Ok(VMValue::Int((*a).min(*b))),
            [_, _] => Err("Math.min requires (Int, Int)".to_string()),
            _ => Err("Math.min requires 2 arguments".to_string()),
        },
        "Math.max" => match args.as_slice() {
            [VMValue::Int(a), VMValue::Int(b)] => Ok(VMValue::Int((*a).max(*b))),
            [_, _] => Err("Math.max requires (Int, Int)".to_string()),
            _ => Err("Math.max requires 2 arguments".to_string()),
        },
        "Math.min_float" => match args.as_slice() {
            [VMValue::Float(a), VMValue::Float(b)] => Ok(VMValue::Float(a.min(*b))),
            [_, _] => Err("Math.min_float requires (Float, Float)".to_string()),
            _ => Err("Math.min_float requires 2 arguments".to_string()),
        },
        "Math.max_float" => match args.as_slice() {
            [VMValue::Float(a), VMValue::Float(b)] => Ok(VMValue::Float(a.max(*b))),
            [_, _] => Err("Math.max_float requires (Float, Float)".to_string()),
            _ => Err("Math.max_float requires 2 arguments".to_string()),
        },
        "Math.clamp" => match args.as_slice() {
            [VMValue::Int(v), VMValue::Int(lo), VMValue::Int(hi)] => {
                Ok(VMValue::Int((*v).max(*lo).min(*hi)))
            }
            [_, _, _] => Err("Math.clamp requires (Int, Int, Int)".to_string()),
            _ => Err("Math.clamp requires 3 arguments".to_string()),
        },
        "Math.pow" => match args.as_slice() {
            [VMValue::Int(base), VMValue::Int(exp)] if *exp >= 0 => {
                Ok(VMValue::Int(base.pow(*exp as u32)))
            }
            [VMValue::Int(_), VMValue::Int(_)] => {
                Err("Math.pow requires a non-negative exponent".to_string())
            }
            [_, _] => Err("Math.pow requires (Int, Int)".to_string()),
            _ => Err("Math.pow requires 2 arguments".to_string()),
        },
        "Math.pow_float" => match args.as_slice() {
            [VMValue::Float(base), VMValue::Float(exp)] => Ok(VMValue::Float(base.powf(*exp))),
            [_, _] => Err("Math.pow_float requires (Float, Float)".to_string()),
            _ => Err("Math.pow_float requires 2 arguments".to_string()),
        },
        "Math.sqrt" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Float(v.sqrt())),
            [_] => Err("Math.sqrt requires a Float argument".to_string()),
            _ => Err("Math.sqrt requires 1 argument".to_string()),
        },
        "Math.floor" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Int(v.floor() as i64)),
            [_] => Err("Math.floor requires a Float argument".to_string()),
            _ => Err("Math.floor requires 1 argument".to_string()),
        },
        "Math.ceil" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Int(v.ceil() as i64)),
            [_] => Err("Math.ceil requires a Float argument".to_string()),
            _ => Err("Math.ceil requires 1 argument".to_string()),
        },
        "Math.round" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Int(v.round() as i64)),
            [_] => Err("Math.round requires a Float argument".to_string()),
            _ => Err("Math.round requires 1 argument".to_string()),
        },
        "IO.println" => {
            let s = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(v) => vmvalue_repr(&v),
                None => return Err("IO.println requires 1 argument".to_string()),
            };
            IO_CAPTURE.with(|c| {
                if let Some(buf) = c.borrow_mut().as_mut() {
                    buf.push_str(&s);
                    buf.push('\n');
                } else if !is_io_suppressed() {
                    println!("{}", s);
                }
            });
            Ok(VMValue::Unit)
        }
        "IO.println_int" => match args.as_slice() {
            [VMValue::Int(n)] => {
                let n = *n;
                IO_CAPTURE.with(|c| {
                    if let Some(buf) = c.borrow_mut().as_mut() {
                        buf.push_str(&n.to_string());
                        buf.push('\n');
                    } else if !is_io_suppressed() {
                        println!("{}", n);
                    }
                });
                Ok(VMValue::Unit)
            }
            [_] => Err("IO.println_int requires an Int argument".to_string()),
            _ => Err("IO.println_int requires 1 argument".to_string()),
        },
        "IO.println_float" => match args.as_slice() {
            [VMValue::Float(n)] => {
                let n = *n;
                IO_CAPTURE.with(|c| {
                    if let Some(buf) = c.borrow_mut().as_mut() {
                        buf.push_str(&n.to_string());
                        buf.push('\n');
                    } else if !is_io_suppressed() {
                        println!("{}", n);
                    }
                });
                Ok(VMValue::Unit)
            }
            [_] => Err("IO.println_float requires a Float argument".to_string()),
            _ => Err("IO.println_float requires 1 argument".to_string()),
        },
        "IO.println_bool" => match args.as_slice() {
            [VMValue::Bool(b)] => {
                let s = if *b { "true" } else { "false" };
                IO_CAPTURE.with(|c| {
                    if let Some(buf) = c.borrow_mut().as_mut() {
                        buf.push_str(s);
                        buf.push('\n');
                    } else if !is_io_suppressed() {
                        println!("{}", s);
                    }
                });
                Ok(VMValue::Unit)
            }
            [_] => Err("IO.println_bool requires a Bool argument".to_string()),
            _ => Err("IO.println_bool requires 1 argument".to_string()),
        },
        "IO.print" => {
            use std::io::Write;
            let s = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(v) => vmvalue_repr(&v),
                None => return Err("IO.print requires 1 argument".to_string()),
            };
            IO_CAPTURE.with(|c| {
                if let Some(buf) = c.borrow_mut().as_mut() {
                    buf.push_str(&s);
                } else if !is_io_suppressed() {
                    print!("{}", s);
                    std::io::stdout().flush().ok();
                }
            });
            Ok(VMValue::Unit)
        }
        "IO.read_line" => {
            if !args.is_empty() {
                return Err("IO.read_line requires 0 arguments".to_string());
            }
            if is_io_suppressed() {
                return Ok(VMValue::Str(String::new()));
            }
            use std::io::BufRead;
            let mut line = String::new();
            std::io::stdin()
                .lock()
                .read_line(&mut line)
                .map_err(|e| format!("IO.read_line failed: {e}"))?;
            if line.ends_with('\n') {
                line.pop();
            }
            if line.ends_with('\r') {
                line.pop();
            }
            Ok(VMValue::Str(line))
        }
        "IO.timestamp" => {
            if !args.is_empty() {
                return Err("IO.timestamp requires 0 arguments".to_string());
            }
            Ok(VMValue::Str(current_timestamp_string()))
        }
        "Debug.show" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Debug.show requires 1 argument".to_string())?;
            Ok(VMValue::Str(vmvalue_repr(&v)))
        }
        "assert" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "assert requires 1 argument".to_string())?;
            match v {
                VMValue::Bool(true) => Ok(VMValue::Unit),
                VMValue::Bool(false) => Err("assertion failed".to_string()),
                other => Err(format!(
                    "assert requires Bool, got {}",
                    vmvalue_type_name(&other)
                )),
            }
        }
        "assert_eq" => {
            let mut it = args.into_iter();
            let a = it
                .next()
                .ok_or_else(|| "assert_eq requires 2 arguments".to_string())?;
            let b = it
                .next()
                .ok_or_else(|| "assert_eq requires 2 arguments".to_string())?;
            if vmvalue_repr(&a) == vmvalue_repr(&b) {
                Ok(VMValue::Unit)
            } else {
                Err(format!(
                    "assert_eq failed: left={}, right={}",
                    vmvalue_repr(&a),
                    vmvalue_repr(&b)
                ))
            }
        }
        "assert_ne" => {
            let mut it = args.into_iter();
            let a = it
                .next()
                .ok_or_else(|| "assert_ne requires 2 arguments".to_string())?;
            let b = it
                .next()
                .ok_or_else(|| "assert_ne requires 2 arguments".to_string())?;
            if vmvalue_repr(&a) != vmvalue_repr(&b) {
                Ok(VMValue::Unit)
            } else {
                Err(format!(
                    "assert_ne failed: both equal to {}",
                    vmvalue_repr(&a)
                ))
            }
        }
        "Result.ok" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Result.ok requires 1 argument".to_string())?;
            Ok(VMValue::Variant("ok".to_string(), Some(Box::new(v))))
        }
        "Result.err" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Result.err requires 1 argument".to_string())?;
            Ok(VMValue::Variant("err".to_string(), Some(Box::new(v))))
        }
        "Option.some" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Option.some requires 1 argument".to_string())?;
            Ok(VMValue::Variant("some".to_string(), Some(Box::new(v))))
        }
        "Option.none" => Ok(VMValue::Variant("none".to_string(), None)),
        "Int.show.show" | "Float.show.show" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| format!("{} requires 1 argument", name))?;
            Ok(VMValue::Str(match v {
                VMValue::Int(n) => n.to_string(),
                VMValue::Float(f) => {
                    if f.fract() == 0.0 {
                        format!("{:.1}", f)
                    } else {
                        f.to_string()
                    }
                }
                other => return Err(format!("{} requires Int/Float, got {:?}", name, other)),
            }))
        }
        "Bool.show.show" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Bool.show.show requires 1 argument".to_string())?;
            Ok(VMValue::Str(match v {
                VMValue::Bool(b) => b.to_string(),
                other => return Err(format!("Bool.show.show requires Bool, got {:?}", other)),
            }))
        }
        "String.show.show" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.show.show requires 1 argument".to_string())?;
            Ok(VMValue::Str(match v {
                VMValue::Str(s) => format!("\"{}\"", s),
                other => return Err(format!("String.show.show requires String, got {:?}", other)),
            }))
        }
        "Int.ord.compare" => {
            let mut it = args.into_iter();
            let a = it
                .next()
                .ok_or_else(|| "Int.ord.compare requires 2 arguments".to_string())?;
            let b = it
                .next()
                .ok_or_else(|| "Int.ord.compare requires 2 arguments".to_string())?;
            match (a, b) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(match x.cmp(&y) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                })),
                _ => Err("Int.ord.compare requires two Int arguments".to_string()),
            }
        }
        "Int.eq.equals" => {
            let mut it = args.into_iter();
            let a = it
                .next()
                .ok_or_else(|| "Int.eq.equals requires 2 arguments".to_string())?;
            let b = it
                .next()
                .ok_or_else(|| "Int.eq.equals requires 2 arguments".to_string())?;
            match (a, b) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Bool(x == y)),
                _ => Err("Int.eq.equals requires two Int arguments".to_string()),
            }
        }
        "String.concat" => {
            let mut it = args.into_iter();
            let a = it
                .next()
                .ok_or_else(|| "String.concat requires 2 arguments".to_string())?;
            let b = it
                .next()
                .ok_or_else(|| "String.concat requires 2 arguments".to_string())?;
            match (a, b) {
                (VMValue::Str(x), VMValue::Str(y)) => Ok(VMValue::Str(x + &y)),
                _ => Err("String.concat requires two String arguments".to_string()),
            }
        }
        "String.length" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.length requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Int(s.len() as i64)),
                _ => Err("String.length requires a String argument".to_string()),
            }
        }
        "String.is_empty" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.is_empty requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Bool(s.is_empty())),
                _ => Err("String.is_empty requires a String argument".to_string()),
            }
        }
        "String.trim" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.trim requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.trim().to_string())),
                _ => Err("String.trim requires a String argument".to_string()),
            }
        }
        "String.upper" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.upper requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.to_uppercase())),
                _ => Err("String.upper requires a String argument".to_string()),
            }
        }
        "String.lower" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.lower requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.to_lowercase())),
                _ => Err("String.lower requires a String argument".to_string()),
            }
        }
        "String.split" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.split requires 2 arguments".to_string())?;
            let d = it
                .next()
                .ok_or_else(|| "String.split requires 2 arguments".to_string())?;
            match (s, d) {
                (VMValue::Str(s), VMValue::Str(delim)) => Ok(VMValue::List(
                    s.split(&*delim)
                        .map(|p| VMValue::Str(p.to_string()))
                        .collect(),
                )),
                _ => Err("String.split requires (String, String)".to_string()),
            }
        }
        "String.join" => {
            let mut it = args.into_iter();
            let xs = it
                .next()
                .ok_or_else(|| "String.join requires 2 arguments".to_string())?;
            let sep = it
                .next()
                .ok_or_else(|| "String.join requires 2 arguments".to_string())?;
            match (xs, sep) {
                (VMValue::List(values), VMValue::Str(sep)) => {
                    let mut parts = Vec::with_capacity(values.len());
                    for value in values {
                        match value {
                            VMValue::Str(s) => parts.push(s),
                            _ => {
                                return Err("String.join requires List<String> as first argument"
                                    .to_string());
                            }
                        }
                    }
                    Ok(VMValue::Str(parts.join(&sep)))
                }
                _ => Err("String.join requires (List<String>, String)".to_string()),
            }
        }
        "String.replace" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.replace requires 3 arguments".to_string())?;
            let from = it
                .next()
                .ok_or_else(|| "String.replace requires 3 arguments".to_string())?;
            let to = it
                .next()
                .ok_or_else(|| "String.replace requires 3 arguments".to_string())?;
            match (s, from, to) {
                (VMValue::Str(s), VMValue::Str(from), VMValue::Str(to)) => {
                    Ok(VMValue::Str(s.replace(&from, &to)))
                }
                _ => Err("String.replace requires (String, String, String)".to_string()),
            }
        }
        "String.index_of" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.index_of requires 2 arguments".to_string())?;
            let needle = it
                .next()
                .ok_or_else(|| "String.index_of requires 2 arguments".to_string())?;
            match (s, needle) {
                (VMValue::Str(s), VMValue::Str(needle)) => Ok(match s.find(&needle) {
                    Some(i) => {
                        VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(i as i64))))
                    }
                    None => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("String.index_of requires (String, String)".to_string()),
            }
        }
        "String.pad_left" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.pad_left requires 3 arguments".to_string())?;
            let width = it
                .next()
                .ok_or_else(|| "String.pad_left requires 3 arguments".to_string())?;
            let fill = it
                .next()
                .ok_or_else(|| "String.pad_left requires 3 arguments".to_string())?;
            match (s, width, fill) {
                (VMValue::Str(s), VMValue::Int(width), VMValue::Str(fill))
                    if width >= 0 && !fill.is_empty() =>
                {
                    let current = s.chars().count();
                    let width = width as usize;
                    if current >= width {
                        Ok(VMValue::Str(s))
                    } else {
                        let needed = width - current;
                        let prefix: String = fill.chars().cycle().take(needed).collect();
                        Ok(VMValue::Str(format!("{prefix}{s}")))
                    }
                }
                (VMValue::Str(_), VMValue::Int(_), VMValue::Str(fill)) if fill.is_empty() => {
                    Err("String.pad_left requires a non-empty fill string".to_string())
                }
                _ => Err("String.pad_left requires (String, Int, String)".to_string()),
            }
        }
        "String.pad_right" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.pad_right requires 3 arguments".to_string())?;
            let width = it
                .next()
                .ok_or_else(|| "String.pad_right requires 3 arguments".to_string())?;
            let fill = it
                .next()
                .ok_or_else(|| "String.pad_right requires 3 arguments".to_string())?;
            match (s, width, fill) {
                (VMValue::Str(s), VMValue::Int(width), VMValue::Str(fill))
                    if width >= 0 && !fill.is_empty() =>
                {
                    let current = s.chars().count();
                    let width = width as usize;
                    if current >= width {
                        Ok(VMValue::Str(s))
                    } else {
                        let needed = width - current;
                        let suffix: String = fill.chars().cycle().take(needed).collect();
                        Ok(VMValue::Str(format!("{s}{suffix}")))
                    }
                }
                (VMValue::Str(_), VMValue::Int(_), VMValue::Str(fill)) if fill.is_empty() => {
                    Err("String.pad_right requires a non-empty fill string".to_string())
                }
                _ => Err("String.pad_right requires (String, Int, String)".to_string()),
            }
        }
        "String.reverse" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.reverse requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.chars().rev().collect())),
                _ => Err("String.reverse requires a String argument".to_string()),
            }
        }
        "String.lines" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.lines requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::List(
                    s.lines()
                        .map(|line| VMValue::Str(line.to_string()))
                        .collect(),
                )),
                _ => Err("String.lines requires a String argument".to_string()),
            }
        }
        "String.words" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.words requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::List(
                    s.split_whitespace()
                        .map(|word| VMValue::Str(word.to_string()))
                        .collect(),
                )),
                _ => Err("String.words requires a String argument".to_string()),
            }
        }
        "String.starts_with" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.starts_with requires 2 arguments".to_string())?;
            let prefix = it
                .next()
                .ok_or_else(|| "String.starts_with requires 2 arguments".to_string())?;
            match (s, prefix) {
                (VMValue::Str(s), VMValue::Str(prefix)) => {
                    Ok(VMValue::Bool(s.starts_with(&prefix)))
                }
                _ => Err("String.starts_with requires (String, String)".to_string()),
            }
        }
        "String.is_url" => {
            let value = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.is_url requires 1 argument".to_string())?;
            match value {
                VMValue::Str(s) => Ok(VMValue::Bool(
                    s.starts_with("http://") || s.starts_with("https://"),
                )),
                _ => Err("String.is_url requires a String argument".to_string()),
            }
        }
        "String.is_slug" => {
            let value = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.is_slug requires 1 argument".to_string())?;
            match value {
                VMValue::Str(s) => Ok(VMValue::Bool(
                    !s.is_empty()
                        && s.chars()
                            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'),
                )),
                _ => Err("String.is_slug requires a String argument".to_string()),
            }
        }
        "String.ends_with" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.ends_with requires 2 arguments".to_string())?;
            let suffix = it
                .next()
                .ok_or_else(|| "String.ends_with requires 2 arguments".to_string())?;
            match (s, suffix) {
                (VMValue::Str(s), VMValue::Str(suffix)) => Ok(VMValue::Bool(s.ends_with(&suffix))),
                _ => Err("String.ends_with requires (String, String)".to_string()),
            }
        }
        "String.contains" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.contains requires 2 arguments".to_string())?;
            let sub = it
                .next()
                .ok_or_else(|| "String.contains requires 2 arguments".to_string())?;
            match (s, sub) {
                (VMValue::Str(s), VMValue::Str(sub)) => Ok(VMValue::Bool(s.contains(&sub))),
                _ => Err("String.contains requires (String, String)".to_string()),
            }
        }
        "String.slice" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.slice requires 3 arguments".to_string())?;
            let start = it
                .next()
                .ok_or_else(|| "String.slice requires 3 arguments".to_string())?;
            let end = it
                .next()
                .ok_or_else(|| "String.slice requires 3 arguments".to_string())?;
            match (s, start, end) {
                (VMValue::Str(s), VMValue::Int(start), VMValue::Int(end)) => {
                    if start < 0 || end < start {
                        return Err("String.slice requires 0 <= start <= end".to_string());
                    }
                    let chars: Vec<char> = s.chars().collect();
                    let start = start as usize;
                    let end = end as usize;
                    if end > chars.len() {
                        return Err("String.slice end is out of bounds".to_string());
                    }
                    Ok(VMValue::Str(chars[start..end].iter().collect()))
                }
                _ => Err("String.slice requires (String, Int, Int)".to_string()),
            }
        }
        "String.repeat" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.repeat requires 2 arguments".to_string())?;
            let n = it
                .next()
                .ok_or_else(|| "String.repeat requires 2 arguments".to_string())?;
            match (s, n) {
                (VMValue::Str(s), VMValue::Int(n)) if n >= 0 => {
                    Ok(VMValue::Str(s.repeat(n as usize)))
                }
                (VMValue::Str(_), VMValue::Int(_)) => {
                    Err("String.repeat requires a non-negative count".to_string())
                }
                _ => Err("String.repeat requires (String, Int)".to_string()),
            }
        }
        "String.char_at" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.char_at requires 2 arguments".to_string())?;
            let idx = it
                .next()
                .ok_or_else(|| "String.char_at requires 2 arguments".to_string())?;
            match (s, idx) {
                (VMValue::Str(s), VMValue::Int(idx)) => {
                    if idx < 0 {
                        return Ok(VMValue::Variant("none".to_string(), None));
                    }
                    let ch = s.chars().nth(idx as usize);
                    Ok(match ch {
                        Some(ch) => VMValue::Variant(
                            "some".to_string(),
                            Some(Box::new(VMValue::Str(ch.to_string()))),
                        ),
                        None => VMValue::Variant("none".to_string(), None),
                    })
                }
                _ => Err("String.char_at requires (String, Int)".to_string()),
            }
        }
        "String.to_int" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.to_int requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(match s.parse::<i64>() {
                    Ok(n) => VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(n)))),
                    Err(_) => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("String.to_int requires a String argument".to_string()),
            }
        }
        "String.to_float" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.to_float requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(match s.parse::<f64>() {
                    Ok(n) => {
                        VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Float(n))))
                    }
                    Err(_) => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("String.to_float requires a String argument".to_string()),
            }
        }
        "String.from_int" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.from_int requires 1 argument".to_string())?;
            match v {
                VMValue::Int(n) => Ok(VMValue::Str(n.to_string())),
                _ => Err("String.from_int requires an Int argument".to_string()),
            }
        }
        "String.from_float" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.from_float requires 1 argument".to_string())?;
            match v {
                VMValue::Float(n) => Ok(VMValue::Str(n.to_string())),
                _ => Err("String.from_float requires a Float argument".to_string()),
            }
        }
        "List.length" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.length requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => Ok(VMValue::Int(xs.len() as i64)),
                _ => Err("List.length requires a List argument".to_string()),
            }
        }
        "List.is_empty" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.is_empty requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => Ok(VMValue::Bool(xs.is_empty())),
                _ => Err("List.is_empty requires a List argument".to_string()),
            }
        }
        "List.first" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.first requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => Ok(match xs.into_iter().next() {
                    Some(first) => VMValue::Variant("some".to_string(), Some(Box::new(first))),
                    None => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("List.first requires a List argument".to_string()),
            }
        }
        "List.last" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.last requires 1 argument".to_string())?;
            match v {
                VMValue::List(mut xs) => Ok(match xs.pop() {
                    Some(last) => VMValue::Variant("some".to_string(), Some(Box::new(last))),
                    None => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("List.last requires a List argument".to_string()),
            }
        }
        "List.push" => {
            let mut it = args.into_iter();
            let list = it
                .next()
                .ok_or_else(|| "List.push requires 2 arguments".to_string())?;
            let item = it
                .next()
                .ok_or_else(|| "List.push requires 2 arguments".to_string())?;
            match list {
                VMValue::List(mut xs) => {
                    xs.push(item);
                    Ok(VMValue::List(xs))
                }
                _ => Err("List.push requires a List as first argument".to_string()),
            }
        }
        "List.zip" => {
            let mut it = args.into_iter();
            let xs = it
                .next()
                .ok_or_else(|| "List.zip requires 2 arguments".to_string())?;
            let ys = it
                .next()
                .ok_or_else(|| "List.zip requires 2 arguments".to_string())?;
            match (xs, ys) {
                (VMValue::List(xs), VMValue::List(ys)) => {
                    let pairs: Vec<VMValue> = xs
                        .into_iter()
                        .zip(ys.into_iter())
                        .map(|(x, y)| {
                            let mut m = HashMap::new();
                            m.insert("first".to_string(), x);
                            m.insert("second".to_string(), y);
                            VMValue::Record(m)
                        })
                        .collect();
                    Ok(VMValue::List(pairs))
                }
                _ => Err("List.zip expects (List, List)".to_string()),
            }
        }
        "List.range" => {
            let mut it = args.into_iter();
            let start = it
                .next()
                .ok_or_else(|| "List.range requires 2 arguments".to_string())?;
            let end = it
                .next()
                .ok_or_else(|| "List.range requires 2 arguments".to_string())?;
            match (start, end) {
                (VMValue::Int(s), VMValue::Int(e)) => {
                    Ok(VMValue::List((s..e).map(VMValue::Int).collect()))
                }
                _ => Err("List.range expects (Int, Int)".to_string()),
            }
        }
        "List.reverse" => match args.into_iter().next() {
            Some(VMValue::List(mut xs)) => {
                xs.reverse();
                Ok(VMValue::List(xs))
            }
            _ => Err("List.reverse expects List".to_string()),
        },
        "List.concat" => {
            let mut it = args.into_iter();
            let xs = it
                .next()
                .ok_or_else(|| "List.concat requires 2 arguments".to_string())?;
            let ys = it
                .next()
                .ok_or_else(|| "List.concat requires 2 arguments".to_string())?;
            match (xs, ys) {
                (VMValue::List(mut xs), VMValue::List(ys)) => {
                    xs.extend(ys);
                    Ok(VMValue::List(xs))
                }
                _ => Err("List.concat expects (List, List)".to_string()),
            }
        }
        "List.take" => {
            let mut it = args.into_iter();
            let list = it
                .next()
                .ok_or_else(|| "List.take requires 2 arguments".to_string())?;
            let n = it
                .next()
                .ok_or_else(|| "List.take requires 2 arguments".to_string())?;
            match (list, n) {
                (VMValue::List(xs), VMValue::Int(n)) => Ok(VMValue::List(
                    xs.into_iter().take(n.max(0) as usize).collect(),
                )),
                _ => Err("List.take expects (List, Int)".to_string()),
            }
        }
        "List.drop" => {
            let mut it = args.into_iter();
            let list = it
                .next()
                .ok_or_else(|| "List.drop requires 2 arguments".to_string())?;
            let n = it
                .next()
                .ok_or_else(|| "List.drop requires 2 arguments".to_string())?;
            match (list, n) {
                (VMValue::List(xs), VMValue::Int(n)) => Ok(VMValue::List(
                    xs.into_iter().skip(n.max(0) as usize).collect(),
                )),
                _ => Err("List.drop expects (List, Int)".to_string()),
            }
        }
        "List.enumerate" => match args.into_iter().next() {
            Some(VMValue::List(xs)) => {
                let pairs: Vec<VMValue> = xs
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let mut m = HashMap::new();
                        m.insert("first".to_string(), VMValue::Int(i as i64));
                        m.insert("second".to_string(), v);
                        VMValue::Record(m)
                    })
                    .collect();
                Ok(VMValue::List(pairs))
            }
            _ => Err("List.enumerate expects List".to_string()),
        },
        "List.join" => {
            let mut it = args.into_iter();
            let list = it
                .next()
                .ok_or_else(|| "List.join requires 2 arguments".to_string())?;
            let sep = it
                .next()
                .ok_or_else(|| "List.join requires 2 arguments".to_string())?;
            match (list, sep) {
                (VMValue::List(xs), VMValue::Str(sep)) => {
                    let mut parts = Vec::with_capacity(xs.len());
                    for v in xs {
                        match v {
                            VMValue::Str(s) => parts.push(s),
                            other => {
                                return Err(format!(
                                    "List.join expects List<String>, got {:?}",
                                    other
                                ));
                            }
                        }
                    }
                    Ok(VMValue::Str(parts.join(&sep)))
                }
                _ => Err("List.join expects (List<String>, String)".to_string()),
            }
        }
        "List.unique" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.unique requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => {
                    let mut seen = HashSet::new();
                    let mut out = Vec::with_capacity(xs.len());
                    for item in xs {
                        let key = vmvalue_repr(&item);
                        if seen.insert(key) {
                            out.push(item);
                        }
                    }
                    Ok(VMValue::List(out))
                }
                _ => Err("List.unique requires a List argument".to_string()),
            }
        }
        "List.flatten" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.flatten requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => {
                    let mut out = Vec::new();
                    for inner in xs {
                        match inner {
                            VMValue::List(items) => out.extend(items),
                            _ => return Err("List.flatten requires List<List<T>>".to_string()),
                        }
                    }
                    Ok(VMValue::List(out))
                }
                _ => Err("List.flatten requires a List argument".to_string()),
            }
        }
        "List.chunk" => {
            let mut it = args.into_iter();
            let list = it
                .next()
                .ok_or_else(|| "List.chunk requires 2 arguments".to_string())?;
            let n = it
                .next()
                .ok_or_else(|| "List.chunk requires 2 arguments".to_string())?;
            match (list, n) {
                (VMValue::List(xs), VMValue::Int(n)) if n > 0 => {
                    let size = n as usize;
                    let chunks = xs
                        .chunks(size)
                        .map(|chunk| VMValue::List(chunk.to_vec()))
                        .collect();
                    Ok(VMValue::List(chunks))
                }
                (VMValue::List(_), VMValue::Int(_)) => {
                    Err("List.chunk requires a positive chunk size".to_string())
                }
                _ => Err("List.chunk expects (List, Int)".to_string()),
            }
        }
        "List.sum" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.sum requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => {
                    let mut sum = 0i64;
                    for item in xs {
                        match item {
                            VMValue::Int(n) => sum += n,
                            _ => return Err("List.sum requires List<Int>".to_string()),
                        }
                    }
                    Ok(VMValue::Int(sum))
                }
                _ => Err("List.sum requires a List argument".to_string()),
            }
        }
        "List.sum_float" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.sum_float requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => {
                    let mut sum = 0.0f64;
                    for item in xs {
                        match item {
                            VMValue::Float(n) => sum += n,
                            _ => return Err("List.sum_float requires List<Float>".to_string()),
                        }
                    }
                    Ok(VMValue::Float(sum))
                }
                _ => Err("List.sum_float requires a List argument".to_string()),
            }
        }
        "List.min" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.min requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => {
                    let mut min: Option<i64> = None;
                    for item in xs {
                        match item {
                            VMValue::Int(n) => min = Some(min.map(|m| m.min(n)).unwrap_or(n)),
                            _ => return Err("List.min requires List<Int>".to_string()),
                        }
                    }
                    Ok(match min {
                        Some(n) => {
                            VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(n))))
                        }
                        None => VMValue::Variant("none".to_string(), None),
                    })
                }
                _ => Err("List.min requires a List argument".to_string()),
            }
        }
        "List.max" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.max requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => {
                    let mut max: Option<i64> = None;
                    for item in xs {
                        match item {
                            VMValue::Int(n) => max = Some(max.map(|m| m.max(n)).unwrap_or(n)),
                            _ => return Err("List.max requires List<Int>".to_string()),
                        }
                    }
                    Ok(match max {
                        Some(n) => {
                            VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(n))))
                        }
                        None => VMValue::Variant("none".to_string(), None),
                    })
                }
                _ => Err("List.max requires a List argument".to_string()),
            }
        }
        "Map.set" => {
            let mut it = args.into_iter();
            let map = it
                .next()
                .ok_or_else(|| "Map.set requires 3 arguments".to_string())?;
            let key = it
                .next()
                .ok_or_else(|| "Map.set requires 3 arguments".to_string())?;
            let val = it
                .next()
                .ok_or_else(|| "Map.set requires 3 arguments".to_string())?;
            let mut m = match map {
                VMValue::Record(m) => m,
                VMValue::Unit => HashMap::new(),
                _ => return Err("Map.set requires a Record or Unit as first argument".to_string()),
            };
            let k = match key {
                VMValue::Str(s) => s,
                _ => return Err("Map.set requires a String key".to_string()),
            };
            m.insert(k, val);
            Ok(VMValue::Record(m))
        }
        "Map.get" => {
            let mut it = args.into_iter();
            let map = it
                .next()
                .ok_or_else(|| "Map.get requires 2 arguments".to_string())?;
            let key = it
                .next()
                .ok_or_else(|| "Map.get requires 2 arguments".to_string())?;
            let m = match map {
                VMValue::Record(m) => m,
                _ => return Err("Map.get requires a Record as first argument".to_string()),
            };
            let k = match key {
                VMValue::Str(s) => s,
                _ => return Err("Map.get requires a String key".to_string()),
            };
            Ok(match m.get(&k) {
                Some(v) => VMValue::Variant("some".to_string(), Some(Box::new(v.clone()))),
                None => VMValue::Variant("none".to_string(), None),
            })
        }
        "Map.delete" => {
            let mut it = args.into_iter();
            let map = it
                .next()
                .ok_or_else(|| "Map.delete requires 2 arguments".to_string())?;
            let key = it
                .next()
                .ok_or_else(|| "Map.delete requires 2 arguments".to_string())?;
            let mut m = match map {
                VMValue::Record(m) => m,
                _ => return Err("Map.delete requires a Record as first argument".to_string()),
            };
            let k = match key {
                VMValue::Str(s) => s,
                _ => return Err("Map.delete requires a String key".to_string()),
            };
            m.remove(&k);
            Ok(VMValue::Record(m))
        }
        "Map.keys" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.keys requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => {
                    let mut keys: Vec<VMValue> =
                        m.keys().map(|k| VMValue::Str(k.clone())).collect();
                    keys.sort_by(|a, b| match (a, b) {
                        (VMValue::Str(x), VMValue::Str(y)) => x.cmp(y),
                        _ => std::cmp::Ordering::Equal,
                    });
                    Ok(VMValue::List(keys))
                }
                _ => Err("Map.keys requires a Record (map) argument".to_string()),
            }
        }
        "Map.values" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.values requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => {
                    let mut pairs: Vec<_> = m.iter().collect();
                    pairs.sort_by(|a, b| a.0.cmp(b.0));
                    Ok(VMValue::List(
                        pairs.into_iter().map(|(_, v)| v.clone()).collect(),
                    ))
                }
                _ => Err("Map.values requires a Record (map) argument".to_string()),
            }
        }
        "Map.has_key" => {
            let mut it = args.into_iter();
            let map = it
                .next()
                .ok_or_else(|| "Map.has_key requires 2 arguments".to_string())?;
            let key = it
                .next()
                .ok_or_else(|| "Map.has_key requires 2 arguments".to_string())?;
            match (map, key) {
                (VMValue::Record(m), VMValue::Str(k)) => Ok(VMValue::Bool(m.contains_key(&k))),
                _ => Err("Map.has_key requires (Map, String)".to_string()),
            }
        }
        "Map.size" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.size requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => Ok(VMValue::Int(m.len() as i64)),
                _ => Err("Map.size requires a Map argument".to_string()),
            }
        }
        "Map.is_empty" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.is_empty requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => Ok(VMValue::Bool(m.is_empty())),
                _ => Err("Map.is_empty requires a Map argument".to_string()),
            }
        }
        "Map.merge" => {
            let mut it = args.into_iter();
            let base = it
                .next()
                .ok_or_else(|| "Map.merge requires 2 arguments".to_string())?;
            let overrides = it
                .next()
                .ok_or_else(|| "Map.merge requires 2 arguments".to_string())?;
            match (base, overrides) {
                (VMValue::Record(mut base), VMValue::Record(overrides)) => {
                    for (k, v) in overrides {
                        base.insert(k, v);
                    }
                    Ok(VMValue::Record(base))
                }
                _ => Err("Map.merge requires (Map, Map)".to_string()),
            }
        }
        "Map.from_list" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.from_list requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => {
                    let mut out = HashMap::with_capacity(xs.len());
                    for pair in xs {
                        match pair {
                            VMValue::Record(mut fields) => {
                                let first = fields.remove("first");
                                let second = fields.remove("second");
                                match (first, second) {
                                    (Some(VMValue::Str(k)), Some(v)) => {
                                        out.insert(k, v);
                                    }
                                    _ => {
                                        return Err(
                                            "Map.from_list requires Pair-like records with { first: String second: V }"
                                                .to_string(),
                                        )
                                    }
                                }
                            }
                            _ => {
                                return Err(
                                    "Map.from_list requires List<Pair<String, V>>".to_string()
                                );
                            }
                        }
                    }
                    Ok(VMValue::Record(out))
                }
                _ => Err("Map.from_list requires a List argument".to_string()),
            }
        }
        "Map.to_list" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.to_list requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => {
                    let mut pairs: Vec<_> = m.into_iter().collect();
                    pairs.sort_by(|a, b| a.0.cmp(&b.0));
                    Ok(VMValue::List(
                        pairs
                            .into_iter()
                            .map(|(k, v)| {
                                let mut fields = HashMap::new();
                                fields.insert("first".to_string(), VMValue::Str(k));
                                fields.insert("second".to_string(), v);
                                VMValue::Record(fields)
                            })
                            .collect(),
                    ))
                }
                _ => Err("Map.to_list requires a Map argument".to_string()),
            }
        }
        "Json.null" => Ok(json_variant_vm("json_null", None)),
        "Json.bool" => match args.into_iter().next() {
            Some(VMValue::Bool(b)) => Ok(json_variant_vm("json_bool", Some(VMValue::Bool(b)))),
            Some(other) => Err(format!(
                "Json.bool expects Bool, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.bool requires 1 argument".to_string()),
        },
        "Json.int" => match args.into_iter().next() {
            Some(VMValue::Int(i)) => Ok(json_variant_vm("json_int", Some(VMValue::Int(i)))),
            Some(other) => Err(format!(
                "Json.int expects Int, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.int requires 1 argument".to_string()),
        },
        "Json.float" => match args.into_iter().next() {
            Some(VMValue::Float(f)) => Ok(json_variant_vm("json_float", Some(VMValue::Float(f)))),
            Some(other) => Err(format!(
                "Json.float expects Float, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.float requires 1 argument".to_string()),
        },
        "Json.str" => match args.into_iter().next() {
            Some(VMValue::Str(s)) => Ok(json_variant_vm("json_str", Some(VMValue::Str(s)))),
            Some(other) => Err(format!(
                "Json.str expects String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.str requires 1 argument".to_string()),
        },
        "Json.array" => match args.into_iter().next() {
            Some(VMValue::List(items)) => {
                Ok(json_variant_vm("json_array", Some(VMValue::List(items))))
            }
            Some(other) => Err(format!(
                "Json.array expects List<Json>, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.array requires 1 argument".to_string()),
        },
        "Json.object" => match args.into_iter().next() {
            Some(VMValue::List(fields)) => {
                let mut obj = HashMap::new();
                for field in fields {
                    let rec = match field {
                        VMValue::Record(rec) => rec,
                        other => {
                            return Err(format!(
                                "Json.object expects List<JsonField>, got {}",
                                vmvalue_type_name(&other)
                            ));
                        }
                    };
                    let key = match rec.get("key") {
                        Some(VMValue::Str(s)) => s.clone(),
                        Some(other) => {
                            return Err(format!(
                                "JsonField.key must be String, got {}",
                                vmvalue_type_name(other)
                            ));
                        }
                        None => return Err("JsonField missing `key`".to_string()),
                    };
                    let value = rec
                        .get("value")
                        .cloned()
                        .ok_or_else(|| "JsonField missing `value`".to_string())?;
                    obj.insert(key, value);
                }
                Ok(json_variant_vm("json_object", Some(VMValue::Record(obj))))
            }
            Some(other) => Err(format!(
                "Json.object expects List<JsonField>, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.object requires 1 argument".to_string()),
        },
        "Json.parse_raw" => match args.into_iter().next() {
            Some(VMValue::Str(text)) => match parse_json_object_raw(&text) {
                Ok(map) => Ok(ok_vm(VMValue::Record(map))),
                Err(message) => Ok(err_vm(schema_error_vm("", "valid json object", message))),
            },
            Some(other) => Err(format!(
                "Json.parse_raw expects String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.parse_raw requires 1 argument".to_string()),
        },
        "Json.parse_array_raw" => match args.into_iter().next() {
            Some(VMValue::Str(text)) => match parse_json_array_raw(&text) {
                Ok(rows) => Ok(ok_vm(VMValue::List(rows))),
                Err(message) => Ok(err_vm(schema_error_vm("", "valid json array", message))),
            },
            Some(other) => Err(format!(
                "Json.parse_array_raw expects String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.parse_array_raw requires 1 argument".to_string()),
        },
        "Json.write_raw" => match args.into_iter().next() {
            Some(VMValue::Record(map)) => serde_json::to_string(&schema_record_to_string_map(&map))
                .map(VMValue::Str)
                .map_err(|e| format!("Json.write_raw failed: {}", e)),
            Some(other) => Err(format!(
                "Json.write_raw expects Map<String,String>, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.write_raw requires 1 argument".to_string()),
        },
        "Json.write_array_raw" => match args.into_iter().next() {
            Some(VMValue::List(rows)) => {
                let objects: Result<Vec<_>, _> = rows
                    .into_iter()
                    .map(|row| match row {
                        VMValue::Record(map) => Ok(schema_record_to_string_map(&map)),
                        other => Err(format!(
                            "Json.write_array_raw expects List<Map<String,String>>, got {}",
                            vmvalue_type_name(&other)
                        )),
                    })
                    .collect();
                serde_json::to_string(&objects?)
                    .map(VMValue::Str)
                    .map_err(|e| format!("Json.write_array_raw failed: {}", e))
            }
            Some(other) => Err(format!(
                "Json.write_array_raw expects List<Map<String,String>>, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.write_array_raw requires 1 argument".to_string()),
        },
        "Json.parse" => match args.into_iter().next() {
            Some(VMValue::Str(s)) => match serde_json::from_str::<SerdeJsonValue>(&s) {
                Ok(v) => Ok(VMValue::Variant(
                    "some".to_string(),
                    Some(Box::new(serde_to_vm_json(v))),
                )),
                Err(_) => Ok(VMValue::Variant("none".to_string(), None)),
            },
            Some(other) => Err(format!(
                "Json.parse expects String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.parse requires 1 argument".to_string()),
        },
        "Json.encode" | "Json.encode_pretty" => {
            let json = args
                .into_iter()
                .next()
                .ok_or_else(|| format!("{} requires 1 argument", name))?;
            let serde = vm_json_to_serde(&json).ok_or_else(|| format!("{} expects Json", name))?;
            let out = if name == "Json.encode_pretty" {
                serde_json::to_string_pretty(&serde)
            } else {
                serde_json::to_string(&serde)
            }
            .map_err(|e| format!("{} failed: {}", name, e))?;
            Ok(VMValue::Str(out))
        }
        "Json.get" => {
            if args.len() != 2 {
                return Err("Json.get requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let json = it.next().unwrap();
            let key = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "Json.get expects String key, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            match json {
                VMValue::Variant(tag, Some(payload)) if tag == "json_object" => match *payload {
                    VMValue::Record(map) => Ok(map
                        .get(&key)
                        .cloned()
                        .map(|v| VMValue::Variant("some".to_string(), Some(Box::new(v))))
                        .unwrap_or(VMValue::Variant("none".to_string(), None))),
                    _ => Err("Json.get received malformed json_object payload".to_string()),
                },
                _ => Ok(VMValue::Variant("none".to_string(), None)),
            }
        }
        "Json.at" => {
            if args.len() != 2 {
                return Err("Json.at requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let json = it.next().unwrap();
            let idx = match it.next().unwrap() {
                VMValue::Int(i) => i,
                other => {
                    return Err(format!(
                        "Json.at expects Int index, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            match json {
                VMValue::Variant(tag, Some(payload)) if tag == "json_array" => match *payload {
                    VMValue::List(items) if idx >= 0 => Ok(items
                        .get(idx as usize)
                        .cloned()
                        .map(|v| VMValue::Variant("some".to_string(), Some(Box::new(v))))
                        .unwrap_or(VMValue::Variant("none".to_string(), None))),
                    VMValue::List(_) => Ok(VMValue::Variant("none".to_string(), None)),
                    _ => Err("Json.at received malformed json_array payload".to_string()),
                },
                _ => Ok(VMValue::Variant("none".to_string(), None)),
            }
        }
        "Json.as_str" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_str" => {
                Ok(VMValue::Variant("some".to_string(), Some(payload)))
            }
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_str requires 1 argument".to_string()),
        },
        "Json.as_int" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_int" => {
                Ok(VMValue::Variant("some".to_string(), Some(payload)))
            }
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_int requires 1 argument".to_string()),
        },
        "Json.as_float" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_float" => {
                Ok(VMValue::Variant("some".to_string(), Some(payload)))
            }
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_float requires 1 argument".to_string()),
        },
        "Json.as_bool" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_bool" => {
                Ok(VMValue::Variant("some".to_string(), Some(payload)))
            }
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_bool requires 1 argument".to_string()),
        },
        "Json.as_array" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_array" => {
                Ok(VMValue::Variant("some".to_string(), Some(payload)))
            }
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_array requires 1 argument".to_string()),
        },
        "Json.is_null" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, None)) if tag == "json_null" => Ok(VMValue::Bool(true)),
            Some(_) => Ok(VMValue::Bool(false)),
            None => Err("Json.is_null requires 1 argument".to_string()),
        },
        "Json.keys" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_object" => match *payload {
                VMValue::Record(map) => {
                    let mut keys: Vec<VMValue> = map.into_keys().map(VMValue::Str).collect();
                    keys.sort_by(|a, b| vmvalue_repr(a).cmp(&vmvalue_repr(b)));
                    Ok(VMValue::Variant(
                        "some".to_string(),
                        Some(Box::new(VMValue::List(keys))),
                    ))
                }
                _ => Err("Json.keys received malformed json_object payload".to_string()),
            },
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.keys requires 1 argument".to_string()),
        },
        "Json.length" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_array" => match *payload {
                VMValue::List(items) => Ok(VMValue::Variant(
                    "some".to_string(),
                    Some(Box::new(VMValue::Int(items.len() as i64))),
                )),
                _ => Err("Json.length received malformed json_array payload".to_string()),
            },
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_object" => match *payload {
                VMValue::Record(map) => Ok(VMValue::Variant(
                    "some".to_string(),
                    Some(Box::new(VMValue::Int(map.len() as i64))),
                )),
                _ => Err("Json.length received malformed json_object payload".to_string()),
            },
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.length requires 1 argument".to_string()),
        },
        "Csv.parse" => {
            let input = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Csv.parse requires 1 argument".to_string())?,
                "Csv.parse",
            )?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(input.as_bytes());
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = record.map_err(|e| format!("Csv.parse failed: {}", e))?;
                rows.push(VMValue::List(
                    record
                        .iter()
                        .map(|cell| VMValue::Str(cell.to_string()))
                        .collect(),
                ));
            }
            Ok(VMValue::List(rows))
        }
        "Csv.parse_raw" => {
            if args.len() != 3 {
                return Err("Csv.parse_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let text = vm_string(it.next().unwrap(), "Csv.parse_raw")?;
            let delimiter = vm_string(it.next().unwrap(), "Csv.parse_raw")?;
            let has_header = match it.next().unwrap() {
                VMValue::Bool(v) => v,
                other => {
                    return Err(format!(
                        "Csv.parse_raw expects Bool has_header, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let delimiter_char = delimiter
                .chars()
                .next()
                .ok_or_else(|| "Csv.parse_raw delimiter must not be empty".to_string())?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(has_header)
                .delimiter(delimiter_char as u8)
                .from_reader(text.as_bytes());
            let headers = if has_header {
                Some(
                    rdr.headers()
                        .map_err(|e| format!("csv parse error: {}", e))?
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            };
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = match record {
                    Ok(record) => record,
                    Err(e) => return Ok(err_vm(schema_error_vm("", "valid csv", e.to_string()))),
                };
                let mut row = HashMap::new();
                for (idx, value) in record.iter().enumerate() {
                    let key = headers
                        .as_ref()
                        .and_then(|h| h.get(idx).cloned())
                        .unwrap_or_else(|| idx.to_string());
                    row.insert(key, VMValue::Str(value.to_string()));
                }
                rows.push(VMValue::Record(row));
            }
            Ok(ok_vm(VMValue::List(rows)))
        }
        "Csv.parse_with_header" => {
            let input = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Csv.parse_with_header requires 1 argument".to_string())?,
                "Csv.parse_with_header",
            )?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .from_reader(input.as_bytes());
            let headers = rdr
                .headers()
                .map_err(|e| format!("Csv.parse_with_header failed: {}", e))?
                .clone();
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = record.map_err(|e| format!("Csv.parse_with_header failed: {}", e))?;
                let mut row = HashMap::new();
                for (key, value) in headers.iter().zip(record.iter()) {
                    row.insert(key.to_string(), VMValue::Str(value.to_string()));
                }
                rows.push(VMValue::Record(row));
            }
            Ok(VMValue::List(rows))
        }
        "Csv.encode" => {
            let rows = match args.into_iter().next() {
                Some(VMValue::List(rows)) => rows,
                Some(other) => {
                    return Err(format!(
                        "Csv.encode expects List<List<String>>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("Csv.encode requires 1 argument".to_string()),
            };
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            for row in rows {
                let fields = vm_string_list(row, "Csv.encode")?;
                writer
                    .write_record(fields)
                    .map_err(|e| format!("Csv.encode failed: {}", e))?;
            }
            let bytes = writer
                .into_inner()
                .map_err(|e| format!("Csv.encode failed: {}", e.into_error()))?;
            let out = String::from_utf8(bytes)
                .map_err(|e| format!("Csv.encode produced invalid UTF-8: {}", e))?;
            Ok(VMValue::Str(out))
        }
        "Csv.encode_with_header" => {
            if args.len() != 2 {
                return Err("Csv.encode_with_header requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let header = vm_string_list(it.next().unwrap(), "Csv.encode_with_header")?;
            let rows = match it.next().unwrap() {
                VMValue::List(rows) => rows,
                other => {
                    return Err(format!(
                        "Csv.encode_with_header expects List<List<String>>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            writer
                .write_record(&header)
                .map_err(|e| format!("Csv.encode_with_header failed: {}", e))?;
            for row in rows {
                let fields = vm_string_list(row, "Csv.encode_with_header")?;
                writer
                    .write_record(fields)
                    .map_err(|e| format!("Csv.encode_with_header failed: {}", e))?;
            }
            let bytes = writer
                .into_inner()
                .map_err(|e| format!("Csv.encode_with_header failed: {}", e.into_error()))?;
            let out = String::from_utf8(bytes)
                .map_err(|e| format!("Csv.encode_with_header produced invalid UTF-8: {}", e))?;
            Ok(VMValue::Str(out))
        }
        "Csv.from_records" => {
            let records = match args.into_iter().next() {
                Some(VMValue::List(records)) => records,
                Some(other) => {
                    return Err(format!(
                        "Csv.from_records expects List<Map<String>>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("Csv.from_records requires 1 argument".to_string()),
            };
            let mut headers = std::collections::BTreeSet::new();
            let mut rows = Vec::new();
            for record in records {
                match record {
                    VMValue::Record(map) => {
                        for key in map.keys() {
                            headers.insert(key.clone());
                        }
                        rows.push(map);
                    }
                    other => {
                        return Err(format!(
                            "Csv.from_records expects record rows, got {}",
                            vmvalue_type_name(&other)
                        ));
                    }
                }
            }
            let header: Vec<String> = headers.into_iter().collect();
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            writer
                .write_record(&header)
                .map_err(|e| format!("Csv.from_records failed: {}", e))?;
            for row in rows {
                let mut values = Vec::with_capacity(header.len());
                for key in &header {
                    let value = row.get(key).cloned().unwrap_or(VMValue::Str(String::new()));
                    values.push(vm_string(value, "Csv.from_records")?);
                }
                writer
                    .write_record(values)
                    .map_err(|e| format!("Csv.from_records failed: {}", e))?;
            }
            let bytes = writer
                .into_inner()
                .map_err(|e| format!("Csv.from_records failed: {}", e.into_error()))?;
            let out = String::from_utf8(bytes)
                .map_err(|e| format!("Csv.from_records produced invalid UTF-8: {}", e))?;
            Ok(VMValue::Str(out))
        }
        "Csv.write_raw" => {
            if args.len() != 2 {
                return Err("Csv.write_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let rows = schema_rows_from_vm(it.next().unwrap(), "Csv.write_raw")?;
            let delimiter = vm_string(it.next().unwrap(), "Csv.write_raw")?;
            let delimiter_char = delimiter
                .chars()
                .next()
                .ok_or_else(|| "Csv.write_raw delimiter must not be empty".to_string())?;
            let mut writer = csv::WriterBuilder::new()
                .delimiter(delimiter_char as u8)
                .from_writer(vec![]);
            if let Some(first) = rows.first() {
                let mut header: Vec<String> = first.keys().cloned().collect();
                header.sort();
                writer
                    .write_record(&header)
                    .map_err(|e| format!("Csv.write_raw failed: {}", e))?;
                for row in rows {
                    let values: Vec<String> = header
                        .iter()
                        .map(|key| {
                            row.get(key)
                                .map(vm_scalar_to_plain_string)
                                .unwrap_or_default()
                        })
                        .collect();
                    writer
                        .write_record(values)
                        .map_err(|e| format!("Csv.write_raw failed: {}", e))?;
                }
            }
            let bytes = writer
                .into_inner()
                .map_err(|e| format!("Csv.write_raw failed: {}", e.into_error()))?;
            String::from_utf8(bytes)
                .map(VMValue::Str)
                .map_err(|e| format!("Csv.write_raw produced invalid UTF-8: {}", e))
        }
        "Schema.adapt" => {
            if args.len() != 2 {
                return Err("Schema.adapt requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let rows = schema_rows_from_vm(it.next().unwrap(), "Schema.adapt")?;
            let type_name = vm_string(it.next().unwrap(), "Schema.adapt")?;
            Ok(schema_adapt_rows(rows, &type_name, type_metas))
        }
        "Schema.adapt_one" => {
            if args.len() != 2 {
                return Err("Schema.adapt_one requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let row = match it.next().unwrap() {
                VMValue::Record(map) => map,
                other => {
                    return Err(format!(
                        "Schema.adapt_one expects Map<String,String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let type_name = vm_string(it.next().unwrap(), "Schema.adapt_one")?;
            let adapted = schema_adapt_rows(vec![row], &type_name, type_metas);
            match &adapted {
                VMValue::Variant(tag, Some(payload)) if tag == "ok" => match payload.as_ref() {
                    VMValue::List(rows) => {
                        Ok(ok_vm(rows.first().cloned().unwrap_or(VMValue::Unit)))
                    }
                    _ => Ok(adapted),
                },
                _ => Ok(adapted),
            }
        }
        "Schema.to_csv" => {
            if args.len() != 2 {
                return Err("Schema.to_csv requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let rows = match it.next().unwrap() {
                VMValue::List(rows) => rows,
                other => {
                    return Err(format!(
                        "Schema.to_csv expects List<Record>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let type_name = vm_string(it.next().unwrap(), "Schema.to_csv")?;
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            let header: Vec<String> = if let Some(meta) = type_metas.get(&type_name) {
                meta.fields.iter().map(|field| field.name.clone()).collect()
            } else if let Some(VMValue::Record(first)) = rows.first() {
                let mut keys: Vec<String> = first.keys().cloned().collect();
                keys.sort();
                keys
            } else {
                Vec::new()
            };
            writer
                .write_record(&header)
                .map_err(|e| format!("Schema.to_csv failed: {}", e))?;
            for row in rows {
                let VMValue::Record(record) = row else {
                    return Err("Schema.to_csv expects record rows".to_string());
                };
                let values: Vec<String> = header
                    .iter()
                    .map(|field_name| {
                        record
                            .get(field_name)
                            .map(vm_scalar_to_plain_string)
                            .unwrap_or_default()
                    })
                    .collect();
                writer
                    .write_record(values)
                    .map_err(|e| format!("Schema.to_csv failed: {}", e))?;
            }
            let bytes = writer
                .into_inner()
                .map_err(|e| format!("Schema.to_csv failed: {}", e.into_error()))?;
            String::from_utf8(bytes)
                .map(VMValue::Str)
                .map_err(|e| format!("Schema.to_csv produced invalid UTF-8: {}", e))
        }
        "Schema.to_json" => {
            if args.len() != 2 {
                return Err("Schema.to_json requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let value = it.next().unwrap();
            let type_name = vm_string(it.next().unwrap(), "Schema.to_json")?;
            let json = schema_to_json_value(&value, &type_name, type_metas)?;
            serde_json::to_string(&json)
                .map(VMValue::Str)
                .map_err(|e| format!("Schema.to_json failed: {}", e))
        }
        "Schema.to_json_array" => {
            if args.len() != 2 {
                return Err("Schema.to_json_array requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let rows = match it.next().unwrap() {
                VMValue::List(rows) => rows,
                other => {
                    return Err(format!(
                        "Schema.to_json_array expects List<Record>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let type_name = vm_string(it.next().unwrap(), "Schema.to_json_array")?;
            let mut json_rows = Vec::with_capacity(rows.len());
            for row in rows {
                json_rows.push(schema_to_json_value(&row, &type_name, type_metas)?);
            }
            serde_json::to_string(&json_rows)
                .map(VMValue::Str)
                .map_err(|e| format!("Schema.to_json_array failed: {}", e))
        }
        "Trace.print" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Trace.print requires 1 argument".to_string())?;
            let s = match v {
                VMValue::Str(s) => s,
                other => vmvalue_repr(&other),
            };
            eprintln!("[trace] {}", s);
            Ok(VMValue::Unit)
        }
        "Trace.log" => {
            let mut it = args.into_iter();
            let label = it
                .next()
                .ok_or_else(|| "Trace.log requires 2 arguments".to_string())?;
            let val = it
                .next()
                .ok_or_else(|| "Trace.log requires 2 arguments".to_string())?;
            let label_s = match label {
                VMValue::Str(s) => s,
                other => vmvalue_repr(&other),
            };
            eprintln!("[trace] {}: {}", label_s, vmvalue_repr(&val));
            Ok(VMValue::Unit)
        }
        "Emit.log" => {
            let log: Vec<VMValue> = emit_log
                .iter()
                .map(|v| VMValue::Str(vmvalue_repr(v)))
                .collect();
            Ok(VMValue::List(log))
        }
        "Db.execute" => {
            if args.is_empty() {
                return Err("Db.execute requires a SQL string".to_string());
            }
            let mut it = args.into_iter();
            let sql = vm_string(it.next().expect("sql"), "Db.execute")?;
            let params: Vec<VMValue> = it.collect();
            with_db_path(db_path, |conn| {
                let mut stmt = conn.prepare(&sql).map_err(|e| format!("Db error: {}", e))?;
                let bound: Vec<rusqlite::types::Value> =
                    params.iter().map(vmvalue_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> =
                    bound.iter().map(|b| b as &dyn rusqlite::ToSql).collect();
                let rows = stmt
                    .execute(refs.as_slice())
                    .map_err(|e| format!("Db error: {}", e))?;
                Ok(VMValue::Int(rows as i64))
            })
        }
        "Db.query" => {
            if args.is_empty() {
                return Err("Db.query requires a SQL string".to_string());
            }
            let mut it = args.into_iter();
            let sql = vm_string(it.next().expect("sql"), "Db.query")?;
            let params: Vec<VMValue> = it.collect();
            with_db_path(db_path, |conn| {
                let mut stmt = conn.prepare(&sql).map_err(|e| format!("Db error: {}", e))?;
                let bound: Vec<rusqlite::types::Value> =
                    params.iter().map(vmvalue_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> =
                    bound.iter().map(|b| b as &dyn rusqlite::ToSql).collect();
                let col_names: Vec<String> =
                    stmt.column_names().iter().map(|s| s.to_string()).collect();
                let rows = stmt
                    .query_map(refs.as_slice(), |row| {
                        let mut map = HashMap::new();
                        for (i, name) in col_names.iter().enumerate() {
                            let value: rusqlite::types::Value = row.get(i)?;
                            map.insert(name.clone(), VMValue::Str(sqlite_value_to_string(value)));
                        }
                        Ok(VMValue::Record(map))
                    })
                    .map_err(|e| format!("Db error: {}", e))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Db error: {}", e))?;
                Ok(VMValue::List(rows))
            })
        }
        "Db.query_one" => {
            if args.is_empty() {
                return Err("Db.query_one requires a SQL string".to_string());
            }
            let mut it = args.into_iter();
            let sql = vm_string(it.next().expect("sql"), "Db.query_one")?;
            let params: Vec<VMValue> = it.collect();
            with_db_path(db_path, |conn| {
                let mut stmt = conn.prepare(&sql).map_err(|e| format!("Db error: {}", e))?;
                let bound: Vec<rusqlite::types::Value> =
                    params.iter().map(vmvalue_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> =
                    bound.iter().map(|b| b as &dyn rusqlite::ToSql).collect();
                let col_names: Vec<String> =
                    stmt.column_names().iter().map(|s| s.to_string()).collect();
                let mut rows = stmt
                    .query(refs.as_slice())
                    .map_err(|e| format!("Db error: {}", e))?;
                match rows.next().map_err(|e| format!("Db error: {}", e))? {
                    None => Ok(VMValue::Variant("none".to_string(), None)),
                    Some(row) => {
                        let mut map = HashMap::new();
                        for (i, name) in col_names.iter().enumerate() {
                            let value: rusqlite::types::Value =
                                row.get(i).map_err(|e| format!("Db error: {}", e))?;
                            map.insert(name.clone(), VMValue::Str(sqlite_value_to_string(value)));
                        }
                        Ok(VMValue::Variant(
                            "some".to_string(),
                            Some(Box::new(VMValue::Record(map))),
                        ))
                    }
                }
            })
        }
        "Http.get" => {
            let url = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Http.get requires a URL argument".to_string())?,
                "Http.get",
            )?;
            match ureq::get(&url).call() {
                Ok(resp) => {
                    let body = resp
                        .into_string()
                        .map_err(|e| format!("Http.get read error: {}", e))?;
                    Ok(VMValue::Variant(
                        "ok".to_string(),
                        Some(Box::new(VMValue::Str(body))),
                    ))
                }
                Err(e) => Ok(VMValue::Variant(
                    "err".to_string(),
                    Some(Box::new(VMValue::Str(e.to_string()))),
                )),
            }
        }
        "Http.get_raw" => {
            let url = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Http.get_raw requires a URL argument".to_string())?,
                "Http.get_raw",
            )?;
            match ureq::get(&url).call() {
                Ok(resp) => {
                    let status = resp.status() as i64;
                    let content_type = resp
                        .header("Content-Type")
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let body = resp
                        .into_string()
                        .map_err(|e| format!("Http.get_raw read error: {}", e))?;
                    Ok(ok_vm(http_response_vm(status, body, content_type)))
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    Ok(err_vm(http_error_vm(2, body, status as i64)))
                }
                Err(ureq::Error::Transport(err)) => {
                    let msg = err.to_string();
                    let code = if msg.to_ascii_lowercase().contains("timed out") {
                        1
                    } else {
                        0
                    };
                    Ok(err_vm(http_error_vm(code, msg, 0)))
                }
            }
        }
        "Http.post" => {
            if args.len() < 2 {
                return Err("Http.post requires 2 arguments (url, body)".to_string());
            }
            let mut it = args.into_iter();
            let url = vm_string(it.next().expect("url"), "Http.post")?;
            let body = match it.next().expect("body") {
                VMValue::Str(s) => s,
                other => vmvalue_repr(&other),
            };
            match ureq::post(&url).send_string(&body) {
                Ok(resp) => {
                    let body = resp
                        .into_string()
                        .map_err(|e| format!("Http.post read error: {}", e))?;
                    Ok(VMValue::Variant(
                        "ok".to_string(),
                        Some(Box::new(VMValue::Str(body))),
                    ))
                }
                Err(e) => Ok(VMValue::Variant(
                    "err".to_string(),
                    Some(Box::new(VMValue::Str(e.to_string()))),
                )),
            }
        }
        "Http.post_raw" => {
            if args.len() != 3 {
                return Err("Http.post_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let url = vm_string(it.next().unwrap(), "Http.post_raw url")?;
            let body = vm_string(it.next().unwrap(), "Http.post_raw body")?;
            let content_type = vm_string(it.next().unwrap(), "Http.post_raw content_type")?;
            match ureq::post(&url)
                .set("Content-Type", &content_type)
                .send_string(&body)
            {
                Ok(resp) => {
                    let status = resp.status() as i64;
                    let response_content_type = resp
                        .header("Content-Type")
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let response_body = resp
                        .into_string()
                        .map_err(|e| format!("Http.post_raw read error: {}", e))?;
                    Ok(ok_vm(http_response_vm(
                        status,
                        response_body,
                        response_content_type,
                    )))
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    Ok(err_vm(http_error_vm(2, body, status as i64)))
                }
                Err(ureq::Error::Transport(err)) => {
                    let msg = err.to_string();
                    let code = if msg.to_ascii_lowercase().contains("timed out") {
                        1
                    } else {
                        0
                    };
                    Ok(err_vm(http_error_vm(code, msg, 0)))
                }
            }
        }
        "Grpc.encode_raw" => {
            if args.len() != 2 {
                return Err("Grpc.encode_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let type_name = vm_string(it.next().unwrap(), "Grpc.encode_raw type_name")?;
            let row = match it.next().unwrap() {
                VMValue::Record(map) => schema_record_to_string_map(&map),
                other => {
                    return Err(format!(
                        "Grpc.encode_raw expects Map<String,String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let bytes = map_to_proto_bytes(&type_name, &row, type_metas)?;
            Ok(VMValue::Str(BASE64.encode(bytes)))
        }
        "Grpc.decode_raw" => {
            if args.len() != 2 {
                return Err("Grpc.decode_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let type_name = vm_string(it.next().unwrap(), "Grpc.decode_raw type_name")?;
            let encoded = vm_string(it.next().unwrap(), "Grpc.decode_raw encoded")?;
            let bytes = BASE64
                .decode(encoded)
                .map_err(|e| format!("Grpc.decode_raw base64 decode failed: {}", e))?;
            let row = proto_bytes_to_map(&type_name, &bytes, type_metas)?;
            Ok(VMValue::Record(
                row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
            ))
        }
        "Grpc.call_raw" => {
            if args.len() != 3 {
                return Err("Grpc.call_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let host = vm_string(it.next().unwrap(), "Grpc.call_raw host")?;
            let method = vm_string(it.next().unwrap(), "Grpc.call_raw method")?;
            let payload = match it.next().unwrap() {
                VMValue::Record(map) => schema_record_to_string_map(&map),
                other => {
                    return Err(format!(
                        "Grpc.call_raw expects Map<String,String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let proto_bytes = string_map_to_proto_bytes(&payload);
            let frame = encode_grpc_frame(&proto_bytes);
            let tcp_addr = grpc_tcp_addr(&host);
            let uri_str = grpc_method_uri(&host, &method);
            let result = std::thread::spawn(move || -> Result<VMValue, String> {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| format!("Grpc.call_raw tokio build failed: {}", e))?;
                rt.block_on(async move {
                    let tcp = match tokio::net::TcpStream::connect(&tcp_addr).await {
                        Ok(s) => s,
                        Err(e) => {
                            return Ok(err_vm(rpc_error_vm(
                                14,
                                format!("connection failed: {}", e),
                            )))
                        }
                    };
                    let (mut h2_client, h2_conn) =
                        match h2::client::handshake(tcp).await {
                            Ok(r) => r,
                            Err(e) => {
                                return Ok(err_vm(rpc_error_vm(
                                    14,
                                    format!("h2 handshake failed: {}", e),
                                )))
                            }
                        };
                    tokio::spawn(async move { let _ = h2_conn.await; });
                    let request = match http::Request::builder()
                        .method("POST")
                        .uri(uri_str.as_str())
                        .header("content-type", "application/grpc")
                        .header("te", "trailers")
                        .body(())
                    {
                        Ok(r) => r,
                        Err(e) => return Err(format!("request build failed: {}", e)),
                    };
                    let (response_future, mut send_stream) =
                        match h2_client.send_request(request, false) {
                            Ok(r) => r,
                            Err(e) => {
                                return Ok(err_vm(rpc_error_vm(
                                    14,
                                    format!("send_request failed: {}", e),
                                )))
                            }
                        };
                    if let Err(e) =
                        send_stream.send_data(Bytes::from(frame), true)
                    {
                        return Ok(err_vm(rpc_error_vm(
                            14,
                            format!("send_data failed: {}", e),
                        )));
                    }
                    let response = match response_future.await {
                        Ok(r) => r,
                        Err(e) => {
                            return Ok(err_vm(rpc_error_vm(
                                14,
                                format!("response failed: {}", e),
                            )))
                        }
                    };
                    if !response.status().is_success() {
                        return Ok(err_vm(rpc_error_vm(
                            14,
                            format!("HTTP {}", response.status()),
                        )));
                    }
                    let mut body = response.into_body();
                    let mut resp_bytes: Vec<u8> = Vec::new();
                    while let Some(chunk) = body.data().await {
                        match chunk {
                            Ok(data) => {
                                let n = data.len();
                                resp_bytes.extend_from_slice(&data);
                                let _ = body.flow_control().release_capacity(n);
                            }
                            Err(e) => {
                                return Ok(err_vm(rpc_error_vm(
                                    14,
                                    format!("body read failed: {}", e),
                                )))
                            }
                        }
                    }
                    // Check gRPC status from trailers
                    if let Ok(Some(trailers)) = body.trailers().await {
                        if let Some(grpc_status) = trailers.get("grpc-status") {
                            if grpc_status.as_bytes() != b"0" {
                                let msg = trailers
                                    .get("grpc-message")
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("gRPC error")
                                    .to_string();
                                let code: i64 = grpc_status
                                    .to_str()
                                    .ok()
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or(2);
                                return Ok(err_vm(rpc_error_vm(code, msg)));
                            }
                        }
                    }
                    let proto =
                        match decode_grpc_frame(&resp_bytes) {
                            Ok(b) => b,
                            Err(e) => {
                                return Err(format!("decode_grpc_frame failed: {}", e))
                            }
                        };
                    let row = match proto_bytes_to_string_map(&proto) {
                        Ok(m) => m,
                        Err(e) => {
                            return Err(format!(
                                "proto_bytes_to_string_map failed: {}",
                                e
                            ))
                        }
                    };
                    Ok(ok_vm(VMValue::Record(
                        row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
                    )))
                })
            })
            .join()
            .map_err(|_| "Grpc.call_raw thread panicked".to_string())??;
            Ok(result)
        }
        "Grpc.call_stream_raw" => {
            if args.len() != 3 {
                return Err("Grpc.call_stream_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let host = vm_string(it.next().unwrap(), "Grpc.call_stream_raw host")?;
            let method = vm_string(it.next().unwrap(), "Grpc.call_stream_raw method")?;
            let payload = match it.next().unwrap() {
                VMValue::Record(map) => schema_record_to_string_map(&map),
                other => {
                    return Err(format!(
                        "Grpc.call_stream_raw expects Map<String,String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let proto_bytes = string_map_to_proto_bytes(&payload);
            let frame = encode_grpc_frame(&proto_bytes);
            let tcp_addr = grpc_tcp_addr(&host);
            let uri_str = grpc_method_uri(&host, &method);
            let result = std::thread::spawn(move || -> Result<VMValue, String> {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| format!("Grpc.call_stream_raw tokio build failed: {}", e))?;
                rt.block_on(async move {
                    let tcp = match tokio::net::TcpStream::connect(&tcp_addr).await {
                        Ok(s) => s,
                        Err(_) => return Ok(VMValue::List(vec![])),
                    };
                    let (mut h2_client, h2_conn) =
                        match h2::client::handshake(tcp).await {
                            Ok(r) => r,
                            Err(_) => return Ok(VMValue::List(vec![])),
                        };
                    tokio::spawn(async move { let _ = h2_conn.await; });
                    let request = match http::Request::builder()
                        .method("POST")
                        .uri(uri_str.as_str())
                        .header("content-type", "application/grpc")
                        .header("te", "trailers")
                        .body(())
                    {
                        Ok(r) => r,
                        Err(_) => return Ok(VMValue::List(vec![])),
                    };
                    let (response_future, mut send_stream) =
                        match h2_client.send_request(request, false) {
                            Ok(r) => r,
                            Err(_) => return Ok(VMValue::List(vec![])),
                        };
                    if send_stream
                        .send_data(Bytes::from(frame), true)
                        .is_err()
                    {
                        return Ok(VMValue::List(vec![]));
                    }
                    let response = match response_future.await {
                        Ok(r) => r,
                        Err(_) => return Ok(VMValue::List(vec![])),
                    };
                    let mut body = response.into_body();
                    let mut resp_bytes: Vec<u8> = Vec::new();
                    while let Some(chunk) = body.data().await {
                        match chunk {
                            Ok(data) => {
                                let n = data.len();
                                resp_bytes.extend_from_slice(&data);
                                let _ = body.flow_control().release_capacity(n);
                            }
                            Err(_) => return Ok(VMValue::List(vec![])),
                        }
                    }
                    let rows = decode_all_grpc_frames(&resp_bytes)?
                        .into_iter()
                        .map(|bytes| {
                            proto_bytes_to_string_map(&bytes).map(|row| {
                                VMValue::Record(
                                    row.into_iter()
                                        .map(|(k, v)| (k, VMValue::Str(v)))
                                        .collect(),
                                )
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(VMValue::List(rows))
                })
            })
            .join()
            .map_err(|_| "Grpc.call_stream_raw thread panicked".to_string())??;
            Ok(result)
        }
        "File.read" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => {
                    return Err(format!(
                        "File.read expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("File.read requires 1 argument".to_string()),
            };
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("File.read failed for `{}`: {}", path, e))?;
            Ok(VMValue::Str(content))
        }
        "File.read_lines" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => {
                    return Err(format!(
                        "File.read_lines expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("File.read_lines requires 1 argument".to_string()),
            };
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("File.read_lines failed for `{}`: {}", path, e))?;
            Ok(VMValue::List(
                content
                    .lines()
                    .map(|line| VMValue::Str(line.to_string()))
                    .collect(),
            ))
        }
        "File.write" => {
            if args.len() != 2 {
                return Err("File.write requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "File.write expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let content = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "File.write expects String content, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            std::fs::write(&path, content)
                .map_err(|e| format!("File.write failed for `{}`: {}", path, e))?;
            Ok(VMValue::Unit)
        }
        "File.write_lines" => {
            if args.len() != 2 {
                return Err("File.write_lines requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "File.write_lines expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let lines = match it.next().unwrap() {
                VMValue::List(items) => {
                    let mut parts = Vec::with_capacity(items.len());
                    for item in items {
                        match item {
                            VMValue::Str(s) => parts.push(s),
                            other => {
                                return Err(format!(
                                    "File.write_lines expects List<String>, got List<{}>",
                                    vmvalue_type_name(&other)
                                ));
                            }
                        }
                    }
                    parts
                }
                other => {
                    return Err(format!(
                        "File.write_lines expects List<String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            std::fs::write(&path, lines.join("\n"))
                .map_err(|e| format!("File.write_lines failed for `{}`: {}", path, e))?;
            Ok(VMValue::Unit)
        }
        "File.append" => {
            use std::io::Write;
            if args.len() != 2 {
                return Err("File.append requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "File.append expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let content = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "File.append expects String content, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| format!("File.append failed for `{}`: {}", path, e))?;
            file.write_all(content.as_bytes())
                .map_err(|e| format!("File.append failed for `{}`: {}", path, e))?;
            Ok(VMValue::Unit)
        }
        "File.exists" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => {
                    return Err(format!(
                        "File.exists expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("File.exists requires 1 argument".to_string()),
            };
            Ok(VMValue::Bool(std::path::Path::new(&path).exists()))
        }
        "File.delete" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => {
                    return Err(format!(
                        "File.delete expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("File.delete requires 1 argument".to_string()),
            };
            match std::fs::remove_file(&path) {
                Ok(_) => Ok(VMValue::Unit),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(VMValue::Unit),
                Err(e) => Err(format!("File.delete failed for `{}`: {}", path, e)),
            }
        }
        // Task builtins (v1.7.0) — synchronous-only implementation
        // Task<T> is transparent at runtime: the value IS the T.
        "Task.run" => {
            // Task.run(t) — returns the task value immediately
            match args.into_iter().next() {
                Some(v) => Ok(v),
                None => Err("Task.run requires 1 argument".to_string()),
            }
        }
        "Task.map" => {
            // Task.map(task_val, f) — f(task_val)
            let mut it = args.into_iter();
            match (it.next(), it.next()) {
                (Some(val), Some(f)) => {
                    match f {
                        VMValue::CompiledFn(_) | VMValue::Closure(_, _) => Err(
                            "Task.map: function calling not supported in builtin context"
                                .to_string(),
                        ),
                        _ => Ok(val), // identity if f is not callable here
                    }
                }
                _ => Err("Task.map requires 2 arguments".to_string()),
            }
        }
        "Task.and_then" => {
            // Task.and_then(task_val, f) — same as Task.map for synchronous tasks
            match args.into_iter().next() {
                Some(v) => Ok(v),
                None => Err("Task.and_then requires 2 arguments".to_string()),
            }
        }
        // Task parallel API (v1.8.0) — synchronous transparent implementation
        "Task.all" => {
            // Task.all(list_of_tasks) — runs all, returns List of results
            // v1.8.0: Tasks are transparent values, so this is identity on the list.
            match args.into_iter().next() {
                Some(VMValue::List(items)) => {
                    if items.is_empty() {
                        return Err("E061: Task.all requires a non-empty list".to_string());
                    }
                    Ok(VMValue::List(items))
                }
                Some(other) => Err(format!("Task.all: expected List, got {:?}", other)),
                None => Err("Task.all requires 1 argument (a List of tasks)".to_string()),
            }
        }
        "Task.race" => {
            // Task.race(list_of_tasks) — returns the first task's result
            // v1.8.0: returns head element (no true parallelism).
            match args.into_iter().next() {
                Some(VMValue::List(mut items)) => {
                    if items.is_empty() {
                        return Err("E061: Task.race requires a non-empty list".to_string());
                    }
                    Ok(items.remove(0))
                }
                Some(other) => Err(format!("Task.race: expected List, got {:?}", other)),
                None => Err("Task.race requires 1 argument (a List of tasks)".to_string()),
            }
        }
        "Task.timeout" => {
            // Task.timeout(task, ms) — v1.8.0: always Some(value), no real timeout.
            let mut it = args.into_iter();
            match (it.next(), it.next()) {
                (Some(val), Some(VMValue::Int(_ms))) => {
                    Ok(VMValue::Variant("some".into(), Some(Box::new(val))))
                }
                (Some(val), None) => Ok(VMValue::Variant("some".into(), Some(Box::new(val)))),
                _ => {
                    Err("Task.timeout requires 2 arguments: task and timeout_ms (Int)".to_string())
                }
            }
        }
        // Random builtins (v2.8.0) — updated v3.5.0 to support seeded RNG
        "Random.int" => {
            let mut it = args.into_iter();
            let min_val = it
                .next()
                .ok_or_else(|| "Random.int requires 2 arguments".to_string())?;
            let max_val = it
                .next()
                .ok_or_else(|| "Random.int requires 2 arguments".to_string())?;
            match (min_val, max_val) {
                (VMValue::Int(lo), VMValue::Int(hi)) => Ok(VMValue::Int(seeded_rand_int(lo, hi))),
                _ => Err("Random.int requires (Int, Int)".to_string()),
            }
        }
        "Random.float" => Ok(VMValue::Float(seeded_rand_float())),
        // Random.seed (v3.5.0)
        "Random.seed" => {
            let n = vm_int(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Random.seed requires 1 argument".to_string())?,
                "Random.seed",
            )?;
            use rand::SeedableRng;
            SEEDED_RNG.with(|r| {
                *r.borrow_mut() = Some(rand::rngs::SmallRng::seed_from_u64(n as u64));
            });
            Ok(VMValue::Unit)
        }

        // ── Gen.* (v3.5.0) ─────────────────────────────────────────────────
        "Gen.string_val" => {
            let len = vm_int(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Gen.string_val requires 1 argument".to_string())?,
                "Gen.string_val",
            )? as usize;
            Ok(VMValue::Str(random_alphanumeric_string(len)))
        }
        "Gen.one_raw" => {
            let type_name = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Gen.one_raw requires 1 argument".to_string())?,
                "Gen.one_raw",
            )?;
            gen_one_row(&type_name, type_metas)
        }
        "Gen.list_raw" => {
            let mut it = args.into_iter();
            let type_name = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.list_raw requires 2 arguments".to_string())?,
                "Gen.list_raw",
            )?;
            let n = vm_int(
                it.next()
                    .ok_or_else(|| "Gen.list_raw requires 2 arguments".to_string())?,
                "Gen.list_raw",
            )? as usize;
            let rows: Result<Vec<VMValue>, String> = (0..n)
                .map(|_| gen_one_row(&type_name, type_metas))
                .collect();
            Ok(VMValue::List(rows?))
        }
        "Gen.simulate_raw" => {
            let mut it = args.into_iter();
            let type_name = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.simulate_raw requires 3 arguments".to_string())?,
                "Gen.simulate_raw",
            )?;
            let n = vm_int(
                it.next()
                    .ok_or_else(|| "Gen.simulate_raw requires 3 arguments".to_string())?,
                "Gen.simulate_raw",
            )? as usize;
            let noise = vm_float(
                it.next()
                    .ok_or_else(|| "Gen.simulate_raw requires 3 arguments".to_string())?,
                "Gen.simulate_raw",
            )?;
            let meta = type_metas
                .get(&type_name)
                .ok_or_else(|| format!("Gen.simulate_raw: unknown type '{type_name}'"))?;
            let noise_thresh = (noise * 1000.0) as i64;
            let rows: Result<Vec<VMValue>, String> = (0..n)
                .map(|_| {
                    let mut map = HashMap::new();
                    for field in &meta.fields {
                        let corrupt = seeded_rand_int(0, 999) < noise_thresh;
                        let val = if corrupt {
                            gen_corrupt_value(&field.ty)
                        } else {
                            gen_value_for_type(&field.ty)
                        };
                        map.insert(field.name.clone(), VMValue::Str(val));
                    }
                    Ok(VMValue::Record(map))
                })
                .collect();
            Ok(VMValue::List(rows?))
        }
        "Gen.profile_raw" => {
            let mut it = args.into_iter();
            let type_name = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.profile_raw requires 2 arguments".to_string())?,
                "Gen.profile_raw",
            )?;
            let data_val = it
                .next()
                .ok_or_else(|| "Gen.profile_raw requires 2 arguments".to_string())?;
            let rows = match data_val {
                VMValue::List(rows) => rows,
                _ => return Err("Gen.profile_raw: second argument must be a list".to_string()),
            };
            let meta = type_metas
                .get(&type_name)
                .ok_or_else(|| format!("Gen.profile_raw: unknown type '{type_name}'"))?;
            let total = rows.len();
            let valid = rows
                .iter()
                .filter(|row| {
                    if let VMValue::Record(map) = row {
                        meta.fields.iter().all(|field| {
                            let val = map
                                .get(&field.name)
                                .and_then(|v| {
                                    if let VMValue::Str(s) = v {
                                        Some(s.as_str())
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or("");
                            is_valid_for_type(val, &field.ty)
                        })
                    } else {
                        false
                    }
                })
                .count();
            let invalid = total - valid;
            let rate = if total > 0 {
                valid as f64 / total as f64
            } else {
                0.0
            };
            let mut profile_map = HashMap::new();
            profile_map.insert("total".to_string(), VMValue::Int(total as i64));
            profile_map.insert("valid".to_string(), VMValue::Int(valid as i64));
            profile_map.insert("invalid".to_string(), VMValue::Int(invalid as i64));
            profile_map.insert("rate".to_string(), VMValue::Float(rate));
            Ok(VMValue::Record(profile_map))
        }

        // ── DB.* (v3.3.0) ──────────────────────────────────────────────────
        "DB.connect" => {
            if args.len() != 1 {
                return Err("DB.connect requires 1 argument".to_string());
            }
            let conn_str = vm_string(args.into_iter().next().unwrap(), "DB.connect")?;
            let conn = if conn_str == "sqlite::memory:" {
                rusqlite::Connection::open_in_memory()
                    .map_err(|e| format!("E0601: db connection failed: {}", e))?
            } else if let Some(path) = conn_str.strip_prefix("sqlite:") {
                rusqlite::Connection::open(path)
                    .map_err(|e| format!("E0601: db connection failed: {}", e))?
            } else if conn_str.starts_with("postgres://") {
                return Ok(err_vm(db_error_vm(
                    "E0605",
                    "db driver unsupported: postgres not compiled in (enable feature 'postgres_integration')",
                )));
            } else {
                return Ok(err_vm(db_error_vm(
                    "E0605",
                    &format!("db driver unsupported: unknown scheme in '{}'", conn_str),
                )));
            };
            let id = DB_NEXT_ID.with(|c| {
                let id = c.get();
                c.set(id + 1);
                id
            });
            DB_CONNECTIONS.with(|store| {
                store
                    .borrow_mut()
                    .insert(id, DbConnWrapper { conn, in_tx: false });
            });
            Ok(ok_vm(VMValue::DbHandle(id)))
        }

        "DB.close" => {
            if args.len() != 1 {
                return Err("DB.close requires 1 argument".to_string());
            }
            match args.into_iter().next().unwrap() {
                VMValue::DbHandle(id) => {
                    DB_CONNECTIONS.with(|store| {
                        store.borrow_mut().remove(&id);
                    });
                    Ok(VMValue::Unit)
                }
                other => Err(format!(
                    "DB.close expects DbHandle, got {}",
                    vmvalue_type_name(&other)
                )),
            }
        }

        "DB.query_raw" => {
            if args.len() != 2 {
                return Err("DB.query_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.query_raw expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.query_raw")?;
            let rows = DB_CONNECTIONS.with(|store| -> Result<Vec<VMValue>, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&handle_id)
                    .ok_or_else(|| "DB.query_raw: invalid DbHandle".to_string())?;
                sqlite_query_raw(&wrapper.conn, &sql)
            })?;
            Ok(ok_vm(VMValue::List(rows)))
        }

        "DB.execute_raw" => {
            if args.len() != 2 {
                return Err("DB.execute_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.execute_raw expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.execute_raw")?;
            let n = DB_CONNECTIONS.with(|store| -> Result<i64, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&handle_id)
                    .ok_or_else(|| "DB.execute_raw: invalid DbHandle".to_string())?;
                wrapper
                    .conn
                    .execute(&sql, [])
                    .map(|n| n as i64)
                    .map_err(|e| format!("E0602: db query failed: {}", e))
            })?;
            Ok(ok_vm(VMValue::Int(n)))
        }

        "DB.query_raw_params" => {
            if args.len() != 3 {
                return Err("DB.query_raw_params requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.query_raw_params expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.query_raw_params")?;
            let params = match it.next().unwrap() {
                VMValue::List(v) => v
                    .into_iter()
                    .map(|p| vm_string(p, "DB.query_raw_params param"))
                    .collect::<Result<Vec<_>, _>>()?,
                other => {
                    return Err(format!(
                        "DB.query_raw_params: params must be List<String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let rows = DB_CONNECTIONS.with(|store| -> Result<Vec<VMValue>, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&handle_id)
                    .ok_or_else(|| "DB.query_raw_params: invalid DbHandle".to_string())?;
                sqlite_query_raw_params(&wrapper.conn, &sql, &params)
            })?;
            Ok(ok_vm(VMValue::List(rows)))
        }

        "DB.execute_raw_params" => {
            if args.len() != 3 {
                return Err("DB.execute_raw_params requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.execute_raw_params expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.execute_raw_params")?;
            let params = match it.next().unwrap() {
                VMValue::List(v) => v
                    .into_iter()
                    .map(|p| vm_string(p, "DB.execute_raw_params param"))
                    .collect::<Result<Vec<_>, _>>()?,
                other => {
                    return Err(format!(
                        "DB.execute_raw_params: params must be List<String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let n = DB_CONNECTIONS.with(|store| -> Result<i64, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&handle_id)
                    .ok_or_else(|| "DB.execute_raw_params: invalid DbHandle".to_string())?;
                let param_refs: Vec<&dyn rusqlite::ToSql> =
                    params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                wrapper
                    .conn
                    .execute(&sql, param_refs.as_slice())
                    .map(|n| n as i64)
                    .map_err(|e| format!("E0602: db query failed: {}", e))
            })?;
            Ok(ok_vm(VMValue::Int(n)))
        }

        "DB.upsert_raw" => {
            if args.len() != 4 {
                return Err("DB.upsert_raw requires 4 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.upsert_raw expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let table_name = vm_string(it.next().unwrap(), "DB.upsert_raw type_name")?;
            let row = match it.next().unwrap() {
                VMValue::Record(map) => map
                    .into_iter()
                    .map(|(k, v)| Ok((k, vm_string(v, "DB.upsert_raw row value")?)))
                    .collect::<Result<HashMap<_, _>, String>>()?,
                other => {
                    return Err(format!(
                        "DB.upsert_raw expects Map<String,String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let key_field = vm_string(it.next().unwrap(), "DB.upsert_raw key_field")?;
            if row.is_empty() {
                return Ok(VMValue::Unit);
            }
            let mut columns: Vec<String> = row.keys().cloned().collect();
            columns.sort();
            if !columns.iter().any(|c| c == &key_field) {
                return Err(format!(
                    "DB.upsert_raw key field `{}` is missing from row",
                    key_field
                ));
            }
            let placeholders = (1..=columns.len())
                .map(|idx| format!("?{}", idx))
                .collect::<Vec<_>>()
                .join(", ");
            let assignments = columns
                .iter()
                .filter(|c| *c != &key_field)
                .map(|c| format!("{c} = excluded.{c}"))
                .collect::<Vec<_>>()
                .join(", ");
            let sql = if assignments.is_empty() {
                format!(
                    "INSERT OR IGNORE INTO {table_name} ({}) VALUES ({})",
                    columns.join(", "),
                    placeholders
                )
            } else {
                format!(
                    "INSERT INTO {table_name} ({}) VALUES ({}) ON CONFLICT({key_field}) DO UPDATE SET {}",
                    columns.join(", "),
                    placeholders,
                    assignments
                )
            };
            let values: Vec<String> = columns
                .iter()
                .map(|c| row.get(c).cloned().unwrap_or_default())
                .collect();
            Ok(DB_CONNECTIONS.with(|store| -> Result<VMValue, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&handle_id)
                    .ok_or_else(|| "DB.upsert_raw: invalid DbHandle".to_string())?;
                let param_refs: Vec<&dyn rusqlite::ToSql> =
                    values.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                wrapper
                    .conn
                    .execute(&sql, param_refs.as_slice())
                    .map_err(|e| format!("E0602: db query failed: {}", e))?;
                Ok(VMValue::Unit)
            })?)
        }

        "DB.begin_tx" => {
            if args.len() != 1 {
                return Err("DB.begin_tx requires 1 argument".to_string());
            }
            let handle_id = match args.into_iter().next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.begin_tx expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            DB_CONNECTIONS.with(|store| -> Result<VMValue, String> {
                let mut store = store.borrow_mut();
                let wrapper = store
                    .get_mut(&handle_id)
                    .ok_or_else(|| "DB.begin_tx: invalid DbHandle".to_string())?;
                if wrapper.in_tx {
                    return Ok(err_vm(db_error_vm(
                        "E0603",
                        "db transaction failed: already in transaction",
                    )));
                }
                wrapper
                    .conn
                    .execute_batch("BEGIN")
                    .map_err(|e| format!("E0603: db transaction failed: {}", e))?;
                wrapper.in_tx = true;
                Ok(ok_vm(VMValue::TxHandle(handle_id)))
            })
        }

        "DB.commit_tx" => {
            if args.len() != 1 {
                return Err("DB.commit_tx requires 1 argument".to_string());
            }
            let tx_id = match args.into_iter().next().unwrap() {
                VMValue::TxHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.commit_tx expects TxHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            DB_CONNECTIONS.with(|store| -> Result<VMValue, String> {
                let mut store = store.borrow_mut();
                let wrapper = store
                    .get_mut(&tx_id)
                    .ok_or_else(|| "DB.commit_tx: invalid TxHandle".to_string())?;
                wrapper
                    .conn
                    .execute_batch("COMMIT")
                    .map_err(|e| format!("E0603: db transaction failed: {}", e))?;
                wrapper.in_tx = false;
                Ok(ok_vm(VMValue::Unit))
            })
        }

        "DB.rollback_tx" => {
            if args.len() != 1 {
                return Err("DB.rollback_tx requires 1 argument".to_string());
            }
            let tx_id = match args.into_iter().next().unwrap() {
                VMValue::TxHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.rollback_tx expects TxHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            DB_CONNECTIONS.with(|store| -> Result<VMValue, String> {
                let mut store = store.borrow_mut();
                let wrapper = store
                    .get_mut(&tx_id)
                    .ok_or_else(|| "DB.rollback_tx: invalid TxHandle".to_string())?;
                wrapper
                    .conn
                    .execute_batch("ROLLBACK")
                    .map_err(|e| format!("E0603: db transaction failed: {}", e))?;
                wrapper.in_tx = false;
                Ok(ok_vm(VMValue::Unit))
            })
        }

        "DB.query_in_tx" => {
            if args.len() != 2 {
                return Err("DB.query_in_tx requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let tx_id = match it.next().unwrap() {
                VMValue::TxHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.query_in_tx expects TxHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.query_in_tx")?;
            let rows = DB_CONNECTIONS.with(|store| -> Result<Vec<VMValue>, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&tx_id)
                    .ok_or_else(|| "DB.query_in_tx: invalid TxHandle".to_string())?;
                sqlite_query_raw(&wrapper.conn, &sql)
            })?;
            Ok(ok_vm(VMValue::List(rows)))
        }

        "DB.execute_in_tx" => {
            if args.len() != 2 {
                return Err("DB.execute_in_tx requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let tx_id = match it.next().unwrap() {
                VMValue::TxHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.execute_in_tx expects TxHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.execute_in_tx")?;
            let n = DB_CONNECTIONS.with(|store| -> Result<i64, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&tx_id)
                    .ok_or_else(|| "DB.execute_in_tx: invalid TxHandle".to_string())?;
                wrapper
                    .conn
                    .execute(&sql, [])
                    .map(|n| n as i64)
                    .map_err(|e| format!("E0602: db query failed: {}", e))
            })?;
            Ok(ok_vm(VMValue::Int(n)))
        }

        // ── Env.* (v3.3.0) ─────────────────────────────────────────────────
        "Env.get" => {
            if args.len() != 1 {
                return Err("Env.get requires 1 argument".to_string());
            }
            let name = vm_string(args.into_iter().next().unwrap(), "Env.get")?;
            match std::env::var(&name) {
                Ok(val) => Ok(ok_vm(VMValue::Str(val))),
                Err(_) => Ok(err_vm(db_error_vm(
                    "E0001",
                    &format!("environment variable '{}' not found", name),
                ))),
            }
        }

        "Env.get_or" => {
            if args.len() != 2 {
                return Err("Env.get_or requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let name = vm_string(it.next().unwrap(), "Env.get_or")?;
            let default = vm_string(it.next().unwrap(), "Env.get_or default")?;
            Ok(VMValue::Str(std::env::var(&name).unwrap_or(default)))
        }

        "Checkpoint.last" => {
            if args.len() != 1 {
                return Err("Checkpoint.last requires 1 argument".to_string());
            }
            let name = vm_string(args.into_iter().next().unwrap(), "Checkpoint.last")?;
            match checkpoint_last_impl(&name)? {
                Some(value) => Ok(VMValue::Variant(
                    "some".to_string(),
                    Some(Box::new(VMValue::Str(value))),
                )),
                None => Ok(VMValue::Variant("none".to_string(), None)),
            }
        }

        "Checkpoint.save" => {
            if args.len() != 2 {
                return Err("Checkpoint.save requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let name = vm_string(it.next().unwrap(), "Checkpoint.save")?;
            let value = vm_string(it.next().unwrap(), "Checkpoint.save value")?;
            checkpoint_save_impl(&name, &value)?;
            Ok(VMValue::Unit)
        }

        "Checkpoint.reset" => {
            if args.len() != 1 {
                return Err("Checkpoint.reset requires 1 argument".to_string());
            }
            let name = vm_string(args.into_iter().next().unwrap(), "Checkpoint.reset")?;
            checkpoint_reset_impl(&name)?;
            Ok(VMValue::Unit)
        }

        "Checkpoint.meta" => {
            if args.len() != 1 {
                return Err("Checkpoint.meta requires 1 argument".to_string());
            }
            let name = vm_string(args.into_iter().next().unwrap(), "Checkpoint.meta")?;
            let meta = checkpoint_meta_impl(&name)?;
            let mut map = HashMap::new();
            map.insert("name".to_string(), VMValue::Str(meta.name));
            map.insert("value".to_string(), VMValue::Str(meta.value));
            map.insert("updated_at".to_string(), VMValue::Str(meta.updated_at));
            Ok(VMValue::Record(map))
        }

        "Parquet.write_raw" => {
            if args.len() != 3 {
                return Err("Parquet.write_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = vm_string(it.next().unwrap(), "Parquet.write_raw path")?;
            let type_name = vm_string(it.next().unwrap(), "Parquet.write_raw type_name")?;
            let rows = schema_rows_from_vm(it.next().unwrap(), "Parquet.write_raw")?;
            match parquet_write_rows(&path, &type_name, rows, type_metas) {
                Ok(()) => Ok(ok_vm(VMValue::Unit)),
                Err(err) => Ok(err_vm(parquet_error_vm(err))),
            }
        }

        "Parquet.read_raw" => {
            if args.len() != 1 {
                return Err("Parquet.read_raw requires 1 argument".to_string());
            }
            let path = vm_string(args.into_iter().next().unwrap(), "Parquet.read_raw path")?;
            match parquet_read_rows(&path) {
                Ok(rows) => Ok(ok_vm(VMValue::List(
                    rows.into_iter().map(VMValue::Record).collect(),
                ))),
                Err(err) => Ok(err_vm(parquet_error_vm(err))),
            }
        }

        other => Err(format!("unknown builtin: {}", other)),
    }
}

#[cfg(test)]
#[path = "vm_legacy_coverage_tests.rs"]
mod vm_legacy_coverage_tests;

#[cfg(test)]
#[path = "vm_stdlib_tests.rs"]
mod vm_stdlib_tests;

#[cfg(test)]
mod wasm_phase0_builtin_tests {
    use super::{
        SuppressIoGuard, VMValue, io_output_suppressed_for_tests, set_suppress_io, vm_call_builtin,
    };

    #[test]
    fn vm_builtin_io_print_variants_return_unit() {
        let mut emit_log = Vec::new();
        assert_eq!(
            vm_call_builtin(
                "IO.print",
                vec![VMValue::Str("hello".into())],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Unit
        );
        assert_eq!(
            vm_call_builtin(
                "IO.println_int",
                vec![VMValue::Int(42)],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Unit
        );
        assert_eq!(
            vm_call_builtin(
                "IO.println_float",
                vec![VMValue::Float(3.5)],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Unit
        );
        assert_eq!(
            vm_call_builtin(
                "IO.println_bool",
                vec![VMValue::Bool(true)],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Unit
        );
    }

    #[test]
    fn vm_builtin_string_state_helpers() {
        let mut emit_log = Vec::new();
        assert_eq!(
            vm_call_builtin(
                "String.is_url",
                vec![VMValue::Str("https://example.com".into())],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Bool(true)
        );
        assert_eq!(
            vm_call_builtin(
                "String.is_url",
                vec![VMValue::Str("ftp://example.com".into())],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Bool(false)
        );
        assert_eq!(
            vm_call_builtin(
                "String.is_slug",
                vec![VMValue::Str("hello-world-2026".into())],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Bool(true)
        );
        assert_eq!(
            vm_call_builtin(
                "String.is_slug",
                vec![VMValue::Str("Hello world".into())],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Bool(false)
        );
    }

    #[test]
    fn suppress_io_guard_restores_previous_state() {
        set_suppress_io(false);
        assert!(!io_output_suppressed_for_tests());
        {
            let _guard = SuppressIoGuard::new(true);
            assert!(io_output_suppressed_for_tests());
        }
        assert!(!io_output_suppressed_for_tests());
    }
}
