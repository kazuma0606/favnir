// Favnir Module Resolver
// Loads .fav files by module path, caches their exported symbols.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::checker::{Checker, Type};
use crate::ast::Visibility;
use crate::frontend::lexer::Span;
use crate::frontend::parser::Parser;
use crate::toml::FavToml;

// ── ModuleScope ───────────────────────────────────────────────────────────────

/// The exported symbols of a loaded module.
#[derive(Debug, Clone)]
pub struct ModuleScope {
    /// Maps symbol name → (resolved type, visibility).
    pub symbols: HashMap<String, (Type, Visibility)>,
}

// ── ResolveError ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ResolveError {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

impl ResolveError {
    fn new(code: &'static str, msg: impl Into<String>, span: Span) -> Self {
        ResolveError {
            code,
            message: msg.into(),
            span,
        }
    }
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error[{}]: {}\n  --> {}:{}:{}",
            self.code, self.message, self.span.file, self.span.line, self.span.col
        )
    }
}

// ── Resolver ──────────────────────────────────────────────────────────────────

pub struct Resolver {
    /// Project configuration (None = single-file mode).
    pub toml: Option<FavToml>,
    /// Absolute path to the project root (directory containing fav.toml).
    pub root: Option<PathBuf>,
    /// Cache: module path → resolved scope.
    modules: HashMap<String, ModuleScope>,
    /// Cycle detection: set of module paths currently being loaded.
    loading: HashSet<String>,
    loading_names: HashMap<String, String>,
}

impl Resolver {
    pub fn new(toml: Option<FavToml>, root: Option<PathBuf>) -> Self {
        Resolver {
            toml,
            root,
            modules: HashMap::new(),
            loading: HashSet::new(),
            loading_names: HashMap::new(),
        }
    }

    /// Load (or return cached) module by dotted path (e.g. `"data.users"`).
    /// Returns the module scope, or `None` if loading failed.
    pub fn load_module(
        &mut self,
        mod_path: &str,
        errors: &mut Vec<ResolveError>,
        span: &Span,
    ) -> Option<&ModuleScope> {
        if self.modules.contains_key(mod_path) {
            return self.modules.get(mod_path);
        }
        if mod_path == "std.states" {
            self.modules.insert(
                mod_path.to_string(),
                ModuleScope {
                    symbols: crate::std_states::export_scope(),
                },
            );
            return self.modules.get(mod_path);
        }
        if self.loading.contains(mod_path) {
            errors.push(ResolveError::new(
                "E012",
                format!("circular import: `{}`", mod_path),
                span.clone(),
            ));
            return None;
        }

        // Resolve file path
        let file = match self.mod_path_to_file(mod_path) {
            Some(p) => p,
            None => {
                errors.push(ResolveError::new(
                    "E013",
                    format!(
                        "module `{}` not found (no fav.toml or src dir configured)",
                        mod_path
                    ),
                    span.clone(),
                ));
                return None;
            }
        };

        let source = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(_) => {
                errors.push(ResolveError::new(
                    "E013",
                    format!(
                        "module `{}` not found: `{}` does not exist",
                        mod_path,
                        file.display()
                    ),
                    span.clone(),
                ));
                return None;
            }
        };

        self.loading.insert(mod_path.to_string());

        // Parse
        let file_str = file.to_string_lossy().to_string();
        let program = match Parser::parse_str(&source, &file_str) {
            Ok(p) => p,
            Err(e) => {
                errors.push(ResolveError::new(
                    "E013",
                    format!("parse error in module `{}`: {}", mod_path, e.message),
                    span.clone(),
                ));
                self.loading.remove(mod_path);
                return None;
            }
        };

        // Type-check and extract exports
        let (type_errors, _, exports) = Checker::check_program_and_export(&program);
        // Surface type errors from the dependency as resolve errors
        for te in type_errors {
            errors.push(ResolveError::new(te.code, te.message, te.span));
        }

