// v19.6.0 — wasm-opt integration and size reporting.

/// Optimization level for wasm-opt (Binaryen).
/// O0 = DCE only, no external tool.  O1-O3 invoke wasm-opt if installed.
/// Os = size optimization (-Os), v51.7.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmOptLevel {
    O0,
    O1,
    O2,
    O3,
    Os, // v51.7.0: サイズ最適化（-Os）
}

impl WasmOptLevel {
    pub fn flag(self) -> &'static str {
        match self {
            WasmOptLevel::O0 => "-O0",
            WasmOptLevel::O1 => "-O1",
            WasmOptLevel::O2 => "-O2",
            WasmOptLevel::O3 => "-O3",
            WasmOptLevel::Os => "-Os",
        }
    }
}

/// Before/after size report from the optimization pipeline.
#[derive(Debug, Clone)]
pub struct WasmSizeReport {
    pub before: usize,
    pub after: usize,
}

impl WasmSizeReport {
    pub fn reduction_pct(&self) -> f64 {
        if self.before == 0 {
            return 0.0;
        }
        (1.0 - self.after as f64 / self.before as f64) * 100.0
    }
}

/// Errors from running wasm-opt.
#[derive(Debug)]
pub enum WasmOptError {
    /// `wasm-opt` binary not found in PATH.
    NotInstalled,
    /// `wasm-opt` exited with non-zero status.
    ExitNonZero(i32),
    /// I/O error (temp file, read, write).
    Io(String),
}

/// Run `wasm-opt` on `bytes` and return optimized bytes + size report.
/// Returns `Err(WasmOptError::NotInstalled)` if the binary is not found.
pub fn run_wasm_opt(
    bytes: &[u8],
    level: WasmOptLevel,
    strip_debug: bool,
) -> Result<(Vec<u8>, WasmSizeReport), WasmOptError> {
    use std::io::Write as _;

    let before = bytes.len();

    if level == WasmOptLevel::O0 {
        return Ok((bytes.to_vec(), WasmSizeReport { before, after: before }));
    }

    let mut in_file =
        tempfile::NamedTempFile::new().map_err(|e| WasmOptError::Io(e.to_string()))?;
    in_file
        .write_all(bytes)
        .map_err(|e| WasmOptError::Io(e.to_string()))?;
    let out_file =
        tempfile::NamedTempFile::new().map_err(|e| WasmOptError::Io(e.to_string()))?;

    let mut cmd = std::process::Command::new("wasm-opt");
    cmd.arg(level.flag());
    if strip_debug {
        cmd.arg("--strip-debug");
    }
    cmd.arg("--vacuum");
    cmd.arg(in_file.path());
    cmd.arg("-o").arg(out_file.path());

    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(WasmOptError::NotInstalled);
        }
        Err(e) => return Err(WasmOptError::Io(e.to_string())),
    };

    if !status.success() {
        return Err(WasmOptError::ExitNonZero(status.code().unwrap_or(-1)));
    }

    let optimized =
        std::fs::read(out_file.path()).map_err(|e| WasmOptError::Io(e.to_string()))?;
    let after = optimized.len();
    Ok((optimized, WasmSizeReport { before, after }))
}

/// Try wasm-opt; fall back gracefully if not installed or if it fails.
/// Always returns a valid (bytes, report) pair.
pub fn try_wasm_opt(
    bytes: Vec<u8>,
    level: WasmOptLevel,
    strip_debug: bool,
) -> (Vec<u8>, WasmSizeReport) {
    let before = bytes.len();
    match run_wasm_opt(&bytes, level, strip_debug) {
        Ok(result) => result,
        Err(WasmOptError::NotInstalled) => {
            eprintln!(
                "[fav] wasm-opt not found; skipping optimization \
                 (install binaryen to enable: brew install binaryen)"
            );
            (bytes, WasmSizeReport { before, after: before })
        }
        Err(e) => {
            eprintln!("[fav] wasm-opt failed ({:?}); using unoptimized output", e);
            (bytes, WasmSizeReport { before, after: before })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_report_reduction_pct() {
        let r = WasmSizeReport { before: 1000, after: 600 };
        assert!((r.reduction_pct() - 40.0).abs() < 0.01);
    }

    #[test]
    fn size_report_zero_before() {
        let r = WasmSizeReport { before: 0, after: 0 };
        assert_eq!(r.reduction_pct(), 0.0);
    }

    #[test]
    fn o0_returns_input_unchanged() {
        let bytes = vec![0u8, b'a', b's', b'm'];
        let (out, report) = run_wasm_opt(&bytes, WasmOptLevel::O0, false).unwrap();
        assert_eq!(out, bytes);
        assert_eq!(report.before, 4);
        assert_eq!(report.after, 4);
    }

    #[test]
    fn try_wasm_opt_o0_succeeds() {
        let bytes = vec![0u8; 100];
        let (out, report) = try_wasm_opt(bytes.clone(), WasmOptLevel::O0, false);
        assert_eq!(out, bytes);
        assert_eq!(report.reduction_pct(), 0.0);
    }

    #[test]
    fn os_flag_is_minus_os() {
        assert_eq!(WasmOptLevel::Os.flag(), "-Os");
    }
}
