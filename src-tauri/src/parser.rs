use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedVariable {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,
    pub line: usize,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub level: DiagnosticLevel,
    pub line: usize,
    pub message: String,
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ParseResult {
    pub variables: Vec<ParsedVariable>,
    pub diagnostics: Vec<ParseDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct ExportVariable {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,
}

pub fn parse_env(input: &str) -> ParseResult {
    let mut result = ParseResult::default();
    let mut pending_comments: Vec<String> = Vec::new();
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut position: i64 = 0;

    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line_no = i + 1;
        let raw = lines[i];
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            pending_comments.clear();
            i += 1;
            continue;
        }

        if trimmed.starts_with('#') {
            let comment = trimmed
                .trim_start_matches('#')
                .trim_start_matches(' ')
                .to_string();
            pending_comments.push(comment);
            i += 1;
            continue;
        }

        match parse_assignment(&lines, i) {
            Ok((key, value, consumed)) => {
                let comment = if pending_comments.is_empty() {
                    None
                } else {
                    Some(pending_comments.join("\n"))
                };
                pending_comments.clear();

                if let Some(prev_index) = seen.get(&key).copied() {
                    result.diagnostics.push(ParseDiagnostic {
                        level: DiagnosticLevel::Warning,
                        line: line_no,
                        message: format!("Duplicate key '{key}': last value wins"),
                        key: Some(key.clone()),
                    });
                    result.variables[prev_index].value = value;
                    result.variables[prev_index].line = line_no;
                    result.variables[prev_index].comment = comment;
                } else {
                    seen.insert(key.clone(), result.variables.len());
                    result.variables.push(ParsedVariable {
                        key,
                        value,
                        comment,
                        line: line_no,
                        position,
                    });
                    position += 1;
                }
                i += consumed;
            }
            Err(message) => {
                pending_comments.clear();
                result.diagnostics.push(ParseDiagnostic {
                    level: DiagnosticLevel::Error,
                    line: line_no,
                    message,
                    key: None,
                });
                i += 1;
            }
        }
    }

    result
}

fn parse_assignment(lines: &[&str], start: usize) -> Result<(String, String, usize), String> {
    let raw = lines[start];
    let trimmed = raw.trim();
    let without_export = trimmed.strip_prefix("export ").unwrap_or(trimmed).trim_start();

    let eq = without_export
        .find('=')
        .ok_or_else(|| format!("Invalid line: missing '='"))?;

    let key = without_export[..eq].trim();
    if !is_valid_key(key) {
        return Err(format!("Invalid key '{key}': expected [A-Za-z_][A-Za-z0-9_]*"));
    }

    let rhs = &without_export[eq + 1..];
    let rhs_trimmed = rhs.trim_start_matches([' ', '\t']);

    if rhs_trimmed.is_empty() {
        return Ok((key.to_string(), String::new(), 1));
    }

    match rhs_trimmed.as_bytes()[0] {
        b'"' => parse_double_quoted(lines, start, key, rhs_trimmed),
        b'\'' => parse_single_quoted(lines, start, key, rhs_trimmed),
        _ => parse_unquoted(key, rhs_trimmed),
    }
}

fn parse_double_quoted(
    lines: &[&str],
    start: usize,
    key: &str,
    rhs_trimmed: &str,
) -> Result<(String, String, usize), String> {
    let mut value = String::new();
    let mut i = start;
    let mut line = rhs_trimmed;
    let mut first = true;

    loop {
        let content = if first {
            first = false;
            &line[1..] // skip opening quote
        } else {
            line
        };

        let mut chars = content.char_indices();
        let mut closed = false;
        while let Some((idx, ch)) = chars.next() {
            match ch {
                '\\' => {
                    let Some((_, esc)) = chars.next() else {
                        value.push('\\');
                        continue;
                    };
                    match esc {
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        '"' => value.push('"'),
                        '\\' => value.push('\\'),
                        '$' => value.push('$'),
                        other => {
                            value.push('\\');
                            value.push(other);
                        }
                    }
                    let _ = idx;
                }
                '"' => {
                    let rest = &content[idx + 1..];
                    if rest.trim().is_empty() || rest.trim_start().starts_with('#') {
                        closed = true;
                        break;
                    }
                    return Err(format!(
                        "Invalid value for '{key}': unexpected content after closing quote"
                    ));
                }
                other => value.push(other),
            }
        }

        if closed {
            return Ok((key.to_string(), value, i - start + 1));
        }

        i += 1;
        if i >= lines.len() {
            return Err(format!("Invalid value for '{key}': unterminated double quote"));
        }
        value.push('\n');
        line = lines[i];
    }
}

