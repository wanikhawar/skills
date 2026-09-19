use clap::{Parser, Subcommand};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::{Duration, UNIX_EPOCH},
};
use walkdir::WalkDir;
use yaml_rust2::{Yaml, YamlLoader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Expansions = BTreeMap<String, Vec<String>>;
const MAX_TEXT: u64 = 4 * 1024 * 1024;
const SCHEMA: i64 = 1;

#[derive(Parser)]
#[command(
    version,
    about = "Local, automatically refreshed fuzzy search for an Obsidian vault"
)]
struct Cli {
    /// Vault directory (otherwise VAULT_ROOT or nearest .obsidian ancestor).
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build/update the index. Search also does this automatically.
    Index {
        /// Re-read unchanged files as well.
        #[arg(long)]
        rebuild: bool,
    },
    /// Search titles, aliases, headings, paths and text with typo tolerance.
    Search {
        query: String,
        #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u32).range(1..=100))]
        limit: u32,
        #[arg(long)]
        json: bool,
        /// Emit concise JSON without redundant paths, URIs or scores.
        #[arg(long)]
        compact: bool,
        /// Require every non-stopword term in the same indexed section.
        #[arg(long)]
        all_terms: bool,
        /// Show multiple matching sections of the same file.
        #[arg(long)]
        sections: bool,
        /// Disable spelling correction (prefix matching remains enabled).
        #[arg(long)]
        no_fuzzy: bool,
    },
    /// Read current file contents with original line numbers.
    Read {
        path: PathBuf,
        #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..))]
        line: u32,
        #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u32).range(1..=2000))]
        lines: u32,
    },
    /// Open an exact vault file in Obsidian, or print its URI.
    Open {
        path: PathBuf,
        #[arg(long)]
        uri: bool,
    },
}

#[derive(Default, Serialize, Debug)]
struct Update {
    files: usize,
    updated: usize,
    removed: usize,
    text_files: usize,
    metadata_only: usize,
}

#[derive(Debug)]
struct Section {
    heading: String,
    line: usize,
    end_line: usize,
    body: String,
}

#[derive(Debug, Serialize)]
struct Hit {
    path: String,
    absolute_path: String,
    title: String,
    heading: String,
    line: usize,
    end_line: usize,
    snippet: String,
    wikilink: String,
    uri: String,
    content_indexed: bool,
    score: f64,
}

#[derive(Serialize)]
struct SearchOutput {
    query: String,
    root: String,
    refresh: Update,
    expansions: Expansions,
    results: Vec<Hit>,
}

fn vault_root(explicit: Option<PathBuf>) -> Result<PathBuf> {
    let root = if let Some(p) = explicit.or_else(|| env::var_os("VAULT_ROOT").map(PathBuf::from)) {
        p.canonicalize()?
    } else {
        env::current_dir()?
            .ancestors()
            .find(|p| p.join(".obsidian").is_dir())
            .ok_or("Run inside a vault or provide --root PATH")?
            .to_owned()
    };
    if !root.join(".obsidian").is_dir() {
        return Err("Vault root must contain .obsidian/".into());
    }
    Ok(root)
}

fn database(root: &Path) -> Result<Connection> {
    let dir = root.join(".agents/skills/vault-operator/scripts/vault-search");
    fs::create_dir_all(&dir)?;
    let db = Connection::open(dir.join("index.sqlite3"))?;
    db.busy_timeout(Duration::from_secs(30))?;
    let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version != 0 && version != SCHEMA {
        return Err("Unsupported index version; use the matching utility version".into());
    }
    db.execute_batch("PRAGMA foreign_keys=ON;
        CREATE TABLE IF NOT EXISTS files(path TEXT PRIMARY KEY, size INTEGER NOT NULL, modified TEXT NOT NULL, content_indexed INTEGER NOT NULL);
        CREATE VIRTUAL TABLE IF NOT EXISTS search USING fts5(path, title, aliases, heading, body, line UNINDEXED, end_line UNINDEXED, content_indexed UNINDEXED, tokenize='porter unicode61 remove_diacritics 2');
        CREATE VIRTUAL TABLE IF NOT EXISTS vocabulary USING fts5vocab(search, 'row');
        PRAGMA user_version=1;")?;
    Ok(db)
}

fn visible(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 {
        return true;
    }
    let name = entry.file_name().to_string_lossy();
    !name.starts_with('.') && !matches!(name.as_ref(), "node_modules" | "target" | "API KEYS")
}

fn text_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "md" | "markdown"
            | "txt"
            | "tex"
            | "bib"
            | "csv"
            | "tsv"
            | "json"
            | "yaml"
            | "yml"
            | "canvas"
            | "svg"
            | "html"
            | "htm"
            | "rst"
            | "org"
            | "rs"
            | "py"
            | "c"
            | "cpp"
            | "h"
            | "css"
            | "js"
            | "toml"
    )
}

