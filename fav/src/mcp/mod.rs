#![allow(dead_code)]

use std::io::{self, BufRead, Write};

use crate::ast;
use crate::backend::codegen::codegen_program;
use crate::backend::vm::VM;
use crate::frontend::parser::Parser;
use crate::middle::checker::Checker;
use crate::middle::compiler::compile_program;

// ── request type ──────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct McpRequest {
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

// ── server ────────────────────────────────────────────────────────────────────

pub struct McpServer<W: Write> {
    writer: W,
}

impl<W: Write> McpServer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn handle(&mut self, req: McpRequest) {
        match req.method.as_str() {
            "initialize" => self.handle_initialize(req.id),
            "initialized" | "$/cancelRequest" => {}
            "tools/list" => self.handle_tools_list(req.id),
            "tools/call" => self.handle_tools_call(req.id, &req.params),
            "resources/list" => self.handle_resources_list(req.id),
            "resources/read" => self.handle_resources_read(req.id, &req.params),
            "prompts/list" => self.handle_prompts_list(req.id),
            "prompts/get" => self.handle_prompts_get(req.id, &req.params),
            "shutdown" => self.send_result(req.id, serde_json::Value::Null),
            "exit" => std::process::exit(0),
            _ => {
                if let Some(id) = req.id {
                    self.send_error(id, -32601, "Method not found");
                }
            }
        }
    }

    fn handle_initialize(&mut self, id: Option<serde_json::Value>) {
        self.send_result(
            id,
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "favnir-mcp",
                    "version": "4.9.0"
                }
            }),
        );
    }

    fn handle_tools_list(&mut self, id: Option<serde_json::Value>) {
        self.send_result(id, serde_json::json!({ "tools": tool_definitions() }));
    }

    fn handle_tools_call(&mut self, id: Option<serde_json::Value>, params: &serde_json::Value) {
        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => {
                let id = id.unwrap_or(serde_json::Value::Null);
                self.send_error(id, -32602, "Missing tool name");
                return;
            }
        };
        let args = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let result = match tool_name.as_str() {
            "favnir_check" => {
                let source = args.get("source").and_then(|v| v.as_str()).unwrap_or("");
                tool_favnir_check(source)
            }
            "favnir_run" => {
                let source = args.get("source").and_then(|v| v.as_str()).unwrap_or("");
                tool_favnir_run(source)
            }
            "favnir_test" => {
                let source = args.get("source").and_then(|v| v.as_str()).unwrap_or("");
                tool_favnir_test(source)
            }
            "favnir_list_runes" => tool_favnir_list_runes(),
            "favnir_rune_docs" => {
                let rune = args.get("rune").and_then(|v| v.as_str()).unwrap_or("");
                tool_favnir_rune_docs(rune)
            }
            unknown => tool_error(format!("Unknown tool: {}", unknown)),
        };
        self.send_result(id, result);
    }

    fn handle_resources_list(&mut self, id: Option<serde_json::Value>) {
        self.send_result(id, serde_json::json!({ "resources": resource_list() }));
    }

    fn handle_resources_read(
        &mut self,
        id: Option<serde_json::Value>,
        params: &serde_json::Value,
    ) {
        let uri = match params.get("uri").and_then(|v| v.as_str()) {
            Some(u) => u.to_string(),
            None => {
                let id = id.unwrap_or(serde_json::Value::Null);
                self.send_error(id, -32602, "Missing uri");
                return;
            }
        };
        let result = resource_read(&uri);
        self.send_result(id, result);
    }

    fn handle_prompts_list(&mut self, id: Option<serde_json::Value>) {
        self.send_result(id, serde_json::json!({ "prompts": prompt_list() }));
    }

    fn handle_prompts_get(&mut self, id: Option<serde_json::Value>, params: &serde_json::Value) {
        let name = match params.get("name").and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => {
                let id = id.unwrap_or(serde_json::Value::Null);
                self.send_error(id, -32602, "Missing prompt name");
                return;
            }
        };
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let result = prompt_get(&name, &arguments);
        self.send_result(id, result);
    }

    fn send_result(&mut self, id: Option<serde_json::Value>, result: serde_json::Value) {
        let id = id.unwrap_or(serde_json::Value::Null);
        let _ = write_json_message(
            &mut self.writer,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            }),
        );
    }

    fn send_error(&mut self, id: serde_json::Value, code: i64, message: &str) {
        let _ = write_json_message(
            &mut self.writer,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": code, "message": message }
            }),
        );
    }
}

