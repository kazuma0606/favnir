#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use crate::backend::codegen::codegen_program;
use crate::backend::vm::VM;
use crate::frontend::parser::Parser;
use crate::middle::checker::Checker;
use crate::middle::compiler::compile_program;
use crate::value::Value;

// ── data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub version: String,
    pub title: String,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: String,
    #[serde(rename = "type")]
    pub cell_type: CellType,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<CellOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    Markdown,
    Code,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellOutput {
    pub kind: OutputKind,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputKind {
    Value,
    Table,
    Error,
    None,
}

// ── notebook operations ───────────────────────────────────────────────────────

impl Notebook {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {path}: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("invalid notebook JSON: {e}"))
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("serialize error: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("cannot write {path}: {e}"))
    }

    pub fn new_notebook(title: &str) -> Self {
        Notebook {
            version: "1.0".to_string(),
            title: title.to_string(),
            cells: vec![
                Cell {
                    id: "c001".to_string(),
                    cell_type: CellType::Markdown,
                    content: format!("# {}\n\nNotebook description here.", title),
                    output: None,
                },
                Cell {
                    id: "c002".to_string(),
                    cell_type: CellType::Code,
                    content: "// Start writing Favnir code here\nfn main() -> Int { 42 }"
                        .to_string(),
                    output: None,
                },
            ],
        }
    }

    pub fn add_cell(&mut self, cell_type: CellType, content: &str) -> String {
        let id = format!("c{:03}", self.cells.len() + 1);
        self.cells.push(Cell {
            id: id.clone(),
            cell_type,
            content: content.to_string(),
            output: None,
        });
        id
    }

    pub fn remove_cell(&mut self, id: &str) -> bool {
        let len_before = self.cells.len();
        self.cells.retain(|c| c.id != id);
        self.cells.len() < len_before
    }

    pub fn update_cell_content(&mut self, id: &str, content: &str) -> bool {
        for cell in &mut self.cells {
            if cell.id == id {
                cell.content = content.to_string();
                cell.output = None;
                return true;
            }
        }
        false
    }
}

// ── execution engine ──────────────────────────────────────────────────────────

/// Build context source from all definition cells (cells without `fn main(`)
/// that precede the cell at `up_to_idx`.
fn build_context(cells: &[Cell], up_to_idx: usize) -> String {
    let mut parts = Vec::new();
    for cell in cells[..up_to_idx]
        .iter()
        .filter(|c| c.cell_type == CellType::Code)
    {
        // Skip execution cells (those defining main) to avoid duplicate main
        if !cell_defines_main(&cell.content) {
            parts.push(cell.content.as_str());
        }
    }
    parts.join("\n")
}

fn cell_defines_main(content: &str) -> bool {
    // Simple heuristic: look for "fn main(" anywhere in the cell
    content.contains("fn main(")
}

