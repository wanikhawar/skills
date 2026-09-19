use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn search(root: &Path, query: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_vault-search"))
        .args(["--root", root.to_str().unwrap(), "search", query, "--json"])
        .output()
        .unwrap();
    assert!(
        matches!(output.status.code(), Some(0 | 1)),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn search_alone_discovers_new_notes_and_tracks_edits_moves_and_deletes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join(".obsidian")).unwrap();
    assert_eq!(
        search(root, "flywheel")["results"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    fs::write(
        root.join("Flywheel.md"),
        "# Flywheel\nStores cyclic energy.",
    )
    .unwrap();
    let added = search(root, "flyweel");
    assert_eq!(added["refresh"]["updated"], 1);
    assert_eq!(added["results"][0]["path"], "Flywheel.md");

    fs::write(
        root.join("Flywheel.md"),
        "# Flywheel\nControls speed fluctuations within each cycle.",
    )
    .unwrap();
    let edited = search(root, "fluctuations");
    assert_eq!(edited["refresh"]["updated"], 1);
    assert!(edited["results"][0]["snippet"]
        .as_str()
        .unwrap()
        .contains("Controls speed"));

    fs::rename(root.join("Flywheel.md"), root.join("Rotating energy.md")).unwrap();
    let moved = search(root, "flywheel");
    assert_eq!(moved["refresh"]["removed"], 1);
    assert_eq!(moved["results"][0]["path"], "Rotating energy.md");

    fs::remove_file(root.join("Rotating energy.md")).unwrap();
    let deleted = search(root, "flywheel");
    assert_eq!(deleted["refresh"]["removed"], 1);
    assert!(deleted["results"].as_array().unwrap().is_empty());
}

#[test]
fn exact_words_do_not_expand_and_read_returns_current_source() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join(".obsidian")).unwrap();
    fs::write(
        root.join("Example.md"),
        "# Example\nSpeed changes.\n## C#\nCode.",
    )
    .unwrap();
    fs::write(root.join("Noise.md"), "Seed changes.").unwrap();
    let found = search(root, "speed");
    assert!(found["expansions"].as_object().unwrap().is_empty());
    assert_eq!(found["results"].as_array().unwrap().len(), 1);
    let read = Command::new(env!("CARGO_BIN_EXE_vault-search"))
        .args([
            "--root",
            root.to_str().unwrap(),
            "read",
            "Example.md",
            "--line",
            "3",
            "--lines",
            "2",
        ])
        .output()
        .unwrap();
    assert!(read.status.success());
    assert!(String::from_utf8_lossy(&read.stdout).contains("3 | ## C#"));
    assert_eq!(search(root, "code")["results"][0]["heading"], "C#");
}

#[test]
fn compact_all_terms_and_default_limit_work_together() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join(".obsidian")).unwrap();
    for n in 0..5 {
        fs::write(
            root.join(format!("Thermal {n}.md")),
            "# Thermal\nHeating transfers energy between bodies.\n",
        )
        .unwrap();
    }
    fs::write(
        root.join("Separate.md"),
        "# Heat\nHeat transfer.\n## Unrelated\nZebras graze.\n",
    )
    .unwrap();
    let run = |query: &str, flags: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_vault-search"))
            .args([
                "--root",
                root.to_str().unwrap(),
                "search",
                query,
                "--compact",
            ])
            .args(flags)
            .output()
            .unwrap();
        assert!(matches!(output.status.code(), Some(0 | 1)));
        serde_json::from_slice::<Value>(&output.stdout).unwrap()
    };
    let compact = run("heat", &[]);
    assert_eq!(compact["results"].as_array().unwrap().len(), 3);
    let hit = &compact["results"][0];
    assert!(hit.get("absolute_path").is_none());
    assert!(hit.get("uri").is_none());
    for key in [
        "path",
        "heading",
        "line",
        "end_line",
        "snippet",
        "content_indexed",
    ] {
        assert!(hit.get(key).is_some());
    }
    assert!(!run("heat zzqxvnotaword", &[])["results"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(run("heat zzqxvnotaword", &["--all-terms"])["results"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(run("transfer zebras", &["--all-terms"])["results"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(!run("heated energy", &["--all-terms"])["results"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn phrases_explanations_and_typo_targets_rank_above_noise() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join(".obsidian")).unwrap();
    fs::write(root.join("Vibrations.md"), "# Vibrations\n## Critical speed\nCritical speed depends on damping. Damping changes the critical response.\n## Damped free vibration\nCritical damping is the boundary between oscillatory and non-oscillatory response.\n").unwrap();
    fs::write(
        root.join("Entropy.md"),
        "---\naliases: [Entropy, Thermodynamic entropy and other state properties]\n---\n# Entropy\n## Definition\nEntropy is a thermodynamic state property of a system.\n",
    )
    .unwrap();
    fs::write(
        root.join("Pollution.md"),
        "# Pollution\nEutrophication affects water quality and aquatic life.\n",
    )
    .unwrap();
    fs::write(root.join("Links.md"), "# Links\n- [[Entropy]]\n").unwrap();
    assert_eq!(
        search(root, "critical damping")["results"][0]["heading"],
        "Damped free vibration"
    );
    let entropy = search(root, "entrophy");
    assert_eq!(entropy["results"][0]["path"], "Entropy.md");
    assert_eq!(entropy["results"][0]["heading"], "Definition");
    assert!(entropy["expansions"]["entrophy"].is_array());
}