// ── tool definitions ──────────────────────────────────────────────────────────

fn tool_definitions() -> serde_json::Value {
    serde_json::json!([
        {
            "name": "favnir_check",
            "description": "Type-check a Favnir code snippet without executing it",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source": { "type": "string", "description": "Favnir source code to type-check" }
                },
                "required": ["source"]
            }
        },
        {
            "name": "favnir_run",
            "description": "Compile and execute a Favnir code snippet (must include a main() function)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source": { "type": "string", "description": "Favnir source code to execute" }
                },
                "required": ["source"]
            }
        },
        {
            "name": "favnir_test",
            "description": "Run test blocks in a Favnir source snippet",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source": { "type": "string", "description": "Favnir source with test blocks" }
                },
                "required": ["source"]
            }
        },
        {
            "name": "favnir_list_runes",
            "description": "List all available Rune modules and their exported functions",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "favnir_rune_docs",
            "description": "Get documentation for a specific Rune module",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "rune": { "type": "string", "description": "Rune name (e.g. db, http, log, gen, auth, env)" }
                },
                "required": ["rune"]
            }
        }
    ])
}

// ── tool implementations ──────────────────────────────────────────────────────

fn tool_favnir_check(source: &str) -> serde_json::Value {
    match Parser::parse_str(source, "<mcp>") {
        Err(e) => tool_error(format!("E0500 at line {}: {}", e.span.line, e.message)),
        Ok(program) => {
            let mut checker = Checker::new();
            let (errors, _) = checker.check_with_self(&program);
            if errors.is_empty() {
                tool_text("OK: no type errors")
            } else {
                let msg = errors
                    .iter()
                    .map(|e| format!("{} at line {}: {}", e.code, e.span.line, e.message))
                    .collect::<Vec<_>>()
                    .join("\n");
                tool_error(msg)
            }
        }
    }
}

fn tool_favnir_run(source: &str) -> serde_json::Value {
    let program = match Parser::parse_str(source, "<mcp>") {
        Err(e) => {
            return tool_error(format!(
                "Parse error at line {}: {}",
                e.span.line, e.message
            ))
        }
        Ok(p) => p,
    };
    let mut checker = Checker::new();
    let (errors, _) = checker.check_with_self(&program);
    if !errors.is_empty() {
        let msg = errors
            .iter()
            .map(|e| format!("{} at line {}: {}", e.code, e.span.line, e.message))
            .collect::<Vec<_>>()
            .join("\n");
        return tool_error(format!("Type errors:\n{}", msg));
    }
    let ir = compile_program(&program);
    let artifact = codegen_program(&ir);
    match artifact.fn_idx_by_name("main") {
        None => tool_error("No main() function found in source"),
        Some(fn_idx) => match VM::run(&artifact, fn_idx, vec![]) {
            Ok(_) => tool_text("Executed successfully (stdout output not captured in v4.9.0)"),
            Err(e) => tool_error(format!("Runtime error: {}", e.message)),
        },
    }
}

