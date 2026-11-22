//! Probe core (CASTLE-SPEC-26.11.20): canonical finding types, the evidence
//! document, the deterministic repo walker, and BLAKE3 digest helpers.
//!
//! ## Law
//!
//! 1. **Findings are pure data with content-derived identity.** A
//!    [`ProbeFinding`] carries no authority and no timestamp; its `id` is the
//!    BLAKE3 hash of its canonical identity fields (rule, severity, category,
//!    file, line, snippet). Same content ⇒ same id, byte-identically, on
//!    every machine and every run. Wall-clock never enters identity.
//! 2. **The walker is deterministic.** [`walk_repo`] visits entries in
//!    sorted order, never follows symlinks, and skips exactly the names in
//!    [`SKIP_DIRS`]. The same tree always yields the same file list in the
//!    same order.
//! 3. **Evidence is canonical and self-digesting.** [`EvidenceDocument`]
//!    serializes through JCS canonical JSON (`serde_json_canonicalizer`, the
//!    repo's admitted canonicalizer) and carries a detached BLAKE3 digest
//!    over its own canonical form *excluding* the digest field, so a sealed
//!    document can be re-verified byte-for-byte after transport. Identical
//!    input trees produce identical digests (replay law).
//!
//! The rule families (`ci`, `auth`, `custody`, `observability`) emit
//! [`ProbeFinding`] values into an [`EvidenceDocument`]; nothing in this
//! module reads their internals, only the shared shapes defined here.

pub mod auth;
pub mod ci;
pub mod custody;
pub mod observability;

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Schema identity stamped on every evidence document.
pub const EVIDENCE_SCHEMA: &str = "castle.evidence.v1";

/// Probe implementation version (spec generation).
pub const PROBE_VERSION: &str = "26.11.20";

/// Directory names the walker never descends into (and never lists).
pub const SKIP_DIRS: &[&str] = &[".git", "node_modules", "target"];

/// Rule-family category constants shared by the probe submodules.
pub const CATEGORY_CI: &str = "ci";
pub const CATEGORY_AUTH: &str = "auth";
pub const CATEGORY_CUSTODY: &str = "custody";
pub const CATEGORY_OBSERVABILITY: &str = "observability";

/// Typed refusal: the walker could not read a directory it was required to
/// descend into. A silently skipped directory would corrupt the evidence.
pub const REFUSED_WALK_READ: &str = "REFUSED:PROBE_WALK_READ_FAILED";

/// Typed refusal: the probe root does not exist or is not a directory.
pub const REFUSED_WALK_ROOT: &str = "REFUSED:PROBE_ROOT_NOT_A_DIRECTORY";

/// True when a directory entry with this name must be skipped by the walker.
#[must_use]
pub fn is_skipped(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

/// BLAKE3-256 of `bytes`, lowercase hex (64 chars).
#[must_use]
pub fn digest_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// BLAKE3-256 of a file's raw bytes, lowercase hex.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] when the file cannot be read.
pub fn digest_file(path: &Path) -> io::Result<String> {
    let bytes = fs::read(path)?;
    Ok(digest_hex(&bytes))
}

/// BLAKE3-256 over a sorted `(path, digest)` file list: each pair is bound as
/// `path \0 digest \n` and the concatenation is hashed. Order-sensitivity is
/// intentional: the same members in a different order are a different tree.
/// Callers pass the walker's sorted output.
#[must_use]
pub fn tree_digest(files: &[(String, String)]) -> String {
    let mut hasher = blake3::Hasher::new();
    for (path, digest) in files {
        hasher.update(path.as_bytes());
        hasher.update(b"\0");
        hasher.update(digest.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

/// Finding severity, ordered most-severe-first by [`ProbeSeverity::rank`].
///
/// Serialized as the exact spec strings `CRITICAL | HIGH | MED | LOW`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProbeSeverity {
    Critical,
    High,
    Med,
    Low,
}

impl ProbeSeverity {
    /// Severity rank: `CRITICAL=3, HIGH=2, MED=1, LOW=0`. Derived enum order
    /// (`Critical < High < Med < Low`) is kept aligned with the rank so both
    /// orderings agree.
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::Critical => 3,
            Self::High => 2,
            Self::Med => 1,
            Self::Low => 0,
        }
    }
}

impl std::fmt::Display for ProbeSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Critical => "CRITICAL",
            Self::High => "HIGH",
            Self::Med => "MED",
            Self::Low => "LOW",
        };
        f.write_str(text)
    }
}