fn parse_single_quoted(
    lines: &[&str],
    start: usize,
    key: &str,
    rhs_trimmed: &str,
) -> Result<(String, String, usize), String> {
    let mut value = String::new();
    let mut i = start;
    let mut line = rhs_trimmed;
    let mut first = true;

    loop {
        let content = if first {
            first = false;
            &line[1..]
        } else {
            line
        };

        if let Some(idx) = content.find('\'') {
            value.push_str(&content[..idx]);
            let rest = &content[idx + 1..];
            if rest.trim().is_empty() || rest.trim_start().starts_with('#') {
                return Ok((key.to_string(), value, i - start + 1));
            }
            return Err(format!(
                "Invalid value for '{key}': unexpected content after closing quote"
            ));
        }

        value.push_str(content);
        i += 1;
        if i >= lines.len() {
            return Err(format!("Invalid value for '{key}': unterminated single quote"));
        }
        value.push('\n');
        line = lines[i];
    }
}

fn parse_unquoted(key: &str, rhs_trimmed: &str) -> Result<(String, String, usize), String> {
    let mut end = rhs_trimmed.len();
    let bytes = rhs_trimmed.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'#' && (i == 0 || bytes[i - 1] == b' ' || bytes[i - 1] == b'\t') {
            end = i;
            break;
        }
    }
    let value = rhs_trimmed[..end].trim_end_matches([' ', '\t']).to_string();
    Ok((key.to_string(), value, 1))
}