fn tool_favnir_test(source: &str) -> serde_json::Value {
    let program = match Parser::parse_str(source, "<mcp>") {
        Err(e) => {
            return tool_error(format!(
                "Parse error at line {}: {}",
                e.span.line, e.message
            ))
        }
        Ok(p) => p,
    };
    let mut checker = Checker::new();
    let (errors, _) = checker.check_with_self(&program);
    if !errors.is_empty() {
        let msg = errors
            .iter()
            .map(|e| format!("{} at line {}: {}", e.code, e.span.line, e.message))
            .collect::<Vec<_>>()
            .join("\n");
        return tool_error(format!("Type errors:\n{}", msg));
    }

    let test_names: Vec<String> = program
        .items
        .iter()
        .filter_map(|item| {
            if let ast::Item::TestDef(t) = item {
                Some(t.name.clone())
            } else {
                None
            }
        })
        .collect();

    if test_names.is_empty() {
        return tool_text("No test blocks found");
    }

    let ir = compile_program(&program);
    let artifact = codegen_program(&ir);

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut fail_msgs: Vec<String> = Vec::new();

    for name in &test_names {
        let fn_name = format!("$test:{}", name);
        match artifact.fn_idx_by_name(&fn_name) {
            None => {
                failed += 1;
                fail_msgs.push(format!("  FAIL: \"{}\" — not found in artifact", name));
            }
            Some(fn_idx) => match VM::run(&artifact, fn_idx, vec![]) {
                Ok(_) => passed += 1,
                Err(e) => {
                    failed += 1;
                    fail_msgs.push(format!("  FAIL: \"{}\" — {}", name, e.message));
                }
            },
        }
    }

    let total = passed + failed;
    let mut summary = format!("{} passed, {} failed (of {})", passed, failed, total);
    if !fail_msgs.is_empty() {
        summary.push('\n');
        summary.push_str(&fail_msgs.join("\n"));
    }

    if failed > 0 {
        tool_error(summary)
    } else {
        tool_text(summary)
    }
}

fn tool_favnir_list_runes() -> serde_json::Value {
    let runes = [
        ("db",   "connect, query, execute, query_one, paginate, batch_insert, with_transaction, savepoint, release_savepoint, rollback_to_savepoint, applied_migrations, mark_applied"),
        ("http", "get, post, put, delete, patch, get_with_headers, post_with_headers"),
        ("grpc", "call, call_typed"),
        ("gen",  "one, list, hint_one, hint_list, to_csv, to_parquet, load_into, edge_cases, set_yaml"),
        ("auth", "jwt_sign, jwt_verify, jwt_decode, hmac_sha256, sha256, get_mode"),
        ("env",  "get, get_opt, require, get_int, require_int, get_bool, require_bool, load_dotenv, load_dotenv_or_ignore"),
        ("log",  "info, warn, error, debug, metric, map_to_json"),
    ];
    let text = runes
        .iter()
        .map(|(name, fns)| format!("{}: {}", name, fns))
        .collect::<Vec<_>>()
        .join("\n");
    tool_text(text)
}

