//! Minimal `.baml` reader sufficient for parity (spec 0019).
//!
//! Parses `enum Name { ... }` and `class Name { field type ... }`. Function
//! bodies (`function ... { ... }`) are skipped — they are offline-generation
//! contracts, not runtime code.

use std::path::Path;

use regex::Regex;

/// Where a parsed type lives: the `class` or `enum` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BamlKind {
    Class,
    Enum,
}

/// One parsed `.baml` `class` or `enum`.
#[derive(Debug, Clone)]
pub struct BamlType {
    pub name: String,
    /// Stem of the source file: `swe_seed` | `harness` | `fabricator`.
    pub source_file: String,
    pub kind: BamlKind,
    /// Class field names (declaration order).
    pub fields: Vec<String>,
    /// Raw `.baml` type strings, parallel to `fields` (e.g. `string[]`, `LayerName`).
    pub field_types: Vec<String>,
    /// Enum variant names (declaration order).
    pub variants: Vec<String>,
}

impl BamlType {
    pub fn field_set(&self) -> Vec<&str> {
        self.fields.iter().map(String::as_str).collect()
    }
    pub fn variant_set(&self) -> Vec<&str> {
        self.variants.iter().map(String::as_str).collect()
    }
}

/// Parse all `*.baml` files under `dir` (stem used as `source_file`).
///
/// Propagates filesystem failures instead of masking them as an empty schema
/// set: an unreadable directory or file is a real error, not "zero types".
pub fn parse_baml_dir(dir: &Path) -> std::io::Result<Vec<BamlType>> {
    let mut out = Vec::new();
    let mut files: Vec<_> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let p = entry?.path();
        if p.extension().is_some_and(|x| x == "baml") {
            files.push(p);
        }
    }
    files.sort();
    for f in files {
        let stem = f
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let src = std::fs::read_to_string(&f)?;
        out.extend(parse_baml_src(&src, &stem));
    }
    Ok(out)
}

/// Parse `.baml` source text. `source_file` tags each returned type.
pub fn parse_baml_src(src: &str, source_file: &str) -> Vec<BamlType> {
    let cleaned = strip_strings_and_comments(src);
    let mut types = Vec::new();

    let block_start = Regex::new(r"(?m)^\s*(class|enum|function)\s+([A-Za-z_][A-Za-z0-9_]*)")
        .expect("static regex");

    for caps in block_start.captures_iter(&cleaned) {
        let kind_kw = caps.get(1).unwrap().as_str();
        let name = caps.get(2).unwrap().as_str().to_string();
        let after = caps.get(0).unwrap().end();
        let Some(open) = cleaned[after..].find('{') else {
            continue;
        };
        let open_abs = after + open;
        let Some(close_abs) = match_brace(&cleaned, open_abs) else {
            continue;
        };
        let body = &cleaned[open_abs + 1..close_abs];

        if kind_kw == "function" {
            continue; // offline contract, not runtime code
        }
        if kind_kw == "enum" {
            let variants = body
                .lines()
                .map(str::trim)
                .map(|l| l.trim_end_matches(','))
                .filter(|l| !l.is_empty())
                .map(|l| l.split_whitespace().next().unwrap_or(l).to_string())
                .collect();
            types.push(BamlType {
                name,
                source_file: source_file.to_string(),
                kind: BamlKind::Enum,
                fields: Vec::new(),
                field_types: Vec::new(),
                variants,
            });
        } else {
            let (fields, field_types) = parse_fields(body);
            types.push(BamlType {
                name,
                source_file: source_file.to_string(),
                kind: BamlKind::Class,
                fields,
                field_types,
                variants: Vec::new(),
            });
        }
    }
    types
}

/// Split a class body into `(field_names, raw_type_strings)`.
fn parse_fields(body: &str) -> (Vec<String>, Vec<String>) {
    let mut fields = Vec::new();
    let mut types = Vec::new();
    for line in body.lines() {
        let line = line.trim().trim_end_matches(',');
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(name) = parts.next() else { continue };
        // Skip stray braces or non-identifier noise.
        if !name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            continue;
        }
        let ty = parts.collect::<Vec<_>>().join(" ");
        fields.push(name.to_string());
        types.push(ty);
    }
    (fields, types)
}

/// Index of the `}` matching the `{` at `open`, tracking depth.
/// Safe to depth-count because strings are already stripped.
fn match_brace(s: &str, open: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Remove raw strings (`#"..."#`), normal strings (`"..."`), and `//` line
/// comments so brace/keyword scanning is unambiguous. Field/variant/class names
/// are identifiers and never live inside strings, so dropping them is lossless
/// for parity purposes.
fn strip_strings_and_comments(src: &str) -> String {
    // r##"..."## so the regex bodies can themselves contain "#"/" chars safely.
    let raw = Regex::new(r##"#"[\s\S]*?"#"##).expect("static regex");
    let norm = Regex::new(r##""(?:\\.|[^"\\])*""##).expect("static regex");
    let comment = Regex::new(r"//[^\n]*").expect("static regex");
    let s = raw.replace_all(src, "");
    let s = norm.replace_all(&s, "");
    comment.replace_all(&s, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_enum_and_class() {
        // r## so the baml prompt body ("#...") does not close the raw string.
        let src = r##"
            enum Color {
              Red
              Green
              Blue
            }
            class Point {
              x int
              y int
              tags string[]
            }
            function Skip(x: int) -> int {
              client "x"
              prompt #"ignore { this } "#
            }
        "##;
        let types = parse_baml_src(src, "test");
        assert_eq!(types.len(), 2);
        let color = types.iter().find(|t| t.name == "Color").unwrap();
        assert_eq!(color.variant_set(), vec!["Red", "Green", "Blue"]);
        let point = types.iter().find(|t| t.name == "Point").unwrap();
        assert_eq!(point.field_set(), vec!["x", "y", "tags"]);
        assert_eq!(point.field_types[2], "string[]");
    }
}
