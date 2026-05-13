use std::collections::HashMap;

pub fn extract_doc_comments(source: &str) -> HashMap<String, String> {
    let mut docs = HashMap::new();
    let mut pending = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(comment) = trimmed.strip_prefix("//") {
            pending.push(comment.trim_start().to_string());
            continue;
        }

        if let Some(name) = extract_decl_name(trimmed) {
            if !pending.is_empty() {
                docs.insert(name, pending.join("\n"));
            }
            pending.clear();
            continue;
        }

        pending.clear();
    }

    docs
}

fn extract_decl_name(line: &str) -> Option<String> {
    let stripped = strip_decl_modifiers(line);
    for keyword in ["fn", "type", "stage", "trf", "seq", "flw", "interface"] {
        if let Some(rest) = stripped.strip_prefix(keyword) {
            let rest = rest.trim_start();
            let name: String = rest
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn strip_decl_modifiers(mut line: &str) -> &str {
    loop {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("public ") {
            line = rest;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("internal ") {
            line = rest;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("private ") {
            line = rest;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("async ") {
            line = rest;
            continue;
        }
        return trimmed;
    }
}

#[cfg(test)]
mod tests {
    use super::extract_doc_comments;

    #[test]
    fn extract_doc_comment_before_fn() {
        let docs = extract_doc_comments("// doc\nfn main() -> Int = 1");
        assert_eq!(docs.get("main"), Some(&"doc".to_string()));
    }

    #[test]
    fn extract_doc_comment_multiline() {
        let docs =
            extract_doc_comments("// line 1\n// line 2\ninterface Show { show: Self -> String }");
        assert_eq!(docs.get("Show"), Some(&"line 1\nline 2".to_string()));
    }
}