/// One probe observation: a rule fired against a location in the repo.
///
/// `file` is the repo-relative path with `/` separators (platform-stable),
/// `line` is 1-based with `0` meaning "not line-addressable". The `id` is
/// content-derived: see [`finding_id`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProbeFinding {
    /// BLAKE3 content identity over the fields below (see [`finding_id`]).
    pub id: String,
    pub severity: ProbeSeverity,
    /// Rule family (`ci`, `auth`, `custody`, `observability`).
    pub category: String,
    /// Repo-relative path, `/` separators, never absolute.
    pub file: String,
    /// 1-based source line; `0` = not line-addressable.
    pub line: u32,
    /// Verbatim evidence excerpt from the location.
    pub snippet: String,
    /// Stable rule identity, e.g. `AUTH-001`.
    pub rule_id: String,
}

/// Canonical identity input for a finding's `id`: every field that makes two
/// findings the same finding. The `id` itself is excluded by construction.
#[derive(Serialize)]
struct FindingIdentity<'a> {
    rule_id: &'a str,
    severity: ProbeSeverity,
    category: &'a str,
    file: &'a str,
    line: u32,
    snippet: &'a str,
}

/// Canonical JSON of any `Serialize` value; falls back to plain compact JSON
/// if the canonicalizer refuses (cannot happen for these plain-data shapes,
/// and the fallback is itself deterministic in the input).
fn canonical_bytes<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json_canonicalizer::to_vec(value)
        .unwrap_or_else(|_| serde_json::to_vec(value).unwrap_or_default())
}

/// Content-derived finding identity: BLAKE3 over the canonical JSON of the
/// identity fields. Same content ⇒ same id, always.
#[must_use]
pub fn finding_id(
    severity: ProbeSeverity,
    category: &str,
    file: &str,
    line: u32,
    snippet: &str,
    rule_id: &str,
) -> String {
    let identity = FindingIdentity {
        rule_id,
        severity,
        category,
        file,
        line,
        snippet,
    };
    digest_hex(&canonical_bytes(&identity))
}

impl ProbeFinding {
    /// Builds a finding with its content-derived [`ProbeFinding::id`].
    #[must_use]
    pub fn new(
        severity: ProbeSeverity,
        category: &str,
        file: &str,
        line: u32,
        snippet: &str,
        rule_id: &str,
    ) -> Self {
        Self {
            id: finding_id(severity, category, file, line, snippet, rule_id),
            severity,
            category: category.to_string(),
            file: file.to_string(),
            line,
            snippet: snippet.to_string(),
            rule_id: rule_id.to_string(),
        }
    }
}

/// One digested file in an evidence document.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileEntry {
    /// Repo-relative path, `/` separators.
    pub path: String,
    /// BLAKE3-256 hex of the file bytes.
    pub digest: String,
    /// File size in bytes.
    pub bytes: u64,
}

/// Canonical `evidence.json`: the durable, replayable product of a probe run.
///
/// ## Determinism
///
/// Files arrive sorted by path (walker law), findings are stored sorted by
/// `(severity rank desc, id)`, and `generated_at` is supplied by the caller —
/// never captured from the wall clock — so identical inputs produce
/// byte-identical canonical documents and identical [`EvidenceDocument::seal`]
/// digests.
///
/// ## Self-digest
///
/// [`EvidenceDocument::seal`] computes BLAKE3 over the canonical JSON of the
/// document with `digest` absent and stores it. [`EvidenceDocument::verify`]
/// recomputes and compares, so any post-transport mutation is detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceDocument {
    /// Always [`EVIDENCE_SCHEMA`].
    pub schema: String,
    /// Repo-relative root the probe ran against (display path).
    pub root: String,
    /// Probe implementation version ([`PROBE_VERSION`]).
    pub probe_version: String,
    /// Optional caller-supplied RFC 3339 stamp (never wall-clock captured).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    pub file_count: usize,
    pub total_bytes: u64,
    /// Sorted by `path`.
    pub files: Vec<FileEntry>,
    /// Sorted by `(severity rank desc, id)`.
    pub findings: Vec<ProbeFinding>,
    /// Detached self-digest; `None` until [`EvidenceDocument::seal`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
}

