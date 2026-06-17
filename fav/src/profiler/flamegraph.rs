// v19.8.0 — SVG flamegraph generation via inferno.

/// Generate a flamegraph SVG from folded stack lines.
///
/// Each line must be in inferno folded format: `"parent;child weight"`
/// Returns the raw SVG bytes on success.
pub fn generate_svg(folded: &[String]) -> Result<Vec<u8>, String> {
    use inferno::flamegraph;

    if folded.is_empty() {
        return Ok(b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>".to_vec());
    }

    let mut opts = flamegraph::Options::default();
    let mut svg_buf = Vec::new();
    flamegraph::from_lines(&mut opts, folded.iter().map(|s| s.as_str()), &mut svg_buf)
        .map_err(|e| format!("flamegraph error: {e}"))?;
    Ok(svg_buf)
}
