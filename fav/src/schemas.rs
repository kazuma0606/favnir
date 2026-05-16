// schemas.rs — Type constraint definitions from schemas/*.yaml (v4.1.5)

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Constraint definition for a single field.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FieldConstraints {
    #[serde(default)]
    pub constraints: Vec<String>, // e.g. ["primary_key", "positive"]
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub pattern: Option<String>,
    #[serde(default)]
    pub nullable: bool,
}

/// Schema for a single type: field name → constraints.
pub type TypeSchema = HashMap<String, FieldConstraints>;

/// All schemas in the project: type name → TypeSchema.
pub type ProjectSchemas = HashMap<String, TypeSchema>;

/// Scan `<project_root>/schemas/` and load all `.yaml` files.
/// Returns an empty map if the directory does not exist or cannot be read.
pub fn load_schemas(project_root: &Path) -> ProjectSchemas {
    let dir = project_root.join("schemas");
    if !dir.is_dir() {
        return HashMap::new();
    }
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return HashMap::new();
    };
    let mut result: ProjectSchemas = HashMap::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(parsed): Result<HashMap<String, TypeSchema>, _> = serde_yaml::from_str(&src) else {
            continue;
        };
        result.extend(parsed);
    }
    result
}