fn frontmatter(lines: &[&str]) -> (usize, String) {
    if lines.first().map(|l| l.trim()) != Some("---") {
        return (0, String::new());
    }
    let Some(end) = (1..lines.len()).find(|&i| matches!(lines[i].trim(), "---" | "...")) else {
        return (0, String::new());
    };
    let mut aliases = Vec::new();
    if let Ok(docs) = YamlLoader::load_from_str(&lines[1..end].join("\n")) {
        if let Some(doc) = docs.first() {
            for key in ["aliases", "alias", "tags"] {
                match &doc[key] {
                    Yaml::String(s) => aliases.push(s.clone()),
                    Yaml::Array(v) => {
                        aliases.extend(v.iter().filter_map(Yaml::as_str).map(str::to_owned))
                    }
                    _ => (),
                }
            }
        }
    }
    (end + 1, aliases.join("\n"))
}

fn sections(text: &str, markdown: bool) -> (String, Vec<Section>) {
    let lines: Vec<_> = text.lines().collect();
    let (front_end, aliases) = if markdown {
        frontmatter(&lines)
    } else {
        (0, String::new())
    };
    let mut starts = vec![(0, String::new())];
    let mut fence: Option<(char, usize)> = None;
    if markdown {
        for (i, line) in lines.iter().enumerate().skip(front_end) {
            let s = line.trim_start();
            let first = s.chars().next().unwrap_or(' ');
            let count = s.chars().take_while(|&c| c == first).count();
            if matches!(first, '`' | '~') && count >= 3 {
                match fence {
                    Some((c, n)) if c == first && count >= n && s[count..].trim().is_empty() => {
                        fence = None
                    }
                    None => fence = Some((first, count)),
                    _ => (),
                }
                continue;
            }
            if fence.is_none()
                && first == '#'
                && (1..=6).contains(&count)
                && s[count..].starts_with(char::is_whitespace)
            {
                let raw = s[count..].trim();
                let without_hashes = raw.trim_end_matches('#');
                let heading = if without_hashes.ends_with(char::is_whitespace) {
                    without_hashes.trim_end()
                } else {
                    raw
                }
                .to_owned();
                if i == 0 {
                    starts[0].1 = heading;
                } else {
                    starts.push((i, heading));
                }
            }
        }
    }
    let mut result = Vec::new();
    for (index, (start, heading)) in starts.iter().enumerate() {
        let end = starts.get(index + 1).map(|s| s.0).unwrap_or(lines.len());
        result.push(Section {
            heading: heading.clone(),
            line: start + 1,
            end_line: end.max(start + 1),
            body: lines[*start..end].join("\n"),
        });
    }
    (aliases, result)
}

