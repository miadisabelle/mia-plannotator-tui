//! `--annotate --occurrence N` marks the named occurrence of a repeated phrase, through the
//! real binary and a private data dir.

#![allow(clippy::expect_used, reason = "tests assert by panicking")]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_plannotator-tui"))
}

fn annotate(data: &Path, doc: &Path, extra: &[&str]) -> Output {
    let mut args =
        vec!["--annotate".to_owned(), doc.display().to_string(), "the review".to_owned(), "note".to_owned()];
    args.extend(extra.iter().map(|s| (*s).to_owned()));
    bin().env("PLANNOTATOR_DATA_DIR", data).args(&args).output().expect("annotate runs")
}

fn export(data: &Path, doc: &Path) -> String {
    let out = bin()
        .env("PLANNOTATOR_DATA_DIR", data)
        .args(["--export", &doc.display().to_string()])
        .output()
        .expect("export runs");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn scratch(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("plannotator-tui-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("dirs");
    let doc = root.join("doc.md");
    std::fs::write(&doc, "# Draft\n\nMark the review.\n\nThen close the review circle.\n").expect("doc");
    (root.clone(), doc, root.join("data"))
}

#[test]
fn occurrence_marks_the_named_match_of_a_repeated_phrase() {
    let (root, doc, data) = scratch("occurrence");
    let out = annotate(&data, &doc, &["comment", "--occurrence", "2"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let text = export(&data, &doc);
    assert!(text.contains("## Annotation 1 (line 5)"), "{text}");
    std::fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn without_occurrence_the_first_match_is_marked() {
    let (root, doc, data) = scratch("first");
    let out = annotate(&data, &doc, &[]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(export(&data, &doc).contains("## Annotation 1 (line 3)"));
    std::fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn a_missing_occurrence_fails_and_says_how_many_there_are() {
    let (root, doc, data) = scratch("missing");
    let out = annotate(&data, &doc, &["--occurrence", "3"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("occurrence 3") && err.contains("2 time"), "{err}");
    let zero = annotate(&data, &doc, &["--occurrence", "0"]);
    assert!(!zero.status.success());
    std::fs::remove_dir_all(&root).expect("cleanup");
}
