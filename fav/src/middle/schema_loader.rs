/// Schema loader for v18.4.0 — loads JSON Schema files and converts to Favnir types.
use std::fs;
use std::path::{Path, PathBuf};
use crate::middle::checker::Type;

// ── SchemaSource ──────────────────────────────────────────────────────────────

pub enum SchemaSource {
    File(PathBuf),
    BigQuery { project: String, dataset: String, table: String },
    Postgres { table: String },
    Snowflake { db: String, schema: String, table: String },
}

// ── SchemaField ──────────────────────────────────────────────────────────────

pub struct SchemaField {
    pub name: String,
    /// Favnir type name: "Int" | "Float" | "String" | "Bool" | "List<...>"
    pub ty: String,
}

// ── URI parsing ──────────────────────────────────────────────────────────────

pub fn parse_schema_uri(uri: &str) -> Result<SchemaSource, String> {
    if let Some(path) = uri.strip_prefix("file:") {
        return Ok(SchemaSource::File(PathBuf::from(path)));
    }
    if let Some(rest) = uri.strip_prefix("bigquery:") {
        let parts: Vec<&str> = rest.splitn(3, '.').collect();
        if parts.len() == 3 {
            return Ok(SchemaSource::BigQuery {
                project: parts[0].to_string(),
                dataset: parts[1].to_string(),
                table: parts[2].to_string(),
            });
        }
        return Err(format!("invalid bigquery URI: expected project.dataset.table, got `{}`", rest));
    }
    if let Some(table) = uri.strip_prefix("postgres:") {
        return Ok(SchemaSource::Postgres { table: table.to_string() });
    }
    if let Some(rest) = uri.strip_prefix("snowflake:") {
        let parts: Vec<&str> = rest.splitn(3, '.').collect();
        if parts.len() == 3 {
            return Ok(SchemaSource::Snowflake {
                db: parts[0].to_string(),
                schema: parts[1].to_string(),
                table: parts[2].to_string(),
            });
        }
        return Err(format!("invalid snowflake URI: expected DB.SCHEMA.TABLE, got `{}`", rest));
    }
    Err(format!("unknown schema URI scheme in `{}`", uri))
}

// ── Cache key ────────────────────────────────────────────────────────────────

pub fn cache_key(source: &SchemaSource) -> String {
    match source {
        SchemaSource::File(path) => {
            let s = path.to_string_lossy().replace(['/', '\\', '.'], "_");
            format!("file__{}", s)
        }
        SchemaSource::BigQuery { project, dataset, table } => {
            format!("bigquery__{}__{}_{}", project, dataset, table)
        }
        SchemaSource::Postgres { table } => format!("postgres__{}", table),
        SchemaSource::Snowflake { db, schema, table } => {
            format!("snowflake__{}__{}__{}", db, schema, table)
        }
    }
}

// ── Load schema ──────────────────────────────────────────────────────────────

/// Load schema from URI, using/writing cache in `.fav/schema-cache/`.
/// `refresh` = true → ignore cache and re-fetch.
pub fn load_schema_uri(uri: &str, refresh: bool) -> Result<Vec<SchemaField>, String> {
    let source = parse_schema_uri(uri)?;
    let cache_dir = PathBuf::from(".fav/schema-cache");
    load_schema(&source, &cache_dir, refresh)
}

pub fn load_schema(
    source: &SchemaSource,
    cache_dir: &Path,
    refresh: bool,
) -> Result<Vec<SchemaField>, String> {
    let key = cache_key(source);
    let cache_file = cache_dir.join(format!("{}.json", key));

    // Try cache first (unless refresh is requested)
    if !refresh && cache_file.exists() {
        if let Ok(cached) = read_cache(&cache_file) {
            return Ok(cached);
        }
    }

    // Fetch from source
    let fields = match source {
        SchemaSource::File(path) => load_from_json_schema_file(path)?,
        // Other sources not yet implemented in v18.4.0 — return empty
        SchemaSource::BigQuery { .. } | SchemaSource::Postgres { .. } | SchemaSource::Snowflake { .. } => {
            vec![]
        }
    };

    // Write cache
    let _ = fs::create_dir_all(cache_dir);
    let _ = write_cache(&cache_file, &fields);

    Ok(fields)
}