fn refresh(db: &mut Connection, root: &Path, rebuild: bool) -> Result<Update> {
    // Enumerate before starting changes; failed traversal must never prune unseen files.
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(visible)
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    files.sort();
    let old: HashMap<String, (i64, String)> = db
        .prepare("SELECT path, size, modified FROM files")?
        .query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))?
        .collect::<std::result::Result<_, _>>()?;
    let tx = db.transaction()?;
    let mut seen = HashSet::new();
    let mut update = Update::default();
    for path in files {
        let relative = path
            .strip_prefix(root)?
            .to_str()
            .ok_or("Non-UTF-8 vault path is unsupported")?
            .replace('\\', "/");
        seen.insert(relative.clone());
        let meta = path.metadata()?;
        let modified = meta
            .modified()?
            .duration_since(UNIX_EPOCH)?
            .as_nanos()
            .to_string();
        if !rebuild
            && old
                .get(&relative)
                .is_some_and(|(size, time)| *size as u64 == meta.len() && *time == modified)
        {
            continue;
        }
        let body = if meta.len() <= MAX_TEXT && text_file(&path) {
            let bytes = fs::read(&path)?;
            if bytes.contains(&0) {
                None
            } else {
                String::from_utf8(bytes).ok()
            }
        } else {
            None
        };
        let indexed = body.is_some();
        let markdown = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"));
        let (aliases, sections) = sections(body.as_deref().unwrap_or(""), markdown);
        let title = path.file_stem().unwrap_or_default().to_string_lossy();
        tx.execute("DELETE FROM search WHERE path=?1", [&relative])?;
        for section in sections {
            tx.execute("INSERT INTO search(path,title,aliases,heading,body,line,end_line,content_indexed) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![relative, title, aliases, section.heading, section.body, section.line as i64, section.end_line as i64, indexed])?;
        }
        tx.execute(
            "INSERT OR REPLACE INTO files VALUES(?1,?2,?3,?4)",
            params![relative, i64::try_from(meta.len())?, modified, indexed],
        )?;
        update.updated += 1;
    }
    for path in old.keys().filter(|p| !seen.contains(*p)) {
        tx.execute("DELETE FROM search WHERE path=?1", [path])?;
        tx.execute("DELETE FROM files WHERE path=?1", [path])?;
        update.removed += 1;
    }
    tx.commit()?;
    update.files = seen.len();
    update.text_files = db.query_row(
        "SELECT count(*) FROM files WHERE content_indexed=1",
        [],
        |r| Ok(r.get::<_, i64>(0)? as usize),
    )?;
    update.metadata_only = update.files - update.text_files;
    Ok(update)
}