/// Execute a code cell. Returns the output.
fn execute_code_cell(cells: &[Cell], idx: usize) -> CellOutput {
    let cell = &cells[idx];
    if cell.cell_type == CellType::Markdown {
        return CellOutput {
            kind: OutputKind::None,
            text: String::new(),
        };
    }

    let ctx = build_context(cells, idx);
    let combined = if ctx.is_empty() {
        cell.content.clone()
    } else {
        format!("{}\n{}", ctx, cell.content)
    };

    let program = match Parser::parse_str(&combined, &cell.id) {
        Err(e) => {
            return CellOutput {
                kind: OutputKind::Error,
                text: format!("E0500 at line {}: {}", e.span.line, e.message),
            }
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
        return CellOutput {
            kind: OutputKind::Error,
            text: msg,
        };
    }

    let ir = compile_program(&program);
    let artifact = codegen_program(&ir);

    match artifact.fn_idx_by_name("main") {
        None => CellOutput {
            kind: OutputKind::None,
            text: String::new(),
        },
        Some(fn_idx) => match VM::run(&artifact, fn_idx, vec![]) {
            Ok(val) => value_to_output(&val),
            Err(e) => CellOutput {
                kind: OutputKind::Error,
                text: format!("Runtime error: {}", e.message),
            },
        },
    }
}

fn value_to_output(val: &Value) -> CellOutput {
    match val {
        Value::Unit => CellOutput {
            kind: OutputKind::None,
            text: String::new(),
        },
        Value::List(items) if is_list_of_records(items) => CellOutput {
            kind: OutputKind::Table,
            text: format_table(items),
        },
        other => CellOutput {
            kind: OutputKind::Value,
            text: other.display(),
        },
    }
}

fn is_list_of_records(items: &[Value]) -> bool {
    !items.is_empty() && matches!(items[0], Value::Record(_))
}

fn format_table(items: &[Value]) -> String {
    let records: Vec<&HashMap<String, Value>> = items
        .iter()
        .filter_map(|v| {
            if let Value::Record(m) = v {
                Some(m)
            } else {
                None
            }
        })
        .collect();

    if records.is_empty() {
        return String::new();
    }

    // Collect headers from first record
    let mut headers: Vec<String> = records[0].keys().cloned().collect();
    headers.sort();

    let header_row = format!("| {} |", headers.join(" | "));
    let sep_row = format!("|{}|", headers.iter().map(|_| "---|").collect::<String>());
    let mut rows = vec![header_row, sep_row];

    for rec in &records {
        let cells: Vec<String> = headers
            .iter()
            .map(|h| {
                rec.get(h)
                    .map(|v| v.display())
                    .unwrap_or_default()
                    .replace('|', "\\|")
            })
            .collect();
        rows.push(format!("| {} |", cells.join(" | ")));
    }

    rows.join("\n")
}

/// Run a single cell by ID, returning the output.
pub fn run_cell(notebook: &mut Notebook, cell_id: &str) -> CellOutput {
    let idx = match notebook.cells.iter().position(|c| c.id == cell_id) {
        Some(i) => i,
        None => {
            return CellOutput {
                kind: OutputKind::Error,
                text: format!("cell '{}' not found", cell_id),
            }
        }
    };
    let output = execute_code_cell(&notebook.cells, idx);
    notebook.cells[idx].output = Some(output.clone());
    output
}

/// Run all cells in order, returning (cell_id, output) pairs.
pub fn run_all(notebook: &mut Notebook) -> Vec<(String, CellOutput)> {
    let ids: Vec<String> = notebook.cells.iter().map(|c| c.id.clone()).collect();
    ids.into_iter()
        .map(|id| {
            let output = run_cell(notebook, &id);
            (id, output)
        })
        .collect()
}

// ── type-check only ───────────────────────────────────────────────────────────

/// Type-check all code cells. Returns (cell_id, error_messages) for cells with errors.
pub fn check_notebook(notebook: &Notebook) -> Vec<(String, Vec<String>)> {
    let mut result = Vec::new();
    let mut ctx = String::new();

    for cell in &notebook.cells {
        if cell.cell_type != CellType::Code {
            continue;
        }
        let combined = if ctx.is_empty() {
            cell.content.clone()
        } else {
            format!("{}\n{}", ctx, cell.content)
        };

        match Parser::parse_str(&combined, &cell.id) {
            Err(e) => {
                result.push((
                    cell.id.clone(),
                    vec![format!("E0500 at line {}: {}", e.span.line, e.message)],
                ));
            }
            Ok(program) => {
                let mut checker = Checker::new();
                let (errors, _) = checker.check_with_self(&program);
                if !errors.is_empty() {
                    result.push((
                        cell.id.clone(),
                        errors
                            .iter()
                            .map(|e| format!("{} at line {}: {}", e.code, e.span.line, e.message))
                            .collect(),
                    ));
                }
            }
        }

        // Only add non-main cells to running context
        if !cell_defines_main(&cell.content) {
            if ctx.is_empty() {
                ctx = cell.content.clone();
            } else {
                ctx = format!("{}\n{}", ctx, cell.content);
            }
        }
    }

    result
}

// ── markdown export ───────────────────────────────────────────────────────────

pub fn export_markdown(notebook: &Notebook) -> String {
    let mut md = format!("# {}\n\n", notebook.title);
    for cell in &notebook.cells {
        match cell.cell_type {
            CellType::Markdown => {
                md.push_str(&cell.content);
                md.push_str("\n\n");
            }
            CellType::Code => {
                md.push_str("```favnir\n");
                md.push_str(&cell.content);
                md.push_str("\n```\n\n");
                if let Some(output) = &cell.output {
                    if output.kind != OutputKind::None && !output.text.is_empty() {
                        match output.kind {
                            OutputKind::Error => {
                                md.push_str(&format!("> Error: {}\n\n", output.text));
                            }
                            OutputKind::Table => {
                                md.push_str(&output.text);
                                md.push_str("\n\n");
                            }
                            _ => {
                                md.push_str("```\n");
                                md.push_str(&output.text);
                                md.push_str("\n```\n\n");
                            }
                        }
                    }
                }
            }
        }
    }
    md
}

// ── HTTP server ───────────────────────────────────────────────────────────────

const NOTEBOOK_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Favnir Notebook</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:monospace;max-width:960px;margin:40px auto;padding:0 20px;background:#fff;color:#222}
h1{font-size:22px;margin-bottom:16px}
.toolbar{margin-bottom:20px;display:flex;gap:8px;flex-wrap:wrap}
button{padding:5px 12px;cursor:pointer;border:1px solid #bbb;background:#f8f8f8;border-radius:3px;font-family:monospace;font-size:13px}
button:hover{background:#e8e8e8}
button.run-btn{background:#e8f4e8;border-color:#6a6}
button.del-btn{background:#fde8e8;border-color:#a66}
.cell{border:1px solid #ddd;margin:12px 0;border-radius:4px;overflow:hidden}
.cell-header{background:#f5f5f5;padding:4px 10px;display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid #eee}
.cell-type{font-size:11px;color:#888;text-transform:uppercase;letter-spacing:1px}
.cell-actions{display:flex;gap:6px}
textarea{width:100%;font-family:monospace;font-size:13px;padding:10px;border:none;border-bottom:1px solid #eee;resize:vertical;min-height:60px;background:#fafff8;outline:none}
.cell-markdown textarea{background:#fffdf8}
.output{padding:8px 12px;font-size:13px;white-space:pre-wrap;min-height:20px}
.output.out-none{display:none}
.output.out-error{color:#c00;background:#fff8f8;border-top:1px solid #fdd}
.output.out-table{overflow-x:auto;font-size:12px;background:#f8f8ff}
.output.out-value{color:#060;background:#f8fff8}
.status{font-size:12px;color:#888;margin-top:8px}
</style>
</head>
<body>
<h1 id="nb-title">Favnir Notebook</h1>
<div class="toolbar">
  <button onclick="runAll()">&#9654; Run All</button>
  <button onclick="addCodeCell()">+ Code Cell</button>
  <button onclick="addMarkdownCell()">+ Markdown Cell</button>
  <button onclick="exportMarkdown()">&#8659; Export MD</button>
</div>
<div id="cells"></div>
<div class="status" id="status"></div>
<script>
let notebook=null;
function setStatus(s){document.getElementById('status').textContent=s}

async function loadNotebook(){
  const r=await fetch('/api/notebook');
  notebook=await r.json();
  document.getElementById('nb-title').textContent=notebook.title;
  renderCells();
}

function renderCells(){
  const c=document.getElementById('cells');
  c.innerHTML='';
  for(const cell of notebook.cells)c.appendChild(makeCell(cell));
}

function makeCell(cell){
  const div=document.createElement('div');
  div.className='cell cell-'+cell.type;
  div.id='cell-'+cell.id;

  const hdr=document.createElement('div');
  hdr.className='cell-header';

  const typeLabel=document.createElement('span');
  typeLabel.className='cell-type';
  typeLabel.textContent=cell.type;
  hdr.appendChild(typeLabel);

  const acts=document.createElement('div');
  acts.className='cell-actions';
  if(cell.type==='code'){
    const run=document.createElement('button');
    run.className='run-btn';
    run.textContent='Run';
    run.onclick=()=>runCell(cell.id);
    acts.appendChild(run);
  }
  const del=document.createElement('button');
  del.className='del-btn';
  del.textContent='Del';
  del.onclick=()=>deleteCell(cell.id);
  acts.appendChild(del);
  hdr.appendChild(acts);
  div.appendChild(hdr);

  const ta=document.createElement('textarea');
  ta.rows=Math.max(3,cell.content.split('\n').length+1);
  ta.value=cell.content;
  let timer=null;
  ta.oninput=()=>{
    clearTimeout(timer);
    timer=setTimeout(()=>updateCell(cell.id,ta.value),500);
  };
  div.appendChild(ta);

  const out=document.createElement('div');
  const ok=cell.output?cell.output.kind:'none';
  out.className='output out-'+ok;
  out.textContent=cell.output&&ok!=='none'?cell.output.text:'';
  div.appendChild(out);

  return div;
}

async function runCell(id){
  setStatus('Running cell '+id+'...');
  await fetch('/api/run/'+id,{method:'POST'});
  await loadNotebook();
  setStatus('Done.');
}

async function runAll(){
  setStatus('Running all cells...');
  await fetch('/api/run-all',{method:'POST'});
  await loadNotebook();
  setStatus('All cells executed.');
}

async function updateCell(id,content){
  await fetch('/api/update/'+id,{
    method:'POST',
    headers:{'Content-Type':'application/json'},
    body:JSON.stringify({content})
  });
}

async function addCodeCell(){
  await fetch('/api/add-cell',{
    method:'POST',
    headers:{'Content-Type':'application/json'},
    body:JSON.stringify({type:'code',content:''})
  });
  await loadNotebook();
}

async function addMarkdownCell(){
  await fetch('/api/add-cell',{
    method:'POST',
    headers:{'Content-Type':'application/json'},
    body:JSON.stringify({type:'markdown',content:''})
  });
  await loadNotebook();
}

async function deleteCell(id){
  if(!confirm('Delete cell '+id+'?'))return;
  await fetch('/api/cell/'+id,{method:'DELETE'});
  await loadNotebook();
}

async function exportMarkdown(){
  const r=await fetch('/api/export');
  const text=await r.text();
  const a=document.createElement('a');
  a.href=URL.createObjectURL(new Blob([text],{type:'text/markdown'}));
  a.download='notebook.md';
  a.click();
}

loadNotebook();
</script>
</body>
</html>"#;

struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_http_request(stream: &mut TcpStream) -> Option<HttpRequest> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);

    // Read request line
    let mut request_line = String::new();
    reader.read_line(&mut request_line).ok()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();

    // Read headers
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed.to_lowercase().strip_prefix("content-length:") {
            content_length = rest.trim().parse().unwrap_or(0);
        }
    }

    // Read body
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).ok()?;
    }

    Some(HttpRequest { method, path, body })
}

fn http_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &str) {
    let status_text = match status {
        200 => "OK",
        204 => "No Content",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    };
    let _ = write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
        status, status_text, content_type, body.len(), body
    );
    let _ = stream.flush();
}

fn handle_notebook_request(
    req: HttpRequest,
    nb_path: &str,
    nb: &Arc<Mutex<Notebook>>,
    stream: &mut TcpStream,
) {
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => {
            http_response(stream, 200, "text/html; charset=utf-8", NOTEBOOK_UI_HTML);
        }
        ("GET", "/api/notebook") => {
            let guard = nb.lock().unwrap();
            let json = serde_json::to_string(&*guard).unwrap_or_default();
            drop(guard);
            http_response(stream, 200, "application/json; charset=utf-8", &json);
        }
        ("GET", "/api/export") => {
            let guard = nb.lock().unwrap();
            let md = export_markdown(&guard);
            drop(guard);
            http_response(stream, 200, "text/markdown; charset=utf-8", &md);
        }
        ("POST", path) if path.starts_with("/api/run/") => {
            let cell_id = &path["/api/run/".len()..];
            let mut guard = nb.lock().unwrap();
            run_cell(&mut guard, cell_id);
            let _ = guard.save(nb_path);
            let json = serde_json::to_string(&*guard).unwrap_or_default();
            drop(guard);
            http_response(stream, 200, "application/json; charset=utf-8", &json);
        }
        ("POST", "/api/run-all") => {
            let mut guard = nb.lock().unwrap();
            run_all(&mut guard);
            let _ = guard.save(nb_path);
            let json = serde_json::to_string(&*guard).unwrap_or_default();
            drop(guard);
            http_response(stream, 200, "application/json; charset=utf-8", &json);
        }
        ("POST", path) if path.starts_with("/api/update/") => {
            let cell_id = &path["/api/update/".len()..];
            if let Ok(body_json) = serde_json::from_slice::<serde_json::Value>(&req.body) {
                if let Some(content) = body_json.get("content").and_then(|v| v.as_str()) {
                    let mut guard = nb.lock().unwrap();
                    guard.update_cell_content(cell_id, content);
                    let _ = guard.save(nb_path);
                }
            }
            http_response(stream, 204, "text/plain", "");
        }
        ("POST", "/api/add-cell") => {
            if let Ok(body_json) = serde_json::from_slice::<serde_json::Value>(&req.body) {
                let cell_type = match body_json.get("type").and_then(|v| v.as_str()) {
                    Some("markdown") => CellType::Markdown,
                    _ => CellType::Code,
                };
                let content = body_json
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let mut guard = nb.lock().unwrap();
                guard.add_cell(cell_type, content);
                let _ = guard.save(nb_path);
                let json = serde_json::to_string(&*guard).unwrap_or_default();
                drop(guard);
                http_response(stream, 200, "application/json; charset=utf-8", &json);
            } else {
                http_response(stream, 400, "text/plain", "bad request");
            }
        }
        ("DELETE", path) if path.starts_with("/api/cell/") => {
            let cell_id = &path["/api/cell/".len()..];
            let mut guard = nb.lock().unwrap();
            guard.remove_cell(cell_id);
            let _ = guard.save(nb_path);
            http_response(stream, 204, "text/plain", "");
        }
        _ => {
            http_response(stream, 404, "text/plain", "Not Found");
        }
    }
}

pub fn serve_notebook(path: &str, port: u16, no_open: bool) {
    let notebook = match Notebook::load(path) {
        Ok(nb) => nb,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };
    let nb = Arc::new(Mutex::new(notebook));
    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: failed to bind on {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    let url = format!("http://localhost:{}", port);
    eprintln!("[notebook] serving {} on {}", path, url);
    eprintln!("[notebook] press Ctrl+C to stop");
    if !no_open {
        let _ = open::that(&url);
    }

    let nb_path = path.to_string();
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                if let Some(req) = read_http_request(&mut s) {
                    let nb_clone = Arc::clone(&nb);
                    handle_notebook_request(req, &nb_path, &nb_clone, &mut s);
                }
            }
            Err(e) => {
                eprintln!("[notebook] accept error: {}", e);
            }
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    fn make_notebook() -> Notebook {
        Notebook {
            version: "1.0".to_string(),
            title: "Test".to_string(),
            cells: vec![
                Cell {
                    id: "c001".to_string(),
                    cell_type: CellType::Markdown,
                    content: "# Hello".to_string(),
                    output: None,
                },
                Cell {
                    id: "c002".to_string(),
                    cell_type: CellType::Code,
                    content: "fn add(a: Int, b: Int) -> Int { a + b }".to_string(),
                    output: None,
                },
                Cell {
                    id: "c003".to_string(),
                    cell_type: CellType::Code,
                    content: "fn main() -> Int { add(2, 3) }".to_string(),
                    output: None,
                },
            ],
        }
    }

    #[test]
    fn notebook_parse_valid_json() {
        let json = r#"{"version":"1.0","title":"Demo","cells":[{"id":"c001","type":"code","content":"fn main() -> Int { 42 }"}]}"#;
        let nb: Notebook = serde_json::from_str(json).expect("parse");
        assert_eq!(nb.title, "Demo");
        assert_eq!(nb.cells.len(), 1);
    }

    #[test]
    fn notebook_serialize_roundtrip() {
        let nb = make_notebook();
        let json = serde_json::to_string(&nb).expect("serialize");
        let nb2: Notebook = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(nb2.title, nb.title);
        assert_eq!(nb2.cells.len(), nb.cells.len());
    }

    #[test]
    fn notebook_new_creates_default_cells() {
        let nb = Notebook::new_notebook("myproject");
        assert_eq!(nb.title, "myproject");
        assert_eq!(nb.cells.len(), 2);
        assert_eq!(nb.cells[0].cell_type, CellType::Markdown);
        assert_eq!(nb.cells[1].cell_type, CellType::Code);
    }

    #[test]
    fn run_cell_returns_int_value() {
        let mut nb = Notebook {
            version: "1.0".to_string(),
            title: "T".to_string(),
            cells: vec![Cell {
                id: "c001".to_string(),
                cell_type: CellType::Code,
                content: "fn main() -> Int { 42 }".to_string(),
                output: None,
            }],
        };
        let out = run_cell(&mut nb, "c001");
        assert_eq!(out.kind, OutputKind::Value, "expected Value, got {:?}: {}", out.kind, out.text);
        assert_eq!(out.text, "42");
    }

    #[test]
    fn run_cell_returns_error_on_type_error() {
        let mut nb = Notebook {
            version: "1.0".to_string(),
            title: "T".to_string(),
            cells: vec![Cell {
                id: "c001".to_string(),
                cell_type: CellType::Code,
                content: "fn main() -> Int { undefined_var }".to_string(),
                output: None,
            }],
        };
        let out = run_cell(&mut nb, "c001");
        assert_eq!(out.kind, OutputKind::Error, "expected Error, got {:?}", out.kind);
    }

    #[test]
    fn run_cell_uses_context_from_previous() {
        let mut nb = make_notebook(); // c002=add fn, c003=main calls add
        let out = run_cell(&mut nb, "c003");
        assert_eq!(out.kind, OutputKind::Value, "got {:?}: {}", out.kind, out.text);
        assert_eq!(out.text, "5");
    }

    #[test]
    fn run_cell_markdown_returns_none() {
        let mut nb = make_notebook();
        let out = run_cell(&mut nb, "c001");
        assert_eq!(out.kind, OutputKind::None);
        assert!(out.text.is_empty());
    }

    #[test]
    fn run_all_returns_all_outputs() {
        let mut nb = make_notebook();
        let results = run_all(&mut nb);
        assert_eq!(results.len(), 3);
        // c001 = markdown → None
        assert_eq!(results[0].1.kind, OutputKind::None);
        // c002 = definition (no main) → None
        assert_eq!(results[1].1.kind, OutputKind::None);
        // c003 = main returning 5 → Value
        assert_eq!(results[2].1.kind, OutputKind::Value);
        assert_eq!(results[2].1.text, "5");
    }

    #[test]
    fn run_cell_unit_output_is_none() {
        let mut nb = Notebook {
            version: "1.0".to_string(),
            title: "T".to_string(),
            cells: vec![Cell {
                id: "c001".to_string(),
                cell_type: CellType::Code,
                content: "fn main() -> Unit !Io { IO.println(\"hi\") }".to_string(),
                output: None,
            }],
        };
        let out = run_cell(&mut nb, "c001");
        assert_eq!(out.kind, OutputKind::None, "Unit should produce None output");
    }

    #[test]
    fn export_markdown_has_favnir_blocks() {
        let nb = make_notebook();
        let md = export_markdown(&nb);
        assert!(md.contains("```favnir"), "expected ```favnir block");
        assert!(md.contains("fn add("));
    }

    #[test]
    fn export_markdown_includes_output() {
        let mut nb = make_notebook();
        run_cell(&mut nb, "c003");
        let md = export_markdown(&nb);
        assert!(md.contains("```\n5\n```"), "expected output block with 5, got:\n{}", md);
    }

    #[test]
    fn notebook_load_save_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.fav.nb");
        let path_str = path.to_str().unwrap();

        let nb = Notebook::new_notebook("RoundTrip");
        nb.save(path_str).expect("save");

        let nb2 = Notebook::load(path_str).expect("load");
        assert_eq!(nb2.title, "RoundTrip");
        assert_eq!(nb2.cells.len(), 2);
    }

    #[test]
    fn check_notebook_no_errors_for_valid() {
        let nb = make_notebook();
        let errors = check_notebook(&nb);
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    }

    #[test]
    fn check_notebook_detects_type_error() {
        let nb = Notebook {
            version: "1.0".to_string(),
            title: "T".to_string(),
            cells: vec![Cell {
                id: "c001".to_string(),
                cell_type: CellType::Code,
                content: "fn main() -> Int { undefined_var }".to_string(),
                output: None,
            }],
        };
        let errors = check_notebook(&nb);
        assert!(!errors.is_empty(), "expected at least one type error");
        assert_eq!(errors[0].0, "c001");
    }

    #[test]
    fn format_table_produces_markdown() {
        use crate::value::Value;
        let mut row = HashMap::new();
        row.insert("name".to_string(), Value::Str("Alice".to_string()));
        row.insert("age".to_string(), Value::Int(30));
        let items = vec![Value::Record(row)];
        let table = format_table(&items);
        assert!(table.contains("| age | name |") || table.contains("| name | age |"), "got: {table}");
        assert!(table.contains("Alice"));
        assert!(table.contains("30"));
    }
}
