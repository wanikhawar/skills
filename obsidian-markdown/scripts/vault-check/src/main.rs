//! Read-only, conservative Obsidian Markdown validation.
mod preserve;
use regex::Regex;
use std::io::Write;
use std::{env, fs, process::ExitCode, sync::LazyLock};
use yaml_rust2::parser::{Event, EventReceiver, Parser};
use yaml_rust2::{Yaml, YamlLoader};

#[derive(Default)]
struct SpecialYaml(bool);
impl EventReceiver for SpecialYaml {
    fn on_event(&mut self, event: Event) {
        if matches!(
            event,
            Event::Alias(_)
                | Event::Scalar(_, _, _, Some(_))
                | Event::SequenceStart(_, Some(_))
                | Event::MappingStart(_, Some(_))
        ) {
            self.0 = true;
        }
    }
}
fn complex_keys(value: &Yaml) -> bool {
    match value {
        Yaml::Hash(map) => map
            .iter()
            .any(|(k, v)| matches!(k, Yaml::Array(_) | Yaml::Hash(_)) || complex_keys(v)),
        Yaml::Array(items) => items.iter().any(complex_keys),
        _ => false,
    }
}
// Preserve PyYAML constructor semantics for uncommon tags, aliases and complex
// keys without paying Python startup costs for ordinary note frontmatter.
fn legacy_yaml(source: &str) -> Result<(), String> {
    let mut child = std::process::Command::new("python3")
        .args(["-c", "import sys, yaml\ntry:\n d=yaml.safe_load(sys.stdin.read())\n if d is not None and not isinstance(d,dict): raise ValueError('frontmatter must be a YAML mapping')\nexcept Exception as e:\n print(str(e)); sys.exit(1)"])
        .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped())
        .spawn().map_err(|e| format!("special YAML requires Python 3 and PyYAML: {e}"))?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .map_err(|e| e.to_string())?;
    let result = child.wait_with_output().map_err(|e| e.to_string())?;
    if result.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{}{}",
            String::from_utf8_lossy(&result.stdout).trim(),
            String::from_utf8_lossy(&result.stderr).trim()
        ))
    }
}
fn frontmatter(source: &str) -> Result<(), String> {
    let mut special = SpecialYaml::default();
    if Parser::new_from_str(source)
        .load(&mut special, true)
        .is_err()
    {
        return legacy_yaml(source).map_err(|e| format!("invalid frontmatter YAML: {e}"));
    }
    let docs = match YamlLoader::load_from_str(source) {
        Ok(docs) => docs,
        Err(_) => return legacy_yaml(source).map_err(|e| format!("invalid frontmatter YAML: {e}")),
    };
    if special.0 || docs.iter().any(complex_keys) {
        return legacy_yaml(source).map_err(|e| format!("invalid frontmatter YAML: {e}"));
    }
    if docs.len() > 1 {
        return Err("invalid frontmatter YAML: expected a single document".into());
    }
    if docs
        .first()
        .is_some_and(|d| !matches!(d, Yaml::Hash(_) | Yaml::Null))
    {
        return Err("frontmatter must be a YAML mapping".into());
    }
    Ok(())
}

static FENCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*(`{3,}|~{3,})(.*)$").unwrap());
static TABLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*\|?\s*:?-{3,}:?\s*(?:\|\s*:?-{3,}:?\s*)+\|?\s*$").unwrap());
static DISPLAY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)\$\$.*?\$\$").unwrap());

fn mask_code(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut runs = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i] == b'`' {
            i += 1;
        }
        runs.push((start, i));
    }
    let mut result = bytes.to_vec();
    let mut i = 0;
    while i < runs.len() {
        let (start, end) = runs[i];
        let slashes = bytes[..start]
            .iter()
            .rev()
            .take_while(|&&b| b == b'\\')
            .count();
        if slashes % 2 == 0 {
            if let Some(j) = (i + 1..runs.len()).find(|&j| runs[j].1 - runs[j].0 == end - start) {
                for byte in &mut result[start..runs[j].1] {
                    if *byte != b'\n' {
                        *byte = b' ';
                    }
                }
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    String::from_utf8(result).expect("mask replaces whole UTF-8 spans")
}

fn pipes(line: &str) -> usize {
    let (mut count, mut escaped) = (0, false);
    for c in line.chars() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '|' {
            count += 1;
        }
    }
    count
}

fn validate(text: &str) -> (Vec<String>, Vec<String>) {
    let (mut errors, mut warnings) = (Vec::new(), Vec::new());
    // Match Python splitlines, including Unicode line separators and CRLF.
    let normalized = text.replace("\r\n", "\n");
    let mut lines: Vec<&str> = normalized
        .split([
            '\n', '\r', '\u{b}', '\u{c}', '\u{1c}', '\u{1d}', '\u{1e}', '\u{85}', '\u{2028}',
            '\u{2029}',
        ])
        .collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut body_start = 0;
    if lines.first().is_some_and(|s| s.trim() == "---") {
        if let Some(end) = (1..lines.len()).find(|&i| lines[i].trim() == "---") {
            body_start = end + 1;
            if let Err(e) = frontmatter(&lines[1..end].join("\n")) {
                errors.push(e);
            }
        } else {
            errors.push("frontmatter opens with --- but has no closing ---".into());
        }
    }
    let mut visible = Vec::new();
    let mut opener: Option<(char, usize, usize)> = None;
    for (i, line) in lines[body_start..].iter().enumerate() {
        if let Some((ch, length, _)) = opener {
            let stripped = line.trim_start();
            let run = stripped.chars().take_while(|&c| c == ch).count();
            if run >= length && stripped[run..].trim().is_empty() {
                opener = None;
            }
            visible.push("");
        } else if let Some(caps) = FENCE.captures(line) {
            let fence = caps.get(1).unwrap().as_str();
            opener = Some((fence.chars().next().unwrap(), fence.len(), i + 1));
            visible.push("");
        } else {
            visible.push(line);
        }
    }
    if let Some((_, _, line)) = opener {
        errors.push(format!(
            "unclosed code fence beginning near body line {line}"
        ));
    }
    let text = mask_code(&visible.join("\n"));
    if text.contains("\\[") || text.contains("\\]") {
        errors.push("bracket-delimited display math found; use $$ delimiters".into());
    }
    if !text.matches("$$").count().is_multiple_of(2) {
        errors.push("unbalanced $$ display-math delimiters".into());
    }
    let math_removed = DISPLAY.replace_all(&text, "");
    let b = math_removed.as_bytes();
    let dollars = (0..b.len())
        .filter(|&i| b[i] == b'$' && (i == 0 || b[i - 1] != b'\\') && b.get(i + 1) != Some(&b'$'))
        .count();
    if dollars % 2 != 0 {
        warnings
            .push("odd number of inline $ delimiters; inspect for math/currency ambiguity".into());
    }
    if text.matches("[[").count() != text.matches("]]").count() {
        errors.push("unbalanced wikilink delimiters".into());
    }
    for (i, line) in visible.iter().enumerate() {
        if !TABLE.is_match(line) {
            continue;
        }
        let expected = pipes(line);
        let mut candidates = Vec::new();
        if i > 0 {
            candidates.push((i, visible[i - 1]));
        }
        let mut j = i + 1;
        while j < visible.len() && visible[j].contains('|') && !visible[j].trim().is_empty() {
            candidates.push((j + 1, visible[j]));
            j += 1;
        }
        for (line_no, candidate) in candidates {
            let found = pipes(candidate);
            if found != expected {
                errors.push(format!("table near body line {}: line {line_no} has {found} unescaped pipes; expected {expected}", i+1));
            }
        }
    }
    errors.sort();
    errors.dedup();
    warnings.sort();
    warnings.dedup();
    (errors, warnings)
}

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let mode = args.next().unwrap_or_default();
    if mode == "preserve" {
        return preserve::run(args);
    }
    if mode == "--help" || mode == "-h" {
        println!("Usage: vault-check note [--quiet] [--] FILE...\nAlso: vault-check preserve [--quiet] [--] BEFORE.md AFTER.md\nRead-only static Markdown checks. --quiet suppresses OK lines; warnings remain.\nExit codes: 0 no errors, 1 validation/read errors, 2 usage error.");
        return ExitCode::SUCCESS;
    }
    if mode != "note" {
        eprintln!("Usage: vault-check note [--quiet] [--] FILE...");
        return ExitCode::from(2);
    }
    let (mut quiet, mut literal, mut failed, mut count) = (false, false, false, 0);
    for arg in args {
        if !literal && arg == "--quiet" {
            quiet = true;
            continue;
        }
        if !literal && arg == "--" {
            literal = true;
            continue;
        }
        if !literal && arg.to_string_lossy().starts_with('-') {
            eprintln!("Unknown option: {}", arg.to_string_lossy());
            return ExitCode::from(2);
        }
        count += 1;
        let path = std::path::Path::new(&arg);
        let (errors, warnings) = match fs::read_to_string(path) {
            Ok(text) => validate(&text),
            Err(e) => (vec![format!("cannot read file: {e}")], vec![]),
        };
        for message in warnings {
            println!("WARNING {}: {message}", path.display());
        }
        for message in &errors {
            println!("ERROR {}: {message}", path.display());
        }
        if errors.is_empty() {
            if !quiet {
                println!("OK {}", path.display());
            }
        } else {
            failed = true;
        }
    }
    if count == 0 {
        eprintln!("At least one file is required");
        return ExitCode::from(2);
    }
    ExitCode::from(u8::from(failed))
}