impl EvidenceDocument {
    /// Walks `root`, digests every admitted file, and returns the document
    /// with an empty findings set (rule families append via
    /// [`EvidenceDocument::add_findings`]). `generated_at` is passed through
    /// verbatim for replay determinism.
    ///
    /// # Errors
    ///
    /// Refuses ([`REFUSED_WALK_ROOT`], [`REFUSED_WALK_READ`]) when the root
    /// is not a readable directory or a directory cannot be listed.
    pub fn collect(root: &Path, generated_at: Option<String>) -> io::Result<Self> {
        let paths = walk_repo(root)?;
        let mut files = Vec::with_capacity(paths.len());
        let mut total_bytes = 0u64;
        for rel in &paths {
            let abs = root.join(rel);
            let meta = fs::metadata(&abs)?;
            let bytes = meta.len();
            let digest = digest_file(&abs)?;
            files.push(FileEntry {
                path: rel.clone(),
                digest,
                bytes,
            });
            total_bytes += bytes;
        }
        let root_text = root.to_string_lossy().to_string();
        Ok(Self {
            schema: EVIDENCE_SCHEMA.to_string(),
            root: root_text,
            probe_version: PROBE_VERSION.to_string(),
            generated_at,
            file_count: files.len(),
            total_bytes,
            files,
            findings: Vec::new(),
            digest: None,
        })
    }

    /// Appends findings and re-sorts the set into the canonical order
    /// `(severity rank desc, id)`.
    pub fn add_findings<I: IntoIterator<Item = ProbeFinding>>(&mut self, findings: I) {
        self.findings.extend(findings);
        self.findings
            .sort_by(|a, b| b.severity.rank().cmp(&a.severity.rank()).then(a.id.cmp(&b.id)));
    }

    /// Canonical JSON (JCS) of the document exactly as serialized.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] when serialization fails.
    pub fn canonical_json(&self) -> serde_json::Result<String> {
        serde_json_canonicalizer::to_string(self)
    }

    /// BLAKE3 over the canonical JSON of the document with the `digest`
    /// field absent — the input domain of [`EvidenceDocument::seal`] and
    /// [`EvidenceDocument::verify`].
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] when serialization fails.
    pub fn digest_input(&self) -> serde_json::Result<String> {
        let mut unsealed = self.clone();
        unsealed.digest = None;
        serde_json_canonicalizer::to_string(&unsealed)
            .map(|json| digest_hex(json.as_bytes()))
    }

    /// Computes and stores the detached self-digest, returning it.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] when serialization fails.
    pub fn seal(&mut self) -> serde_json::Result<String> {
        let digest = self.digest_input()?;
        self.digest = Some(digest.clone());
        Ok(digest)
    }

    /// Re-verifies the sealed digest against the current content.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] when serialization fails.
    pub fn verify(&self) -> serde_json::Result<bool> {
        match &self.digest {
            Some(sealed) => Ok(&self.digest_input()? == sealed),
            None => Ok(false),
        }
    }
}