        let scope = ModuleScope { symbols: exports };
        self.modules.insert(mod_path.to_string(), scope);
        self.loading.remove(mod_path);
        self.modules.get(mod_path)
    }

    /// Resolve a `use` path (e.g. `["data","users","create"]`) into a symbol name + Type.
    /// Returns `(symbol_name, type)` on success, or pushes an error and returns `None`.
    pub fn resolve_use(
        &mut self,
        use_path: &[String],
        errors: &mut Vec<ResolveError>,
        span: &Span,
    ) -> Option<(String, Type)> {
        if use_path.is_empty() {
            return None;
        }
        let sym_name = use_path.last().unwrap().clone();
        let mod_path = use_path[..use_path.len() - 1].join(".");

        if mod_path.is_empty() {
            // `use foo` with no module — nothing to load
            errors.push(ResolveError::new(
                "E013",
                format!(
                    "`use {}` needs at least two segments (module.symbol)",
                    sym_name
                ),
                span.clone(),
            ));
            return None;
        }

        let scope = self.load_module(&mod_path, errors, span)?;

        match scope.symbols.get(&sym_name) {
            None => {
                errors.push(ResolveError::new(
                    "E013",
                    format!("`{}` is not defined in module `{}`", sym_name, mod_path),
                    span.clone(),
                ));
                None
            }
            Some((_, vis)) if *vis == Visibility::Private => {
                errors.push(ResolveError::new(
                    "E014",
                    format!(
                        "`{}::{}` is private — only `public` or `internal` symbols can be imported",
                        mod_path, sym_name
                    ),
                    span.clone(),
                ));
                None
            }
            Some((ty, _)) => Some((sym_name, ty.clone())),
        }
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    /// Convert a dotted module path to an absolute file path.
    /// Returns `None` if there is no project root configured.
    fn mod_path_to_file(&self, mod_path: &str) -> Option<PathBuf> {
        let root = self.root.as_ref()?;
        let src = self.toml.as_ref().map(|t| t.src.as_str()).unwrap_or(".");
        let rel: PathBuf = mod_path.split('.').collect();
        Some(root.join(src).join(rel).with_extension("fav"))
    }

    pub fn resolve_local_import_file(&self, import_path: &str) -> Option<PathBuf> {
        let root = self.root.as_ref()?;
        let rel = PathBuf::from(import_path);
        Some(
            self.toml
                .as_ref()
                .map(|t| t.src_dir(root))
                .unwrap_or_else(|| root.clone())
                .join(rel)
                .with_extension("fav"),
        )
    }

    pub fn resolve_rune_import_file(&self, import_path: &str) -> Option<PathBuf> {
        let root = self.root.as_ref()?;
        let base = self
            .toml
            .as_ref()
            .map(|t| t.runes_dir(root))
            .unwrap_or_else(|| root.join("runes"));
        Some(base.join(import_path).join(format!("{import_path}.fav")))
    }

    pub fn cached_scope(&self, key: &str) -> Option<ModuleScope> {
        self.modules.get(key).cloned()
    }

    pub fn cache_scope(&mut self, key: impl Into<String>, scope: ModuleScope) {
        self.modules.insert(key.into(), scope);
    }

    pub fn begin_loading(&mut self, key: &str, display_name: &str) -> Option<String> {
        if self.loading.contains(key) {
            return self.loading_names.get(key).cloned();
        }
        self.loading.insert(key.to_string());
        self.loading_names
            .insert(key.to_string(), display_name.to_string());
        None
    }

    pub fn finish_loading(&mut self, key: &str) {
        self.loading.remove(key);
        self.loading_names.remove(key);
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Derive a default module path from the file path relative to `src_dir`.
/// `src/data/users.fav` → `"data.users"`
pub fn derive_module_path(file: &Path, src_dir: &Path) -> Option<String> {
    let rel = file.strip_prefix(src_dir).ok()?;
    let without_ext = rel.with_extension("");
    let parts: Vec<&str> = without_ext
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    Some(parts.join("."))
}

// ── tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Returns (Resolver, TempDir) — TempDir must be kept alive for the duration of the test.
    fn make_resolver_with_file(
        src_content: &str,
        mod_path_filename: &str,
    ) -> (Resolver, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        // fav.toml
        let toml_content = "[rune]\nname = \"test\"\nversion = \"0.1.0\"\nsrc = \"src\"\n";
        std::fs::write(root.join("fav.toml"), toml_content).unwrap();
        // src/
        let src_dir = root.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        // Write the .fav file at src/<mod_path_filename>
        // Use Path to normalise slashes on Windows
        let fav_path: PathBuf =
            src_dir.join(mod_path_filename.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = fav_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&fav_path, src_content).unwrap();

        let toml = FavToml {
            name: "test".into(),
            version: "0.1.0".into(),
            src: "src".into(),
            runes_path: None,
            dependencies: vec![],
        };
        let resolver = Resolver::new(Some(toml), Some(root));
        (resolver, dir) // dir must outlive the test
    }

    #[test]
    fn test_load_module_public_fn() {
        let (mut r, _dir) =
            make_resolver_with_file("public fn greet() -> Unit { () }", "greetings.fav");
        let mut errors = Vec::new();
        let span = Span::new("test", 0, 0, 1, 1);
        let scope = r.load_module("greetings", &mut errors, &span).unwrap();
        assert!(errors.is_empty(), "{:?}", errors);
        assert!(scope.symbols.contains_key("greet"));
        let (_, vis) = &scope.symbols["greet"];
        assert_eq!(*vis, Visibility::Public);
    }

    #[test]
    fn test_load_module_private_not_exported() {
        let (mut r, _dir) = make_resolver_with_file("fn secret() -> Unit { () }", "secret.fav");
        let mut errors = Vec::new();
        let span = Span::new("test", 0, 0, 1, 1);
        let scope = r.load_module("secret", &mut errors, &span).unwrap();
        // private fn is still in exports map but with Private visibility
        let (_, vis) = &scope.symbols["secret"];
        assert_eq!(*vis, Visibility::Private);
    }

    #[test]
    fn test_resolve_use_public() {
        let (mut r, _dir) =
            make_resolver_with_file("public fn add(a: Int, b: Int) -> Int { a }", "math/ops.fav");
        let mut errors = Vec::new();
        let span = Span::new("test", 0, 0, 1, 1);
        let path = vec!["math".to_string(), "ops".to_string(), "add".to_string()];
        let result = r.resolve_use(&path, &mut errors, &span);
        assert!(errors.is_empty(), "{:?}", errors);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "add");
    }

    #[test]
    fn test_resolve_use_private_error() {
        let (mut r, _dir) = make_resolver_with_file("fn hidden() -> Unit { () }", "utils.fav");
        let mut errors = Vec::new();
        let span = Span::new("test", 0, 0, 1, 1);
        let path = vec!["utils".to_string(), "hidden".to_string()];
        let result = r.resolve_use(&path, &mut errors, &span);
        assert!(result.is_none());
        assert_eq!(errors[0].code, "E014");
    }

    #[test]
    fn test_resolve_use_missing_symbol() {
        let (mut r, _dir) = make_resolver_with_file("public fn real() -> Unit { () }", "stuff.fav");
        let mut errors = Vec::new();
        let span = Span::new("test", 0, 0, 1, 1);
        let path = vec!["stuff".to_string(), "ghost".to_string()];
        let result = r.resolve_use(&path, &mut errors, &span);
        assert!(result.is_none());
        assert_eq!(errors[0].code, "E013");
    }

    #[test]
    fn test_module_not_found() {
        let dir = tempdir().unwrap();
        let toml = FavToml {
            name: "t".into(),
            version: "0.1.0".into(),
            src: "src".into(),
            runes_path: None,
            dependencies: vec![],
        };
        let mut r = Resolver::new(Some(toml), Some(dir.path().to_path_buf()));
        let mut errors = Vec::new();
        let span = Span::new("test", 0, 0, 1, 1);
        let result = r.load_module("no.such.module", &mut errors, &span);
        assert!(result.is_none());
        assert_eq!(errors[0].code, "E013");
    }

    #[test]
    fn test_load_std_states_module_exports_known_symbols() {
        let mut r = Resolver::new(None, None);
        let mut errors = Vec::new();
        let span = Span::new("test", 0, 0, 1, 1);
        let scope = r.load_module("std.states", &mut errors, &span).unwrap();
        assert!(errors.is_empty(), "{:?}", errors);
        assert!(scope.symbols.contains_key("PosInt"));
        assert!(scope.symbols.contains_key("Email"));
        let (_, vis) = &scope.symbols["PosInt"];
        assert_eq!(*vis, Visibility::Public);
    }

    #[test]
    fn test_resolve_use_std_states_symbol() {
        let mut r = Resolver::new(None, None);
        let mut errors = Vec::new();
        let span = Span::new("test", 0, 0, 1, 1);
        let path = vec![
            "std".to_string(),
            "states".to_string(),
            "PosInt".to_string(),
        ];
        let result = r.resolve_use(&path, &mut errors, &span);
        assert!(errors.is_empty(), "{:?}", errors);
        let (sym, ty) = result.expect("std.states.PosInt resolves");
        assert_eq!(sym, "PosInt");
        assert_eq!(ty, Type::Named("PosInt".into(), vec![]));
    }

    #[test]
    fn test_derive_module_path() {
        let src = Path::new("/proj/src");
        let file = Path::new("/proj/src/data/users.fav");
        assert_eq!(
            derive_module_path(file, src),
            Some("data.users".to_string())
        );
    }
}