fn is_valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn serialize_env(variables: &[ExportVariable]) -> String {
    let mut out = String::new();
    for (i, var) in variables.iter().enumerate() {
        if let Some(comment) = &var.comment {
            for line in comment.lines() {
                if line.is_empty() {
                    out.push_str("#\n");
                } else {
                    out.push_str("# ");
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }

        out.push_str(&var.key);
        out.push('=');
        out.push_str(&serialize_value(&var.value));
        out.push('\n');

        if i + 1 < variables.len() {
            // keep output compact; comments provide separation naturally
        }
    }
    out
}

fn serialize_value(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    let needs_quotes = value.contains(' ')
        || value.contains('\t')
        || value.contains('#')
        || value.contains('"')
        || value.contains('\'')
        || value.contains('\n')
        || value.contains('\r')
        || value.starts_with(' ')
        || value.ends_with(' ')
        || value.starts_with('$')
        || value.contains('\\');

    if !needs_quotes {
        return value.to_string();
    }

    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '$' => out.push_str("\\$"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_and_export() {
        let input = "FOO=bar\nexport BAZ=qux\n";
        let res = parse_env(input);
        assert!(res.diagnostics.is_empty());
        assert_eq!(res.variables.len(), 2);
        assert_eq!(res.variables[0].key, "FOO");
        assert_eq!(res.variables[0].value, "bar");
        assert_eq!(res.variables[1].key, "BAZ");
        assert_eq!(res.variables[1].value, "qux");
    }

    #[test]
    fn handles_comments_blank_lines_and_attached_comment() {
        let input = "# app config\n# keep secret\nAPI_KEY=abc\n\n# ignored after blank\n\nDB_HOST=localhost\n";
        let res = parse_env(input);
        assert_eq!(res.variables.len(), 2);
        assert_eq!(res.variables[0].comment.as_deref(), Some("app config\nkeep secret"));
        assert_eq!(res.variables[1].comment, None);
    }

    #[test]
    fn handles_quotes_escapes_hash_and_empty() {
        let input = "A='literal # value'\nB=\"hello # world\"\nC=\"line1\\nline2\\t\\\"q\\\"\\\\\"\nD=\nE=\"\"\n";
        let res = parse_env(input);
        assert!(res.diagnostics.iter().all(|d| d.level == DiagnosticLevel::Warning));
        assert_eq!(res.variables[0].value, "literal # value");
        assert_eq!(res.variables[1].value, "hello # world");
        assert_eq!(res.variables[2].value, "line1\nline2\t\"q\"\\");
        assert_eq!(res.variables[3].value, "");
        assert_eq!(res.variables[4].value, "");
    }

    #[test]
    fn handles_unquoted_inline_comment() {
        let input = "FOO=bar # comment\nBAZ=qux#not-comment\n";
        let res = parse_env(input);
        assert_eq!(res.variables[0].value, "bar");
        assert_eq!(res.variables[1].value, "qux#not-comment");
    }

    #[test]
    fn handles_multiline_double_quotes() {
        let input = "PRIVATE_KEY=\"-----BEGIN KEY-----\nabc\n-----END KEY-----\"\nNEXT=1\n";
        let res = parse_env(input);
        assert!(res.diagnostics.is_empty());
        assert_eq!(res.variables[0].key, "PRIVATE_KEY");
        assert!(res.variables[0].value.contains("\nabc\n"));
        assert_eq!(res.variables[1].key, "NEXT");
    }

    #[test]
    fn duplicate_keys_last_wins_with_warning() {
        let input = "A=1\nA=2\n";
        let res = parse_env(input);
        assert_eq!(res.variables.len(), 1);
        assert_eq!(res.variables[0].value, "2");
        assert!(res.diagnostics.iter().any(|d| d.level == DiagnosticLevel::Warning && d.message.contains("Duplicate key")));
    }

    #[test]
    fn invalid_lines_reported_without_crash() {
        let input = "GOOD=1\nBROKEN\nBAD-KEY=x\n=novalue\nALSO_GOOD=2\n";
        let res = parse_env(input);
        assert_eq!(res.variables.len(), 2);
        assert_eq!(res.variables[0].key, "GOOD");
        assert_eq!(res.variables[1].key, "ALSO_GOOD");
        let errors: Vec<_> = res.diagnostics.iter().filter(|d| d.level == DiagnosticLevel::Error).collect();
        assert_eq!(errors.len(), 3);
        assert!(errors.iter().any(|e| e.line == 2));
        assert!(errors.iter().any(|e| e.line == 3));
        assert!(errors.iter().any(|e| e.line == 4));
    }

    #[test]
    fn serializer_quotes_when_needed_and_preserves_comments() {
        let vars = vec![
            ExportVariable { key: "A".into(), value: "simple".into(), comment: Some("head".into()) },
            ExportVariable { key: "B".into(), value: "has space".into(), comment: None },
            ExportVariable { key: "C".into(), value: "hash # value".into(), comment: None },
            ExportVariable { key: "D".into(), value: "line1\nline2".into(), comment: None },
            ExportVariable { key: "E".into(), value: "".into(), comment: None },
        ];
        let out = serialize_env(&vars);
        assert!(out.contains("# head\nA=simple\n"));
        assert!(out.contains("B=\"has space\"\n"));
        assert!(out.contains("C=\"hash # value\"\n"));
        assert!(out.contains("D=\"line1\\nline2\"\n"));
        assert!(out.contains("E=\n"));
    }

    #[test]
    fn parse_serialize_parse_round_trip() {
        let input = "# comment\nFOO=bar\nSPACED=\"hello world\"\nMULTI=\"a\\nb\"\nEMPTY=\n";
        let parsed = parse_env(input);
        let exported: Vec<ExportVariable> = parsed
            .variables
            .iter()
            .map(|v| ExportVariable { key: v.key.clone(), value: v.value.clone(), comment: v.comment.clone() })
            .collect();
        let out = serialize_env(&exported);
        let reparsed = parse_env(&out);
        assert_eq!(parsed.variables.len(), reparsed.variables.len());
        for (a, b) in parsed.variables.iter().zip(reparsed.variables.iter()) {
            assert_eq!(a.key, b.key);
            assert_eq!(a.value, b.value);
            assert_eq!(a.comment, b.comment);
        }
    }
}