/// Deterministic directory walk over `root`.
///
/// Returns repo-relative paths with `/` separators, sorted lexicographically.
/// Skips exactly [`SKIP_DIRS`] by entry name (directories and files alike),
/// never follows symlinks, and lists only regular files (directories are
/// descended, other file types are excluded).
///
/// # Errors
///
/// Refuses with [`REFUSED_WALK_ROOT`] when `root` is not an existing
/// directory and with [`REFUSED_WALK_READ`] when a required directory cannot
/// be listed — a silently skipped directory would corrupt the evidence.
pub fn walk_repo(root: &Path) -> io::Result<Vec<String>> {
    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{REFUSED_WALK_ROOT}: {}", root.display()),
        ));
    }
    let mut out = Vec::new();
    walk_inner(root, &mut String::new(), &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_inner(dir: &Path, prefix: &mut String, out: &mut Vec<String>) -> io::Result<()> {
    let entries = fs::read_dir(dir)
        .map_err(|e| io::Error::new(e.kind(), format!("{REFUSED_WALK_READ}: {}", dir.display())))?;
    // Sort by OS file name so descent order is deterministic; the final
    // full-list sort makes the guarantee global regardless.
    let mut named: Vec<(String, std::fs::DirEntry)> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            io::Error::new(e.kind(), format!("{REFUSED_WALK_READ}: {}", dir.display()))
        })?;
        named.push((entry.file_name().to_string_lossy().to_string(), entry));
    }
    named.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, entry) in named {
        if is_skipped(&name) {
            continue;
        }
        let path = entry.path();
        // symlink_metadata does not follow the link: symlinks are excluded
        // entirely, which both prevents cycles and keeps digests stable.
        let meta = fs::symlink_metadata(&path)?;
        let file_type = meta.file_type();
        let joined = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if file_type.is_dir() {
            let pushed = std::mem::replace(prefix, joined);
            walk_inner(&path, prefix, out)?;
            *prefix = pushed;
        } else if file_type.is_file() {
            out.push(joined);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SCRATCH_SEQ: AtomicUsize = AtomicUsize::new(0);

    /// Unique per-run scratch directory under the system temp dir; the caller
    /// removes it at test end (law: a test-minted dir is deleted by its test).
    fn scratch(label: &str) -> PathBuf {
        let seq = SCRATCH_SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "castle-probe-{label}-{}-{seq}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    fn write(path: &Path, content: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, content).expect("write file");
    }

    // ---------- severity ----------

    #[test]
    fn severity_rank_orders_most_severe_first() {
        assert!(ProbeSeverity::Critical.rank() > ProbeSeverity::High.rank());
        assert!(ProbeSeverity::High.rank() > ProbeSeverity::Med.rank());
        assert!(ProbeSeverity::Med.rank() > ProbeSeverity::Low.rank());
        assert_eq!(ProbeSeverity::Critical.rank(), 3);
        assert_eq!(ProbeSeverity::Low.rank(), 0);
    }

    #[test]
    fn severity_serializes_to_exact_spec_strings() {
        let json = serde_json::to_string(&ProbeSeverity::Critical).expect("serialize");
        assert_eq!(json, "\"CRITICAL\"");
        let json = serde_json::to_string(&ProbeSeverity::Med).expect("serialize");
        assert_eq!(json, "\"MED\"");
        let round: ProbeSeverity = serde_json::from_str("\"LOW\"").expect("deserialize");
        assert_eq!(round, ProbeSeverity::Low);
    }

    #[test]
    fn severity_display_matches_serialized_form() {
        for (sev, text) in [
            (ProbeSeverity::Critical, "CRITICAL"),
            (ProbeSeverity::High, "HIGH"),
            (ProbeSeverity::Med, "MED"),
            (ProbeSeverity::Low, "LOW"),
        ] {
            assert_eq!(sev.to_string(), text);
        }
    }

    // ---------- finding identity ----------

    #[test]
    fn finding_id_is_deterministic_for_identical_content() {
        let a = ProbeFinding::new(
            ProbeSeverity::High,
            CATEGORY_AUTH,
            "src/lib.rs",
            42,
            "let token = env::var(\"T\")",
            "AUTH-001",
        );
        let b = ProbeFinding::new(
            ProbeSeverity::High,
            CATEGORY_AUTH,
            "src/lib.rs",
            42,
            "let token = env::var(\"T\")",
            "AUTH-001",
        );
        assert_eq!(a, b);
        assert_eq!(a.id, b.id);
        assert_eq!(a.id.len(), 64, "id is a full BLAKE3 hex digest");
    }

    #[test]
    fn finding_id_changes_when_any_identity_field_changes() {
        let base = ProbeFinding::new(ProbeSeverity::Low, CATEGORY_CI, "a.yml", 1, "run: x", "CI-001");
        let variants = [
            ProbeFinding::new(ProbeSeverity::Med, CATEGORY_CI, "a.yml", 1, "run: x", "CI-001"),
            ProbeFinding::new(ProbeSeverity::Low, CATEGORY_AUTH, "a.yml", 1, "run: x", "CI-001"),
            ProbeFinding::new(ProbeSeverity::Low, CATEGORY_CI, "b.yml", 1, "run: x", "CI-001"),
            ProbeFinding::new(ProbeSeverity::Low, CATEGORY_CI, "a.yml", 2, "run: x", "CI-001"),
            ProbeFinding::new(ProbeSeverity::Low, CATEGORY_CI, "a.yml", 1, "run: y", "CI-001"),
            ProbeFinding::new(ProbeSeverity::Low, CATEGORY_CI, "a.yml", 1, "run: x", "CI-002"),
        ];
        for v in &variants {
            assert_ne!(base.id, v.id, "id must change when identity changes");
        }
    }

    #[test]
    fn finding_serde_roundtrip_preserves_all_fields() {
        let f = ProbeFinding::new(
            ProbeSeverity::Critical,
            CATEGORY_CUSTODY,
            "src/probe/custody.rs",
            7,
            "fs::write(out, bytes)",
            "CUSTODY-003",
        );
        let json = serde_json::to_string(&f).expect("serialize");
        assert!(json.contains("\"severity\":\"CRITICAL\""));
        assert!(json.contains("\"rule_id\":\"CUSTODY-003\""));
        let back: ProbeFinding = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, f);
    }

    // ---------- digests ----------

    #[test]
    fn digest_hex_matches_known_blake3_empty_vector() {
        // Admitted ecosystem vector (BLAKE3 of the empty input).
        assert_eq!(
            digest_hex(b""),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }

    #[test]
    fn digest_hex_is_deterministic_and_input_sensitive() {
        let a = digest_hex(b"castle");
        assert_eq!(a, digest_hex(b"castle"));
        assert_ne!(a, digest_hex(b"castla"));
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn digest_file_equals_digest_of_file_bytes() {
        let dir = scratch("digest-file");
        let path = dir.join("payload.bin");
        write(&path, b"evidence bytes");
        let expected = digest_hex(b"evidence bytes");
        assert_eq!(digest_file(&path).expect("digest file"), expected);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn tree_digest_is_order_sensitive_and_content_sensitive() {
        let t1 = tree_digest(&[
            ("a.txt".to_string(), "aa".to_string()),
            ("b.txt".to_string(), "bb".to_string()),
        ]);
        let t2 = tree_digest(&[
            ("b.txt".to_string(), "bb".to_string()),
            ("a.txt".to_string(), "aa".to_string()),
        ]);
        assert_ne!(t1, t2, "member order is part of tree identity");
        let t3 = tree_digest(&[
            ("a.txt".to_string(), "XX".to_string()),
            ("b.txt".to_string(), "bb".to_string()),
        ]);
        assert_ne!(t1, t3, "member digest is part of tree identity");
        assert_eq!(t1, tree_digest(&[
            ("a.txt".to_string(), "aa".to_string()),
            ("b.txt".to_string(), "bb".to_string()),
        ]));
    }

    // ---------- walker ----------

    #[test]
    fn walk_repo_refuses_missing_root() {
        let dir = scratch("walk-missing");
        let missing = dir.join("does-not-exist");
        let err = walk_repo(&missing).expect_err("must refuse");
        assert!(err.to_string().starts_with(REFUSED_WALK_ROOT));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn walk_repo_refuses_file_root() {
        let dir = scratch("walk-file-root");
        let file_root = dir.join("plain.txt");
        write(&file_root, b"x");
        let err = walk_repo(&file_root).expect_err("must refuse");
        assert!(err.to_string().starts_with(REFUSED_WALK_ROOT));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn walk_repo_skips_git_node_modules_target_and_is_sorted() {
        let dir = scratch("walk-skip");
        // Admitted files, deliberately created out of lexicographic order.
        write(&dir.join("zz.txt"), b"z");
        write(&dir.join("dir1/sub/c.txt"), b"c");
        write(&dir.join("dir1/b.txt"), b"b");
        write(&dir.join("a.txt"), b"a");
        // Hidden files are NOT skipped (only the three named dirs are).
        write(&dir.join(".hidden.cfg"), b"h");
        // Skipped trees with content that must never appear.
        write(&dir.join(".git/objects/ab/cd"), b"g");
        write(&dir.join("node_modules/left-pad/index.js"), b"n");
        write(&dir.join("target/debug/castle"), b"t");

        let got = walk_repo(&dir).expect("walk");
        assert_eq!(
            got,
            vec![
                ".hidden.cfg",
                "a.txt",
                "dir1/b.txt",
                "dir1/sub/c.txt",
                "zz.txt",
            ]
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn walk_repo_excludes_symlinks_without_following() {
        let dir = scratch("walk-symlink");
        write(&dir.join("real.txt"), b"r");
        let outside = scratch("walk-symlink-outside");
        write(&outside.join("secret.txt"), b"s");
        std::os::unix::fs::symlink(&outside.join("secret.txt"), dir.join("link-file.txt"))
            .expect("symlink file");
        std::os::unix::fs::symlink(&outside, dir.join("link-dir")).expect("symlink dir");

        let got = walk_repo(&dir).expect("walk");
        assert_eq!(got, vec!["real.txt"], "symlinks excluded, no follow, no cycle");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&outside);
    }

    #[test]
    fn is_skipped_matches_exactly_the_named_dirs() {
        for name in SKIP_DIRS {
            assert!(is_skipped(name));
        }
        assert!(!is_skipped(".github"));
        assert!(!is_skipped("targets"));
        assert!(!is_skipped("src"));
    }

    // ---------- evidence document ----------

    #[test]
    fn collect_produces_sorted_file_entries_and_totals() {
        let dir = scratch("evidence-collect");
        write(&dir.join("b.bin"), [0u8, 1, 2, 3]);
        write(&dir.join("a.txt"), b"hello");
        write(&dir.join(".git/HEAD"), b"ref: refs/heads/main");

        let mut doc = EvidenceDocument::collect(&dir, None).expect("collect");
        assert_eq!(doc.schema, EVIDENCE_SCHEMA);
        assert_eq!(doc.probe_version, PROBE_VERSION);
        assert_eq!(doc.file_count, 2);
        assert_eq!(doc.total_bytes, 5 + 4);
        assert_eq!(
            doc.files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(),
            vec!["a.txt", "b.bin"]
        );
        assert_eq!(doc.files[0].digest, digest_hex(b"hello"));
        assert_eq!(doc.files[0].bytes, 5);
        assert!(doc.findings.is_empty());
        assert!(doc.digest.is_none());

        doc.add_findings([
            ProbeFinding::new(ProbeSeverity::Low, CATEGORY_CI, "a.txt", 1, "hello", "CI-001"),
            ProbeFinding::new(ProbeSeverity::High, CATEGORY_AUTH, "b.bin", 0, "", "AUTH-002"),
        ]);
        assert_eq!(doc.findings[0].severity, ProbeSeverity::High, "sorted by rank desc");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seal_is_replayable_and_verify_detects_tampering() {
        let dir = scratch("evidence-seal");
        write(&dir.join("f.txt"), b"stable");
        let mut doc1 = EvidenceDocument::collect(&dir, Some("2026-11-20T00:00:00Z".to_string()))
            .expect("collect 1");
        let mut doc2 = EvidenceDocument::collect(&dir, Some("2026-11-20T00:00:00Z".to_string()))
            .expect("collect 2");
        let d1 = doc1.seal().expect("seal 1");
        let d2 = doc2.seal().expect("seal 2");
        assert_eq!(d1, d2, "identical inputs must seal to identical digests");
        assert!(doc1.verify().expect("verify sealed"));

        // Tamper with content: digest must stop matching.
        write(&dir.join("f.txt"), b"TAMPERED");
        let mut doc3 = EvidenceDocument::collect(&dir, Some("2026-11-20T00:00:00Z".to_string()))
            .expect("collect 3");
        doc3.digest = Some(d1.clone());
        assert!(!doc3.verify().expect("verify tampered"), "mutation must be detected");

        // Unsealed document never verifies.
        let mut doc4 = EvidenceDocument::collect(&dir, None).expect("collect 4");
        doc4.digest = None;
        assert!(!doc4.verify().expect("verify unsealed"));

        // generated_at participates: different stamp, different digest input.
        let mut doc5 = EvidenceDocument::collect(&dir, Some("2027-01-01T00:00:00Z".to_string()))
            .expect("collect 5");
        let d5 = doc5.seal().expect("seal 5");
        assert_ne!(d5, d1, "caller-supplied timestamp is part of the sealed form");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn canonical_json_is_whitespace_free_and_stable_across_builds() {
        let dir = scratch("evidence-canonical");
        write(&dir.join("z.txt"), b"z");
        let mut doc = EvidenceDocument::collect(&dir, None).expect("collect");
        doc.add_findings([ProbeFinding::new(
            ProbeSeverity::Med,
            CATEGORY_OBSERVABILITY,
            "z.txt",
            1,
            "z",
            "OBS-001",
        )]);
        let j1 = doc.canonical_json().expect("canonical 1");
        let j2 = doc.canonical_json().expect("canonical 2");
        assert_eq!(j1, j2);
        assert!(!j1.contains(' '), "JCS emits no insignificant whitespace");
        assert!(j1.starts_with('{') && j1.ends_with('}'));
        assert!(j1.contains("\"schema\":\"castle.evidence.v1\""));
        assert!(j1.contains("\"severity\":\"MED\""));
        // Canonical form is loadable back into an identical document.
        let back: EvidenceDocument = serde_json::from_str(&j1).expect("from canonical");
        assert_eq!(back, doc);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn canonicalizer_orders_keys_lexicographically() {
        // Direct evidence that the canonical byte form is key-sorted: the
        // document schema relies on this for cross-runtime byte-identity.
        let value = serde_json::json!({"schema": "s", "probe_version": "v", "root": "r"});
        let canonical =
            serde_json_canonicalizer::to_string(&value).expect("canonicalize value");
        assert_eq!(canonical, r#"{"probe_version":"v","root":"r","schema":"s"}"#);
    }

    #[test]
    fn evidence_document_is_self_digesting_over_canonical_form() {
        // digest_input equals BLAKE3 of the canonical JSON with digest absent.
        let dir = scratch("evidence-digest-input");
        write(&dir.join("one.txt"), b"1");
        let doc = EvidenceDocument::collect(&dir, None).expect("collect");
        let canonical = doc.canonical_json().expect("canonical");
        assert_eq!(
            doc.digest_input().expect("digest input"),
            digest_hex(canonical.as_bytes()),
            "unsealed document: canonical form IS the digest input"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ---------- tree digest over collected evidence ----------

    #[test]
    fn tree_digest_binds_collected_file_entries() {
        let dir = scratch("evidence-tree");
        write(&dir.join("x.txt"), b"x");
        write(&dir.join("y/y.txt"), b"y");
        let doc = EvidenceDocument::collect(&dir, None).expect("collect");
        let pairs: Vec<(String, String)> = doc
            .files
            .iter()
            .map(|f| (f.path.clone(), f.digest.clone()))
            .collect();
        let expected = tree_digest(&[
            ("x.txt".to_string(), digest_hex(b"x")),
            ("y/y.txt".to_string(), digest_hex(b"y")),
        ]);
        assert_eq!(tree_digest(&pairs), expected);
        let _ = fs::remove_dir_all(&dir);
    }
}