fn tool_favnir_rune_docs(rune: &str) -> serde_json::Value {
    let rune_path = std::env::var("FAV_RUNES_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("runes"));
    let barrel = rune_path.join(rune).join(format!("{}.fav", rune));
    if barrel.exists() {
        if let Ok(content) = std::fs::read_to_string(&barrel) {
            return tool_text(format!("## {} Rune\n\n```favnir\n{}\n```", rune, content));
        }
    }
    let docs = match rune {
        "db" => "## db Rune\n\n\
### connect(url: String) -> Result<DbHandle, String> !Io\n\
Establish a database connection.\n\n\
### query(conn: DbHandle, sql: String, params: List<String>) -> Result<List<Map<String,String>>, String> !Io\n\
Execute a SELECT query.\n\n\
### execute(conn: DbHandle, sql: String, params: List<String>) -> Result<Int, String> !Io\n\
Execute an INSERT/UPDATE/DELETE.\n\n\
### query_one(conn: DbHandle, sql: String, params: List<String>) -> Result<Map<String,String>, String> !Io\n\
Execute a SELECT expecting exactly one row.\n\n\
### with_transaction(conn: DbHandle, f: (DbHandle) -> Result<A, String>) -> Result<A, String> !Io\n\
Run f inside a transaction; rollback on error.",
        "http" => "## http Rune\n\n\
### get(url: String) -> Result<String, String> !Io\n\
HTTP GET request.\n\n\
### post(url: String, body: String) -> Result<String, String> !Io\n\
HTTP POST request.\n\n\
### put(url: String, body: String) -> Result<String, String> !Io\n\
HTTP PUT request.\n\n\
### delete(url: String) -> Result<String, String> !Io\n\
HTTP DELETE request.",
        "log" => "## log Rune\n\n\
### info(code: String, message: String, ctx: Map<String,String>) -> Unit !Io\n\
Log at INFO level.\n\n\
### warn(code: String, message: String, ctx: Map<String,String>) -> Unit !Io\n\
Log at WARN level.\n\n\
### error(code: String, message: String, ctx: Map<String,String>) -> Unit !Io\n\
Log at ERROR level.\n\n\
### metric(name: String, value: Float, unit: String) -> Unit !Io\n\
Emit a metric data point.",
        "gen" => "## gen Rune\n\n\
### one(type_name: String) -> Result<Map<String,String>, String>\n\
Generate one synthetic record for the given type.\n\n\
### list(type_name: String, n: Int) -> Result<List<Map<String,String>>, String>\n\
Generate N synthetic records.\n\n\
### hint_one(type_name: String) -> Result<Map<String,String>, String>\n\
Generate a hint-based realistic record using field name patterns.\n\n\
### to_csv(path: String, rows: List<Map<String,String>>) -> Result<Unit, String> !File\n\
Write rows to a CSV file.",
        "auth" => "## auth Rune\n\n\
### jwt_sign(payload: Map<String,String>, secret: String) -> Result<String, String> !Auth\n\
Sign a JWT token.\n\n\
### jwt_verify(token: String, secret: String) -> Result<Bool, String> !Auth\n\
Verify a JWT token signature.\n\n\
### hmac_sha256(message: String, secret: String) -> String !Auth\n\
Compute HMAC-SHA256 hex digest.",
        "env" => "## env Rune\n\n\
### get(key: String, default: String) -> String !Env\n\
Get an environment variable with a fallback default.\n\n\
### require(key: String) -> Result<String, String> !Env\n\
Get an environment variable or return an error.\n\n\
### get_int(key: String, default: Int) -> Int !Env\n\
Get an env var parsed as Int.\n\n\
### load_dotenv(path: String) -> Result<Unit, String> !Env\n\
Load a .env file into the environment.",
        "grpc" => "## grpc Rune\n\n\
### call(host: String, method: String, payload: Map<String,String>) -> Result<Map<String,String>, String> !Io\n\
Call a gRPC method with a Map payload.\n\n\
### call_typed(response_type: String, host: String, method: String, payload: Map<String,String>) -> Result<Map<String,String>, String> !Io\n\
Call a gRPC method and decode the response using the given type name.",
        other => {
            return tool_error(format!(
                "Unknown rune: {}. Available: db, http, grpc, gen, auth, env, log",
                other
            ))
        }
    };
    tool_text(docs)
}

// ── resource implementations ──────────────────────────────────────────────────

fn resource_list() -> serde_json::Value {
    serde_json::json!([
        { "uri": "favnir://docs/stdlib",   "name": "Favnir stdlib docs",  "mimeType": "text/markdown" },
        { "uri": "favnir://runes/db",      "name": "db Rune",             "mimeType": "text/plain" },
        { "uri": "favnir://runes/http",    "name": "http Rune",           "mimeType": "text/plain" },
        { "uri": "favnir://runes/log",     "name": "log Rune",            "mimeType": "text/plain" },
        { "uri": "favnir://runes/gen",     "name": "gen Rune",            "mimeType": "text/plain" },
        { "uri": "favnir://runes/auth",    "name": "auth Rune",           "mimeType": "text/plain" },
        { "uri": "favnir://runes/env",     "name": "env Rune",            "mimeType": "text/plain" },
        { "uri": "favnir://project/files", "name": "Project .fav files",  "mimeType": "text/plain" }
    ])
}

fn resource_read(uri: &str) -> serde_json::Value {
    if uri == "favnir://docs/stdlib" {
        return serde_json::json!({
            "contents": [{
                "uri": uri,
                "mimeType": "text/markdown",
                "text": stdlib_docs()
            }]
        });
    }
    if let Some(rune_name) = uri.strip_prefix("favnir://runes/") {
        let docs_result = tool_favnir_rune_docs(rune_name);
        let text = docs_result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|o| o.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        return serde_json::json!({
            "contents": [{ "uri": uri, "mimeType": "text/plain", "text": text }]
        });
    }
    if uri == "favnir://project/files" {
        let files = collect_project_fav_files();
        return serde_json::json!({
            "contents": [{
                "uri": uri,
                "mimeType": "text/plain",
                "text": files.join("\n")
            }]
        });
    }
    serde_json::json!({
        "error": { "code": -32602, "message": format!("Unknown resource URI: {}", uri) }
    })
}

fn stdlib_docs() -> &'static str {
    "# Favnir Standard Library\n\n\
## String\n\
- `String.contains(s, sub)` → Bool\n\
- `String.split(s, sep)` → List<String>\n\
- `String.concat(a, b)` → String\n\
- `String.length(s)` → Int\n\
- `String.starts_with(s, prefix)` → Bool\n\
- `String.ends_with(s, suffix)` → Bool\n\
- `String.trim(s)` → String\n\
- `String.to_uppercase(s)` → String\n\
- `String.to_lowercase(s)` → String\n\
- `String.replace(s, from, to)` → String\n\
- `String.from_int(n)` → String\n\
- `String.from_bool(b)` → String\n\
\n\
## List\n\
- `List.map(list, f)` → List<B>\n\
- `List.filter(list, f)` → List<A>\n\
- `List.fold(list, init, f)` → B\n\
- `List.any(list, f)` → Bool\n\
- `List.all(list, f)` → Bool\n\
- `List.length(list)` → Int\n\
- `List.append(list, item)` → List<A>\n\
- `List.first(list)` → Option<A>\n\
- `List.last(list)` → Option<A>\n\
- `List.reverse(list)` → List<A>\n\
- `List.zip(a, b)` → List<Tuple<A,B>>\n\
- `List.sort_by(list, f)` → List<A>\n\
\n\
## Map\n\
- `Map.get(map, key)` → Option<V>\n\
- `Map.set(map, key, val)` → Map<K,V>\n\
- `Map.delete(map, key)` → Map<K,V>\n\
- `Map.keys(map)` → List<K>\n\
- `Map.values(map)` → List<V>\n\
- `Map.contains_key(map, key)` → Bool\n\
- `Map.size(map)` → Int\n\
\n\
## Option\n\
- `Option.map(opt, f)` → Option<B>\n\
- `Option.and_then(opt, f)` → Option<B>\n\
- `Option.unwrap_or(opt, default)` → A\n\
- `Option.is_some(opt)` → Bool\n\
- `Option.is_none(opt)` → Bool\n\
\n\
## Result\n\
- `Result.map(res, f)` → Result<B,E>\n\
- `Result.and_then(res, f)` → Result<B,E>\n\
- `Result.unwrap_or(res, default)` → A\n\
- `Result.is_ok(res)` → Bool\n\
- `Result.is_err(res)` → Bool\n\
\n\
## IO\n\
- `IO.println(s)` → Unit !Io\n\
- `IO.print(s)` → Unit !Io\n\
- `IO.read_line()` → String !Io\n\
- `IO.read_file(path)` → Result<String, String> !File\n\
- `IO.write_file(path, content)` → Result<Unit, String> !File\n"
}