// ── JSON Schema file parsing ──────────────────────────────────────────────────

fn load_from_json_schema_file(path: &Path) -> Result<Vec<SchemaField>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("E0338: schema file not found `{}`: {}", path.display(), e))?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("E0339: invalid JSON schema `{}`: {}", path.display(), e))?;
    parse_json_schema_object(&value)
}

fn parse_json_schema_object(value: &serde_json::Value) -> Result<Vec<SchemaField>, String> {
    // Handle top-level: { "type": "object", "properties": { ... } }
    // or directly { "properties": { ... } }
    let properties = if let Some(props) = value.get("properties") {
        props
    } else if value.is_object() {
        // Maybe the value itself is a flat map of field → { type }
        value
    } else {
        return Err("E0339: JSON schema must have a `properties` object".to_string());
    };

    let obj = properties
        .as_object()
        .ok_or_else(|| "E0339: `properties` must be a JSON object".to_string())?;

    let mut fields = Vec::new();
    for (name, field_schema) in obj {
        let ty = json_schema_type_to_favnir(field_schema);
        fields.push(SchemaField { name: name.clone(), ty });
    }
    Ok(fields)
}

fn json_schema_type_to_favnir(schema: &serde_json::Value) -> String {
    // Handle nullable: { "type": ["string", "null"] }
    if let Some(type_arr) = schema.get("type").and_then(|t| t.as_array()) {
        let non_null: Vec<&str> = type_arr
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|&s| s != "null")
            .collect();
        if non_null.len() == 1 {
            return format!("Option<{}>", json_type_str_to_favnir(non_null[0], schema));
        }
        return "String".to_string(); // fallback
    }

    let type_str = schema
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("string");

    json_type_str_to_favnir(type_str, schema)
}

fn json_type_str_to_favnir(type_str: &str, schema: &serde_json::Value) -> String {
    match type_str {
        "integer" => "Int".to_string(),
        "number" => "Float".to_string(),
        "boolean" => "Bool".to_string(),
        "string" => "String".to_string(),
        "array" => {
            if let Some(items) = schema.get("items") {
                format!("List<{}>", json_schema_type_to_favnir(items))
            } else {
                "List<String>".to_string()
            }
        }
        "object" => "String".to_string(), // nested objects → String for now
        _ => "String".to_string(),
    }
}

// ── Cache I/O ─────────────────────────────────────────────────────────────────

fn read_cache(path: &Path) -> Result<Vec<SchemaField>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let fields = v["fields"]
        .as_array()
        .ok_or("invalid cache format")?
        .iter()
        .filter_map(|f| {
            let name = f["name"].as_str()?;
            let ty = f["type"].as_str()?;
            Some(SchemaField { name: name.to_string(), ty: ty.to_string() })
        })
        .collect();
    Ok(fields)
}

fn write_cache(path: &Path, fields: &[SchemaField]) -> Result<(), String> {
    let arr: Vec<serde_json::Value> = fields
        .iter()
        .map(|f| serde_json::json!({ "name": f.name, "type": f.ty }))
        .collect();
    let content = serde_json::to_string_pretty(&serde_json::json!({ "fields": arr }))
        .map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

// ── Type conversion ──────────────────────────────────────────────────────────

/// Convert a schema field type string (e.g. "Int", "List<String>") to a Favnir `Type`.
pub fn schema_field_type_to_type(ty_str: &str) -> Type {
    match ty_str {
        "Int" => Type::Int,
        "Float" => Type::Float,
        "String" => Type::String,
        "Bool" => Type::Bool,
        _ if ty_str.starts_with("Option<") => Type::Option(Box::new(Type::String)),
        _ if ty_str.starts_with("List<") => Type::List(Box::new(Type::String)),
        _ => Type::String,
    }
}
