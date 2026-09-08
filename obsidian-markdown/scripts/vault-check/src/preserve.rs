//! Structural loss detection, not a semantic completeness proof.
use regex::Regex;
use std::{collections::BTreeMap, ffi::OsString, fs, process::ExitCode, sync::LazyLock};

static HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^ {0,3}(#{1,6})[ \t]+(.+?)(?:[ \t]+#+)?[ \t]*$").unwrap());
static WIKI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!?\[\[[^\]\n]+\]\]").unwrap());
static BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)\^([A-Za-z0-9-]+)\s*$").unwrap());
static EXAM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(GATE|ESE|IES|RRB(?:[ \t]+JE)?|SSC(?:[ \t]+JE)?|JKSSB|ISRO)[ \t]*(?:ME|JE|Prelims|Mains)?[ \t]*[-–—:,(]?[ \t]*((?:19|20)\d{2})\b").unwrap()
});

type Inventory = BTreeMap<(&'static str, String), usize>;
fn add(items: &mut Inventory, kind: &'static str, value: String) {
    *items.entry((kind, value)).or_default() += 1;
}

fn inventory(text: &str) -> Result<(Option<String>, Inventory), String> {
    let text = text.replace("\r\n", "\n");
    let lines: Vec<_> = text.lines().collect();
    let (frontmatter, start) = if lines.first().is_some_and(|s| s.trim() == "---") {
        let end = (1..lines.len())
            .find(|&i| lines[i].trim() == "---")
            .ok_or("unclosed frontmatter; comparison incomplete")?;
        (Some(lines[..=end].join("\n")), end + 1)
    } else {
        (None, 0)
    };
    let mut visible = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    for line in &lines[start..] {
        // Unwrap block quotes, including callout bodies, before recognizing fences.
        let mut line = *line;
        while let Some(rest) = line.trim_start().strip_prefix('>') {
            line = rest.strip_prefix(' ').unwrap_or(rest);
        }
        if let Some((ch, length)) = fence {
            let trimmed = line.trim_start();
            let n = trimmed.chars().take_while(|&c| c == ch).count();
            if n >= length && trimmed[n..].trim().is_empty() {
                fence = None;
            }
            visible.push("");
        } else if let Some(caps) = crate::FENCE.captures(line) {
            let run = caps.get(1).unwrap().as_str();
            fence = Some((run.chars().next().unwrap(), run.len()));
            visible.push("");
        } else {
            visible.push(line);
        }
    }
    if fence.is_some() {
        return Err("unclosed code fence; comparison incomplete".into());
    }
    let mut items = Inventory::new();
    for (i, line) in visible.iter().enumerate() {
        if let Some(caps) = HEADING.captures(line) {
            add(
                &mut items,
                "heading",
                format!("{} {}", &caps[1], caps[2].trim()),
            );
        } else if i > 0 && !visible[i - 1].trim().is_empty() {
            let marker = line.trim();
            if marker.len() >= 3
                && (marker.chars().all(|c| c == '=') || marker.chars().all(|c| c == '-'))
            {
                let level = if marker.starts_with('=') { "#" } else { "##" };
                add(
                    &mut items,
                    "heading",
                    format!("{level} {}", visible[i - 1].trim()),
                );
            }
        }
    }
    let body = crate::mask_code(&visible.join("\n"));
    for m in WIKI.find_iter(&body) {
        let escapes = body.as_bytes()[..m.start()]
            .iter()
            .rev()
            .take_while(|&&b| b == b'\\')
            .count();
        if escapes % 2 != 0 {
            continue;
        }
        let kind = if m.as_str().starts_with('!') {
            "embed"
        } else {
            "wikilink"
        };
        add(&mut items, kind, m.as_str().to_owned());
    }
    for line in body.lines() {
        if let Some(caps) = BLOCK.captures(line) {
            add(&mut items, "block ID", caps[1].to_owned());
        }
        for caps in EXAM.captures_iter(line) {
            let exam = caps[1]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_uppercase();
            add(&mut items, "exam reference", format!("{exam} {}", &caps[2]));
        }
    }
    Ok((frontmatter, items))
}

pub fn compare(before: &str, after: &str) -> Result<Vec<String>, String> {
    let (old_frontmatter, old) = inventory(before).map_err(|e| format!("BEFORE: {e}"))?;
    let (new_frontmatter, new) = inventory(after).map_err(|e| format!("AFTER: {e}"))?;
    let mut findings = Vec::new();
    if old_frontmatter != new_frontmatter {
        findings.push("frontmatter changed (text comparison; inspect the diff)".into());
    }
    for ((kind, value), count) in old {
        let remaining = new.get(&(kind, value.clone())).copied().unwrap_or(0);
        if remaining < count {
            findings.push(format!("{kind} count {count} -> {remaining}: {value:?}"));
        }
    }
    Ok(findings)
}

pub fn run(args: impl Iterator<Item = OsString>) -> ExitCode {
    let (mut quiet, mut literal) = (false, false);
    let mut paths = Vec::new();
    for arg in args {
        if !literal && arg == "--quiet" {
            quiet = true;
        } else if !literal && arg == "--" {
            literal = true;
        } else if !literal && (arg == "--help" || arg == "-h") {
            println!("Usage: vault-check preserve [--quiet] [--] BEFORE.md AFTER.md\nRead-only structural comparison; additions/reordering allowed.\nExit: 0 no findings, 1 review findings, 2 read/structure/usage error.\nDoes not prove semantic completeness. See obsidian-markdown/SKILL.md for coverage.");
            return ExitCode::SUCCESS;
        } else if !literal && arg.to_string_lossy().starts_with('-') {
            eprintln!("Unknown option: {}", arg.to_string_lossy());
            return ExitCode::from(2);
        } else {
            paths.push(arg);
        }
    }
    if paths.len() != 2 {
        eprintln!("Usage: vault-check preserve [--quiet] [--] BEFORE.md AFTER.md");
        return ExitCode::from(2);
    }
    let read =
        |p: &OsString| fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.to_string_lossy()));
    let result = read(&paths[0])
        .and_then(|before| read(&paths[1]).and_then(|after| compare(&before, &after)));
    match result {
        Err(e) => {
            eprintln!("ERROR {e}");
            ExitCode::from(2)
        }
        Ok(findings) if findings.is_empty() => {
            if !quiet {
                println!("OK no tracked structural losses; semantic completeness was not checked");
            }
            ExitCode::SUCCESS
        }
        Ok(findings) => {
            for finding in findings {
                println!("REVIEW {finding}");
            }
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compare;
    #[test]
    fn additions_and_reordering_are_allowed() {
        let a = "---\ntags: [x]\n---\n# A\n![[x.svg]]\n[[B]]\ntext ^id\nESE 2020\n## C";
        let b =
            "---\ntags: [x]\n---\n## C\n# A\nESE 2020\ntext ^id\n[[B]]\n![[x.svg]]\nNew text\n## D";
        assert!(compare(a, b).unwrap().is_empty());
        assert!(compare(a, &a.replace('\n', "\r\n")).unwrap().is_empty());
    }
    #[test]
    fn detects_losses_and_duplicate_counts() {
        let a = "---\na: 1\n---\n# Old\n![[x.svg|400]]\n[[B#H]] [[B#H]]\ntext ^id\nGATE ME 2023";
        let b = "---\na: 2\n---\n# New\n[[B#H]]";
        let result = compare(a, b).unwrap();
        for kind in [
            "frontmatter",
            "heading",
            "embed",
            "wikilink count 2 -> 1",
            "block ID",
            "exam reference",
        ] {
            assert!(
                result.iter().any(|s| s.contains(kind)),
                "{kind}: {result:?}"
            );
        }
    }
    #[test]
    fn ignores_examples_but_retains_callout_content() {
        let a = "```md\n# Fake\n[[Fake]] ESE 2020\n```\n`[[Literal]]`\n> ```\n> [[Fake]]\n> ```\n\\[[Escaped]]";
        assert!(compare(a, "").unwrap().is_empty());
        assert!(!compare("> [!note]\n> [[Real]]", "").unwrap().is_empty());
    }
    #[test]
    fn incomplete_structure_is_not_a_clean_check() {
        assert!(compare("---\na: 1", "").is_err());
        assert!(compare("", "```\nhidden").is_err());
    }
    #[test]
    fn setext_headings_and_code_titles() {
        assert!(compare("Title\n===", "# Title").unwrap().is_empty());
        assert!(!compare("# Use `x`", "# Use `y`").unwrap().is_empty());
        assert!(!compare("# C#", "# C").unwrap().is_empty());
        assert!(compare("# Title ###", "# Title").unwrap().is_empty());
    }
}