fn words(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn query_terms(query: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "do", "does", "did",
        "how", "why", "what", "which", "when", "where", "who", "can", "could", "would", "should",
        "will", "of", "in", "on", "at", "to", "for", "from", "with", "and", "or", "it", "its",
        "this", "that", "these", "those", "i", "me", "my", "you", "your", "please", "explain",
        "tell", "about", "between", "not",
    ];
    let all = words(query);
    let mut filtered: Vec<_> = all
        .iter()
        .filter(|w| !STOP.contains(&w.as_str()))
        .cloned()
        .collect();
    if filtered.is_empty() {
        filtered = all;
    }
    let mut seen = HashSet::new();
    filtered.retain(|w| seen.insert(w.clone()));
    filtered.truncate(24);
    filtered
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

fn search(
    db: &Connection,
    root: &Path,
    query: &str,
    limit: usize,
    all_sections: bool,
    fuzzy: bool,
    all_terms: bool,
) -> Result<(Vec<Hit>, Expansions)> {
    let terms = query_terms(query);
    if terms.is_empty() {
        return Err("Query needs at least one letter or number".into());
    }
    let vocabulary: Vec<String> = if fuzzy {
        db.prepare("SELECT term FROM vocabulary")?
            .query_map([], |r| r.get(0))?
            .collect::<std::result::Result<_, _>>()?
    } else {
        Vec::new()
    };
    let mut expansions = BTreeMap::new();
    let mut groups = Vec::new();
    for term in &terms {
        let n = term.chars().count();
        let mut candidates = Vec::new();
        // Correct only terms with no existing literal/stem/prefix match. Expanding
        // valid words such as "mean" to "heat" or "speed" to "seed" adds noise.
        let literal = format!("\"{term}\"{}", if n >= 4 { "*" } else { "" });
        let exists: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM search WHERE search MATCH ?1)",
            [literal],
            |r| r.get(0),
        )?;
        if fuzzy && n >= 4 && !exists {
            let max_distance = if n >= 8 { 2 } else { 1 };
            for word in &vocabulary {
                if word == term || word.chars().count().abs_diff(n) > max_distance {
                    continue;
                }
                let distance = strsim::damerau_levenshtein(term, word);
                if distance <= max_distance {
                    candidates.push((distance, word.clone()));
                }
            }
        }
        candidates.sort();
        candidates.truncate(6);
        let variants: Vec<_> = candidates.into_iter().map(|(_, w)| w).collect();
        let mut group = vec![term.clone()];
        group.extend(variants.iter().cloned());
        if !variants.is_empty() {
            expansions.insert(term.clone(), variants);
        }
        groups.push(group);
    }
    let expression = groups
        .iter()
        .map(|group| {
            format!(
                "({})",
                group
                    .iter()
                    .map(|word| format!(
                        "\"{}\"{}",
                        word,
                        if word.chars().count() >= 4 { "*" } else { "" }
                    ))
                    .collect::<Vec<_>>()
                    .join(" OR ")
            )
        })
        .collect::<Vec<_>>()
        .join(if all_terms { " AND " } else { " OR " });
    let mut stmt = db.prepare("SELECT path,title,heading,body,line,end_line,content_indexed,bm25(search,2.0,10.0,8.0,6.0,1.0) FROM search WHERE search MATCH ?1 ORDER BY bm25(search,2.0,10.0,8.0,6.0,1.0) LIMIT 1000")?;
    let rows = stmt.query_map([expression], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, i64>(4)? as usize,
            r.get::<_, i64>(5)? as usize,
            r.get::<_, bool>(6)?,
            r.get::<_, f64>(7)?,
        ))
    })?;
    let phrase = words(query);
    let mut hits = Vec::new();
    for row in rows {
        let (path, title, heading, body, line, end_line, content_indexed, rank) = row?;
        let context = words(&format!("{path} {heading} {body}"));
        let matched = groups
            .iter()
            .filter(|g| {
                g.iter().any(|t| {
                    context
                        .iter()
                        .any(|w| w == t || (t.len() >= 4 && w.starts_with(t)))
                })
            })
            .count();
        let coverage = matched as f64 / groups.len() as f64;
        let title_exact = title.to_lowercase() == query.trim().to_lowercase();
        // FTS vocabulary contains stems: a typo can expand to unrelated stems.
        // Prefer a close whole-title spelling without suppressing alternatives.
        let typo_title = terms.len() == 1
            && expansions.contains_key(&terms[0])
            && strsim::damerau_levenshtein(&terms[0], &title.to_lowercase())
                <= if terms[0].chars().count() >= 8 { 2 } else { 1 };
        // Literal phrases distinguish, for example, critical damping from
        // separate mentions of damping and critical speed. Never span sections.
        let body_words = words(&body);
        let phrase_match =
            phrase.len() > 1 && body_words.windows(phrase.len()).any(|w| w == phrase);
        let body_lines: Vec<_> = body.lines().collect();
        let metadata_end = if line == 1 && path.ends_with(".md") {
            frontmatter(&body_lines).0
        } else {
            0
        };
        let substantive = body_lines.iter().skip(metadata_end).any(|line| {
            let line = line.trim();
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with("- [[")
                && !line.starts_with("- [ ] [[")
                && !line.starts_with("- [x] [[")
                && words(line).len() >= 5
        });
        let quality = if substantive { 1.0 } else { 0.35 };
        let score = -rank
            * (1.0 + 3.0 * coverage * coverage)
            * if phrase_match { 4.0 } else { 1.0 }
            * quality
            + if title_exact {
                100.0
            } else if typo_title {
                60.0
            } else {
                0.0
            };
        let snippet = body_lines
            .iter()
            .skip(metadata_end)
            .copied()
            .filter(|s| !s.trim().is_empty())
            .max_by_key(|s| {
                let lower = s.to_lowercase();
                groups
                    .iter()
                    .filter(|g| g.iter().any(|w| lower.contains(w)))
                    .count()
            })
            .unwrap_or("")
            .trim()
            .chars()
            .take(300)
            .collect();
        let target = path.strip_suffix(".md").unwrap_or(&path);
        let wikilink = if heading.is_empty() {
            format!("[[{target}]]")
        } else {
            format!("[[{target}#{heading}]]")
        };
        let absolute = root.join(&path);
        hits.push(Hit {
            path,
            absolute_path: absolute.to_string_lossy().into_owned(),
            title,
            heading,
            line,
            end_line,
            snippet,
            wikilink,
            uri: uri(&absolute),
            content_indexed,
            score,
        });
    }
    hits.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then(a.path.cmp(&b.path))
            .then(a.line.cmp(&b.line))
    });
    if !all_sections {
        let mut seen = HashSet::new();
        hits.retain(|h| seen.insert(h.path.clone()));
    }
    hits.truncate(limit);
    Ok((hits, expansions))
}