fn collect_project_fav_files() -> Vec<String> {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let mut files = Vec::new();
    collect_fav_files_rec(&cwd, &cwd, &mut files);
    files.sort();
    files
}

fn collect_fav_files_rec(
    dir: &std::path::Path,
    root: &std::path::Path,
    out: &mut Vec<String>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') && name != "target" {
                collect_fav_files_rec(&path, root, out);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("fav") {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().to_string());
            }
        }
    }
}

// ── prompt implementations ────────────────────────────────────────────────────

fn prompt_list() -> serde_json::Value {
    serde_json::json!([
        {
            "name": "write_pipeline",
            "description": "Generate a Favnir data pipeline template",
            "arguments": [
                { "name": "source_type", "description": "Input data source type (csv/parquet/db/http)", "required": true },
                { "name": "output_type", "description": "Output destination type (csv/parquet/db/stdout)", "required": true }
            ]
        },
        {
            "name": "fix_type_error",
            "description": "Fix a Favnir type error",
            "arguments": [
                { "name": "source", "description": "Favnir source code with the error", "required": true },
                { "name": "error",  "description": "The error message",                 "required": true }
            ]
        }
    ])
}

fn prompt_get(name: &str, arguments: &serde_json::Value) -> serde_json::Value {
    match name {
        "write_pipeline" => {
            let source_type = arguments
                .get("source_type")
                .and_then(|v| v.as_str())
                .unwrap_or("csv");
            let output_type = arguments
                .get("output_type")
                .and_then(|v| v.as_str())
                .unwrap_or("stdout");
            let text = format!(
                "Write a Favnir pipeline that reads from {} and writes to {}. \
Use appropriate Runes (import rune \"...\") and declare effects (!Io, !File, etc.). \
Include a public fn main() -> Unit !Io entry point.",
                source_type, output_type
            );
            serde_json::json!({
                "description": "Generate a Favnir data pipeline template",
                "messages": [{ "role": "user", "content": { "type": "text", "text": text } }]
            })
        }
        "fix_type_error" => {
            let source = arguments
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let error = arguments
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let text = format!(
                "Fix the following Favnir type error:\n\nCode:\n```favnir\n{}\n```\n\nError:\n{}\n\n\
Explain what caused the error and provide corrected Favnir code.",
                source, error
            );
            serde_json::json!({
                "description": "Fix a Favnir type error",
                "messages": [{ "role": "user", "content": { "type": "text", "text": text } }]
            })
        }
        other => serde_json::json!({
            "error": { "code": -32602, "message": format!("Unknown prompt: {}", other) }
        }),
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn tool_text(text: impl Into<String>) -> serde_json::Value {
    serde_json::json!({ "content": [{ "type": "text", "text": text.into() }] })
}

fn tool_error(text: impl Into<String>) -> serde_json::Value {
    serde_json::json!({ "content": [{ "type": "text", "text": text.into() }], "isError": true })
}

// ── transport ─────────────────────────────────────────────────────────────────

pub fn run_mcp_server() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = io::BufReader::new(stdin.lock());
    let mut server = McpServer::new(stdout.lock());
    while let Some(req) = read_message(&mut reader) {
        server.handle(req);
    }
}

fn read_message(reader: &mut impl BufRead) -> Option<McpRequest> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();
    loop {
        line.clear();
        let bytes = reader.read_line(&mut line).ok()?;
        if bytes == 0 {
            return None;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse::<usize>().ok();
        }
    }
    let len = content_length?;
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice::<McpRequest>(&body).ok()
}

