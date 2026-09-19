//! Small read-only vault workflows. No index or background service.
use regex::Regex;
use std::{
    collections::HashSet,
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::SystemTime,
};
use yaml_rust2::{Yaml, YamlLoader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const HELP: &str = "vault [--root PATH] COMMAND\n\n  find QUERY [--limit N] [--paths] [--open N | --uri N]\n      Rank title, alias, path, then literal content matches; N is a 1-based result.\n  recent [--limit N] [--paths] [--open N | --uri N]\n      Notes ordered by filesystem modification time.\n  check --changed [--quiet]\n      Check staged, unstaged and untracked notes in the vault's Git repository.\n  check [--quiet] -- FILE...\n      Check explicit files (relative to current directory).\n\nRoot: --root, VAULT_ROOT, or nearest ancestor containing .obsidian.\nSearch is case-insensitive; all whitespace-separated terms must match.\nHidden directories, symlinks and node_modules are excluded from discovery.\ncheck reuses sibling vault-check and checks wiki embeds and inline Markdown images.\nEmbed checks verify files only, not headings/blocks or Obsidian plugin rendering.\nExit: 0 success, 1 findings/no search results, 2 usage or operational error.";

fn root(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = explicit.or_else(|| env::var_os("VAULT_ROOT").map(PathBuf::from)) {
        let p = p.canonicalize()?;
        if !p.join(".obsidian").is_dir() {
            return Err("root must contain .obsidian/".into());
        }
        return Ok(p);
    }
    env::current_dir()?
        .ancestors()
        .find(|p| p.join(".obsidian").is_dir())
        .map(Path::to_path_buf)
        .ok_or_else(|| "run inside a vault or use --root PATH".into())
}
fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') || name == "node_modules" {
            continue;
        }
        let ty = entry.file_type()?;
        if ty.is_dir() {
            walk(&entry.path(), files)?;
        } else if ty.is_file() {
            files.push(entry.path());
        }
    }
    Ok(())
}
fn is_note(p: &Path) -> bool {
    p.extension().is_some_and(|x| x.eq_ignore_ascii_case("md"))
}
fn aliases(text: &str) -> String {
    let mut lines = text.lines();
    if lines.next() != Some("---") {
        return String::new();
    }
    let yaml = lines
        .take_while(|l| *l != "---" && *l != "...")
        .collect::<Vec<_>>()
        .join("\n");
    let Ok(docs) = YamlLoader::load_from_str(&yaml) else {
        return String::new();
    };
    let Some(doc) = docs.first() else {
        return String::new();
    };
    match &doc["aliases"] {
        Yaml::String(s) => s.clone(),
        Yaml::Array(v) => v
            .iter()
            .filter_map(Yaml::as_str)
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}
fn score(
    title: &str,
    alias: &str,
    path: &str,
    body: &str,
    query: &str,
) -> Option<(u8, &'static str)> {
    let q = query.to_lowercase();
    let all = |s: &str| {
        let s = s.to_lowercase();
        q.split_whitespace().all(|t| s.contains(t))
    };
    if title.to_lowercase() == q {
        Some((0, "title"))
    } else if alias.lines().any(|a| a.to_lowercase() == q) {
        Some((1, "alias"))
    } else if all(title) {
        Some((2, "title"))
    } else if all(alias) {
        Some((3, "alias"))
    } else if all(path) {
        Some((4, "path"))
    } else if all(body) {
        Some((5, "content"))
    } else {
        None
    }
}
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
fn uri(path: &Path) -> String {
    format!("obsidian://open?path={}", encode(&path.to_string_lossy()))
}
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned().into());
    }
    Ok(out.stdout)
}
fn changed(root: &Path) -> Result<Vec<PathBuf>> {
    // --no-renames emits destinations as additions; NUL records preserve spaces/newlines.
    let top = git(root, &["rev-parse", "--show-toplevel"])?;
    let top = PathBuf::from(String::from_utf8(top)?.trim_end_matches('\n'));
    let mut paths = HashSet::new();
    for args in [
        vec![
            "diff",
            "--no-relative",
            "--name-only",
            "-z",
            "--no-renames",
            "--diff-filter=ACMT",
        ],
        vec![
            "diff",
            "--no-relative",
            "--cached",
            "--name-only",
            "-z",
            "--no-renames",
            "--diff-filter=ACMT",
        ],
        vec![
            "ls-files",
            "--others",
            "--exclude-standard",
            "--full-name",
            "-z",
        ],
    ] {
        for bytes in git(root, &args)?
            .split(|b| *b == 0)
            .filter(|b| !b.is_empty())
        {
            let p = top.join(std::str::from_utf8(bytes)?);
            if !is_note(&p) || !p.is_file() {
                continue;
            }
            let p = p.canonicalize()?;
            if p.starts_with(root)
                && !p
                    .strip_prefix(root)?
                    .components()
                    .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
            {
                paths.insert(p);
            }
        }
    }
    let mut paths: Vec<_> = paths.into_iter().collect();
    paths.sort();
    Ok(paths)
}
fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) =
                u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or(""), 16)
            {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
fn embed_findings(note: &Path, text: &str, root: &Path, files: &[PathBuf]) -> Vec<String> {
    let wiki = Regex::new(r"!\[\[([^\]\n]+)\]\]").unwrap();
    let md = Regex::new(r#"!\[[^\]\n]*\]\(\s*(?:<([^>\n]+)>|([^\s)]+))(?:\s+\"[^\"]*\")?\s*\)"#)
        .unwrap();
    let fence = Regex::new(r"^\s*(?:>\s*)*(`{3,}|~{3,})(.*)$").unwrap();
    let inline = Regex::new(r"(`+)[^`]*`+").unwrap();
    let mut findings = Vec::new();
    let mut fenced: Option<(char, usize)> = None;
    let mut front = text.starts_with("---\n") || text.starts_with("---\r\n");
    for (line_no, line) in text.lines().enumerate() {
        if front {
            if line_no > 0 && matches!(line, "---" | "...") {
                front = false;
            }
            continue;
        }
        if let Some(c) = fence.captures(line) {
            let run = &c[1];
            let ch = run.chars().next().unwrap();
            match fenced {
                None => fenced = Some((ch, run.len())),
                Some((old, n)) if ch == old && run.len() >= n && c[2].trim().is_empty() => {
                    fenced = None
                }
                _ => (),
            }
            continue;
        }
        if fenced.is_some() {
            continue;
        }
        let line = inline.replace_all(line, "");
        let targets = wiki
            .captures_iter(&line)
            .map(|c| {
                (
                    c[1].split('|')
                        .next()
                        .unwrap()
                        .trim_end_matches('\\')
                        .to_owned(),
                    true,
                )
            })
            .chain(md.captures_iter(&line).map(|c| {
                (
                    c.get(1).or_else(|| c.get(2)).unwrap().as_str().to_owned(),
                    false,
                )
            }));
        for (target, wiki) in targets {
            let target = target.split('#').next().unwrap();
            if target.is_empty() || target.contains(":") || target.starts_with("//") {
                continue;
            }
            let target = if wiki {
                target.to_owned()
            } else {
                decode(target)
            };
            let candidates = if wiki && Path::new(&target).extension().is_none() {
                vec![target.clone(), format!("{target}.md")]
            } else {
                vec![target.clone()]
            };
            let direct = candidates.iter().any(|t| {
                files.contains(&root.join(t.trim_start_matches('/')))
                    || files.contains(
                        &note
                            .parent()
                            .unwrap()
                            .join(t)
                            .canonicalize()
                            .unwrap_or_default(),
                    )
            });
            let matches = if direct {
                1
            } else if wiki {
                files
                    .iter()
                    .filter(|p| {
                        candidates
                            .iter()
                            .any(|t| p.strip_prefix(root).unwrap().ends_with(t))
                    })
                    .count()
            } else {
                0
            };
            if matches != 1 {
                findings.push(format!(
                    "{}:{}: {} embed: {}",
                    note.strip_prefix(root).unwrap().display(),
                    line_no + 1,
                    if matches == 0 { "missing" } else { "ambiguous" },
                    target
                ));
            }
        }
    }
    findings
}
fn run() -> Result<u8> {
    let mut args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{HELP}");
        return Ok(0);
    }
    let mut explicit = None;
    if args.first().is_some_and(|a| a == "--root") {
        if args.len() < 3 {
            return Err("--root requires a path and command".into());
        }
        explicit = Some(PathBuf::from(args.remove(1)));
        args.remove(0);
    }
    let root = root(explicit)?;
    let mode = args.remove(0);
    let mut limit = 20usize;
    let mut paths_only = false;
    let mut open = None;
    let mut uri_only = false;
    let mut changed_only = false;
    let mut quiet = false;
    let mut positional = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--" {
            positional.extend_from_slice(&args[i + 1..]);
            break;
        }
        match a.to_str().unwrap_or("") {
            "--limit" | "--open" | "--uri" if mode == "find" || mode == "recent" => {
                i += 1;
                let n: usize = args
                    .get(i)
                    .ok_or("missing number")?
                    .to_str()
                    .ok_or("invalid number")?
                    .parse()?;
                if n == 0 {
                    return Err("number must be positive".into());
                }
                if a == "--limit" {
                    limit = n;
                } else {
                    open = Some(n);
                    uri_only = a == "--uri";
                }
            }
            "--paths" if mode == "find" || mode == "recent" => paths_only = true,
            "--changed" if mode == "check" => changed_only = true,
            "--quiet" if mode == "check" => quiet = true,
            s if s.starts_with('-') => return Err(format!("unknown option: {s}").into()),
            _ => positional.push(a.clone()),
        }
        i += 1;
    }
    if mode == "check" {
        if changed_only && !positional.is_empty() {
            return Err("choose --changed or explicit files".into());
        }
        let notes = if changed_only {
            changed(&root)?
        } else {
            if positional.is_empty() {
                return Err("check requires --changed or files".into());
            }
            positional
                .iter()
                .map(|p| fs::canonicalize(Path::new(p)))
                .collect::<std::io::Result<Vec<_>>>()?
        };
        for note in &notes {
            if !note.starts_with(&root) || !is_note(note) {
                return Err("check files must be Markdown inside the vault".into());
            }
        }
        if notes.is_empty() {
            if !quiet {
                println!("No changed notes.");
            }
            return Ok(0);
        }
        let validator = env::current_exe()?.with_file_name("vault-check");
        let mut cmd = Command::new(validator);
        cmd.arg("note");
        if quiet {
            cmd.arg("--quiet");
        }
        let status = cmd.arg("--").args(&notes).status().map_err(|e| {
            format!("cannot run sibling vault-check; build/install both binaries: {e}")
        })?;
        if !matches!(status.code(), Some(0 | 1)) {
            return Err("vault-check failed operationally".into());
        }
        let mut failed = !status.success();
        let mut files = Vec::new();
        walk(&root, &mut files)?;
        for note in &notes {
            for finding in embed_findings(note, &fs::read_to_string(note)?, &root, &files) {
                println!("{finding}");
                failed = true;
            }
        }
        if !quiet {
            println!(
                "Checked {} note(s); static syntax and embedded-file targets.",
                notes.len()
            );
        }
        return Ok(u8::from(failed));
    }
    if mode != "find" && mode != "recent" {
        return Err("unknown command; use --help".into());
    }
    if (mode == "find" && positional.len() != 1) || (mode == "recent" && !positional.is_empty()) {
        return Err("find requires one quoted query; recent takes no query".into());
    }
    let query = positional
        .first()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    if mode == "find" && query.trim().is_empty() {
        return Err("query cannot be empty".into());
    }
    let mut files = Vec::new();
    walk(&root, &mut files)?;
    let mut results = Vec::new();
    for p in files.into_iter().filter(|p| is_note(p)) {
        let relative = p.strip_prefix(&root)?.to_string_lossy().into_owned();
        if mode == "recent" {
            results.push((0, p.metadata()?.modified()?, relative, "modified", p));
        } else {
            let body = fs::read_to_string(&p)?;
            if let Some((rank, why)) = score(
                &p.file_stem().unwrap().to_string_lossy(),
                &aliases(&body),
                &relative,
                &body,
                &query,
            ) {
                results.push((rank, SystemTime::UNIX_EPOCH, relative, why, p));
            }
        }
    }
    results.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
    if let Some(n) = open {
        let selected = results
            .get(n - 1)
            .filter(|_| n <= limit)
            .ok_or("selected result is outside displayed range")?;
        let url = uri(&selected.4);
        if uri_only {
            println!("{url}");
        } else if !Command::new("xdg-open").arg(url).status()?.success() {
            return Err("xdg-open failed".into());
        }
    } else {
        for (i, (_, time, p, why, _)) in results.iter().take(limit).enumerate() {
            if paths_only {
                println!("{p}");
            } else if mode == "recent" {
                println!(
                    "{}. {}  ({}m ago)",
                    i + 1,
                    p,
                    time.elapsed().unwrap_or_default().as_secs() / 60
                );
            } else {
                println!("{}. {}  [{}]", i + 1, p, why);
            }
        }
        if results.is_empty() {
            eprintln!("No matching notes.");
        }
    }
    Ok(u8::from(results.is_empty()))
}
fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("vault: {e}");
            ExitCode::from(2)
        }
    }
}