fn resolve(root: &Path, path: &Path) -> Result<PathBuf> {
    let full = root.join(path).canonicalize()?;
    if !full.starts_with(root) || !full.is_file() {
        return Err("Path must resolve to a file inside this vault".into());
    }
    Ok(full)
}

fn run() -> Result<bool> {
    let cli = Cli::parse();
    let root = vault_root(cli.root)?;
    match cli.command {
        Commands::Index { rebuild } => {
            let mut db = database(&root)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&refresh(&mut db, &root, rebuild)?)?
            );
        }
        Commands::Search {
            query,
            limit,
            json,
            compact,
            all_terms,
            sections,
            no_fuzzy,
        } => {
            let mut db = database(&root)?;
            let update = refresh(&mut db, &root, false)?;
            let (results, expansions) = search(
                &db,
                &root,
                &query,
                limit as usize,
                sections,
                !no_fuzzy,
                all_terms,
            )?;
            let empty = results.is_empty();
            if compact {
                let hits: Vec<_> = results
                    .iter()
                    .map(|hit| {
                        serde_json::json!({
                            "path": hit.path, "heading": hit.heading,
                            "line": hit.line, "end_line": hit.end_line,
                            "snippet": hit.snippet, "content_indexed": hit.content_indexed
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::json!({
                        "expansions": expansions, "results": hits
                    })
                );
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&SearchOutput {
                        query,
                        root: root.to_string_lossy().into_owned(),
                        refresh: update,
                        expansions,
                        results
                    })?
                );
            } else {
                eprintln!(
                    "Index: {} files, {} updated, {} removed",
                    update.files, update.updated, update.removed
                );
                for (i, hit) in results.iter().enumerate() {
                    println!(
                        "{}. {}:{}–{}\n   {}\n   {}{}\n",
                        i + 1,
                        hit.path,
                        hit.line,
                        hit.end_line,
                        hit.wikilink,
                        hit.snippet,
                        if hit.content_indexed {
                            ""
                        } else {
                            " [filename/path only]"
                        }
                    );
                }
                if empty {
                    println!("No matching files found. Try a related term.");
                }
            }
            return Ok(!empty);
        }
        Commands::Read { path, line, lines } => {
            let full = resolve(&root, &path)?;
            if full.metadata()?.len() > MAX_TEXT {
                return Err(
                    "File exceeds the 4 MiB text-read limit; use a suitable file reader".into(),
                );
            }
            let text = fs::read_to_string(&full)?;
            if text.contains('\0') {
                return Err("Binary file; use a suitable file reader".into());
            }
            let total = text.lines().count();
            if line as usize > total.max(1) {
                return Err(format!("Line {line} is beyond the file's {total} lines").into());
            }
            println!("{} ({} lines)", full.display(), total);
            for (i, text) in text
                .lines()
                .enumerate()
                .skip(line as usize - 1)
                .take(lines as usize)
            {
                println!("{:>5} | {}", i + 1, text);
            }
        }
        Commands::Open {
            path,
            uri: print_uri,
        } => {
            let target = uri(&resolve(&root, &path)?);
            if print_uri {
                println!("{target}");
            } else {
                let launcher = if cfg!(target_os = "macos") {
                    "open"
                } else {
                    "xdg-open"
                };
                if !Command::new(launcher).arg(&target).status()?.success() {
                    return Err("Could not open Obsidian; try open --uri".into());
                }
            }
        }
    }
    Ok(true)
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("vault-search: {e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".obsidian")).unwrap();
        dir
    }

    #[test]
    fn new_changed_renamed_and_deleted_notes_refresh_automatically() {
        let dir = fixture();
        let root = dir.path();
        let mut db = database(root).unwrap();
        refresh(&mut db, root, false).unwrap();
        fs::write(
            root.join("Flywheel.md"),
            "# Flywheel\nEnergy storage during cyclic speed fluctuations.",
        )
        .unwrap();
        assert_eq!(refresh(&mut db, root, false).unwrap().updated, 1);
        assert_eq!(
            search(&db, root, "flyweel", 5, false, true, false)
                .unwrap()
                .0[0]
                .path,
            "Flywheel.md"
        );
        fs::write(
            root.join("Flywheel.md"),
            "# Flywheel\nCoefficient of fluctuation of speed and energy.",
        )
        .unwrap();
        assert_eq!(refresh(&mut db, root, false).unwrap().updated, 1);
        assert!(!search(&db, root, "coefficient", 5, false, true, false)
            .unwrap()
            .0
            .is_empty());
        fs::rename(root.join("Flywheel.md"), root.join("Energy.md")).unwrap();
        let update = refresh(&mut db, root, false).unwrap();
        assert_eq!((update.updated, update.removed), (1, 1));
        fs::remove_file(root.join("Energy.md")).unwrap();
        assert_eq!(refresh(&mut db, root, false).unwrap().removed, 1);
        assert!(search(&db, root, "flywheel", 5, false, true, false)
            .unwrap()
            .0
            .is_empty());
    }

    #[test]
    fn aliases_headings_unicode_attachments_and_exclusions() {
        let dir = fixture();
        let root = dir.path();
        fs::write(root.join("Rotating inertia.md"), "---\naliases: [Flywheel, Volant]\n---\n# Rotating inertia\n\n## Énergie\nStores kinetic energy.\n```rust\n# fake heading\n```\n").unwrap();
        fs::write(root.join("Rotor diagram.png"), [0, 1, 2]).unwrap();
        fs::create_dir(root.join("API KEYS")).unwrap();
        fs::write(root.join("API KEYS/secret.md"), "privatecredentials").unwrap();
        let mut db = database(root).unwrap();
        let update = refresh(&mut db, root, false).unwrap();
        assert_eq!(
            (update.files, update.text_files, update.metadata_only),
            (2, 1, 1)
        );
        assert_eq!(
            search(&db, root, "volant", 5, false, true, false)
                .unwrap()
                .0[0]
                .path,
            "Rotating inertia.md"
        );
        let hits = search(&db, root, "énergie", 5, true, true, false)
            .unwrap()
            .0;
        assert_eq!((hits[0].heading.as_str(), hits[0].line), ("Énergie", 6));
        assert!(
            !search(&db, root, "rotor diagram", 5, false, true, false)
                .unwrap()
                .0[0]
                .content_indexed
        );
        assert!(
            search(&db, root, "privatecredentials", 5, false, true, false)
                .unwrap()
                .0
                .is_empty()
        );
        assert_eq!(refresh(&mut db, root, false).unwrap().updated, 0);
    }

    #[test]
    fn paths_and_query_input_are_safe() {
        let dir = fixture();
        fs::write(dir.path().join("A note.md"), "Text").unwrap();
        assert!(resolve(dir.path(), Path::new("A note.md")).is_ok());
        assert!(resolve(dir.path(), Path::new("/etc/passwd")).is_err());
        let mut db = database(dir.path()).unwrap();
        refresh(&mut db, dir.path(), false).unwrap();
        assert!(search(&db, dir.path(), "\" OR * ( )", 5, false, true, false).is_ok());
        assert!(search(&db, dir.path(), "***", 5, false, true, false).is_err());
        assert_eq!(encode("A & B.md"), "A%20%26%20B.md");
    }
}