fn write_json_message(writer: &mut impl Write, value: &serde_json::Value) -> io::Result<()> {
    let body =
        serde_json::to_vec(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{McpRequest, McpServer};

    fn last_response(buf: &[u8]) -> serde_json::Value {
        let text = std::str::from_utf8(buf).unwrap_or("");
        if let Some(pos) = text.rfind("\r\n\r\n") {
            serde_json::from_str(&text[pos + 4..]).unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        }
    }

    fn call_tool(name: &str, args: serde_json::Value) -> serde_json::Value {
        let mut out = Vec::new();
        let mut server = McpServer::new(&mut out);
        server.handle(McpRequest {
            id: Some(serde_json::json!(1)),
            method: "tools/call".to_string(),
            params: serde_json::json!({ "name": name, "arguments": args }),
        });
        let resp = last_response(&out);
        resp.get("result").cloned().unwrap_or(serde_json::Value::Null)
    }

    #[test]
    fn initialize_returns_capabilities() {
        let mut out = Vec::new();
        let mut server = McpServer::new(&mut out);
        server.handle(McpRequest {
            id: Some(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: serde_json::json!({}),
        });
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\"protocolVersion\":\"2024-11-05\""), "got: {text}");
        assert!(text.contains("\"tools\":{}"), "got: {text}");
        assert!(text.contains("favnir-mcp"), "got: {text}");
    }

    #[test]
    fn tools_list_returns_five_tools() {
        let mut out = Vec::new();
        let mut server = McpServer::new(&mut out);
        server.handle(McpRequest {
            id: Some(serde_json::json!(1)),
            method: "tools/list".to_string(),
            params: serde_json::json!({}),
        });
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("favnir_check"));
        assert!(text.contains("favnir_run"));
        assert!(text.contains("favnir_test"));
        assert!(text.contains("favnir_list_runes"));
        assert!(text.contains("favnir_rune_docs"));
    }

    #[test]
    fn tools_call_favnir_check_ok() {
        let result = call_tool(
            "favnir_check",
            serde_json::json!({ "source": "fn main() -> Int { 42 }" }),
        );
        let text = result["content"][0]["text"].as_str().unwrap_or("");
        assert!(text.contains("OK: no type errors"), "got: {text}");
        assert!(result.get("isError").is_none());
    }

    #[test]
    fn tools_call_favnir_check_error() {
        let result = call_tool(
            "favnir_check",
            serde_json::json!({ "source": "fn main() -> Int { foo }" }),
        );
        let text = result["content"][0]["text"].as_str().unwrap_or("");
        assert!(text.contains("E0"), "expected error code, got: {text}");
        assert_eq!(result.get("isError"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn tools_call_favnir_run_ok() {
        let result = call_tool(
            "favnir_run",
            serde_json::json!({ "source": "fn main() -> Int { 42 }" }),
        );
        let text = result["content"][0]["text"].as_str().unwrap_or("");
        assert!(
            text.contains("successfully") || text.contains("Executed"),
            "got: {text}"
        );
    }

    #[test]
    fn tools_call_favnir_run_type_error() {
        let result = call_tool(
            "favnir_run",
            serde_json::json!({ "source": "fn main() -> Int { foo }" }),
        );
        assert_eq!(result.get("isError"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn tools_call_favnir_test_ok() {
        let src = "fn add(a: Int, b: Int) -> Int { a + b }\ntest \"add works\" { assert_eq(add(2, 3), 5) }";
        let result = call_tool("favnir_test", serde_json::json!({ "source": src }));
        let text = result["content"][0]["text"].as_str().unwrap_or("");
        assert!(text.contains("passed"), "got: {text}");
        assert!(result.get("isError").is_none(), "unexpected isError, text: {text}");
    }

    #[test]
    fn tools_call_favnir_test_fail() {
        let src = "fn add(a: Int, b: Int) -> Int { a + b }\ntest \"add fails\" { assert_eq(add(2, 3), 99) }";
        let result = call_tool("favnir_test", serde_json::json!({ "source": src }));
        let text = result["content"][0]["text"].as_str().unwrap_or("");
        assert!(
            text.contains("failed") || text.contains("FAIL"),
            "got: {text}"
        );
    }

    #[test]
    fn tools_call_favnir_list_runes() {
        let result = call_tool("favnir_list_runes", serde_json::json!({}));
        let text = result["content"][0]["text"].as_str().unwrap_or("");
        assert!(text.contains("db:"), "got: {text}");
        assert!(text.contains("http:"), "got: {text}");
        assert!(text.contains("log:"), "got: {text}");
    }

    #[test]
    fn tools_call_favnir_rune_docs_db() {
        let result = call_tool(
            "favnir_rune_docs",
            serde_json::json!({ "rune": "db" }),
        );
        let text = result["content"][0]["text"].as_str().unwrap_or("");
        assert!(text.contains("db"), "got: {text}");
        assert!(result.get("isError").is_none());
    }

    #[test]
    fn tools_call_favnir_rune_docs_unknown() {
        let result = call_tool(
            "favnir_rune_docs",
            serde_json::json!({ "rune": "doesnotexist" }),
        );
        assert_eq!(result.get("isError"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn tools_call_unknown_tool_returns_error() {
        let result = call_tool("nonexistent_tool", serde_json::json!({}));
        assert_eq!(result.get("isError"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn resources_list_returns_uris() {
        let mut out = Vec::new();
        let mut server = McpServer::new(&mut out);
        server.handle(McpRequest {
            id: Some(serde_json::json!(1)),
            method: "resources/list".to_string(),
            params: serde_json::json!({}),
        });
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("favnir://docs/stdlib"), "got: {text}");
        assert!(text.contains("favnir://runes/db"), "got: {text}");
        assert!(text.contains("favnir://project/files"), "got: {text}");
    }

    #[test]
    fn resources_read_stdlib() {
        let mut out = Vec::new();
        let mut server = McpServer::new(&mut out);
        server.handle(McpRequest {
            id: Some(serde_json::json!(1)),
            method: "resources/read".to_string(),
            params: serde_json::json!({ "uri": "favnir://docs/stdlib" }),
        });
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("String"), "got: {text}");
        assert!(text.contains("List"), "got: {text}");
    }

    #[test]
    fn resources_read_rune_db() {
        let mut out = Vec::new();
        let mut server = McpServer::new(&mut out);
        server.handle(McpRequest {
            id: Some(serde_json::json!(1)),
            method: "resources/read".to_string(),
            params: serde_json::json!({ "uri": "favnir://runes/db" }),
        });
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("db"), "got: {text}");
    }

    #[test]
    fn prompts_list_returns_two() {
        let mut out = Vec::new();
        let mut server = McpServer::new(&mut out);
        server.handle(McpRequest {
            id: Some(serde_json::json!(1)),
            method: "prompts/list".to_string(),
            params: serde_json::json!({}),
        });
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("write_pipeline"), "got: {text}");
        assert!(text.contains("fix_type_error"), "got: {text}");
    }

    #[test]
    fn prompts_get_write_pipeline() {
        let mut out = Vec::new();
        let mut server = McpServer::new(&mut out);
        server.handle(McpRequest {
            id: Some(serde_json::json!(1)),
            method: "prompts/get".to_string(),
            params: serde_json::json!({
                "name": "write_pipeline",
                "arguments": { "source_type": "csv", "output_type": "db" }
            }),
        });
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("messages"), "got: {text}");
        assert!(text.contains("csv"), "got: {text}");
        assert!(text.contains("db"), "got: {text}");
    }

    #[test]
    fn prompts_get_fix_type_error() {
        let mut out = Vec::new();
        let mut server = McpServer::new(&mut out);
        server.handle(McpRequest {
            id: Some(serde_json::json!(1)),
            method: "prompts/get".to_string(),
            params: serde_json::json!({
                "name": "fix_type_error",
                "arguments": {
                    "source": "fn main() -> Int { foo }",
                    "error": "E0102: undefined foo"
                }
            }),
        });
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("messages"), "got: {text}");
        assert!(text.contains("E0102"), "got: {text}");
    }

    #[test]
    fn shutdown_returns_null() {
        let mut out = Vec::new();
        let mut server = McpServer::new(&mut out);
        server.handle(McpRequest {
            id: Some(serde_json::json!(1)),
            method: "shutdown".to_string(),
            params: serde_json::json!({}),
        });
        let resp = last_response(&out);
        assert_eq!(resp["result"], serde_json::Value::Null);
    }
}
