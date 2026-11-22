//! Optional Typst → PDF compile for carousel decks.
//!
//! Honest standing: **Typst source emit is ALIVE** via `linkedin::publish_carousel`.
//! PDF compile is best-effort against a host `typst` binary. When the binary is
//! absent (common on clean CI / laptop images), this module returns
//! [`PdfCompileStatus::Gap`] and never pretends a PDF was produced.
#![forbid(unsafe_code)]

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Standing of an optional PDF compile attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "standing", rename_all = "snake_case")]
pub enum PdfCompileStatus {
    /// `typst compile` succeeded; PDF bytes written to `pdf_path`.
    Compiled {
        pdf_path: String,
        typst_version: Option<String>,
    },
    /// Documented gap: Typst CLI unavailable or compile failed. Source emit still OK.
    Gap {
        reason: String,
        typst_source_emitted: bool,
    },
}

impl PdfCompileStatus {
    pub fn is_compiled(&self) -> bool {
        matches!(self, Self::Compiled { .. })
    }
}

/// Probe whether a `typst` binary is on `PATH`.
pub fn typst_available() -> bool {
    Command::new("typst")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Attempt to compile `typst_source` to `out_pdf`.
///
/// Writes a temporary `.typ` beside the PDF when `scratch_typ` is `None`.
/// Never panics; failures become [`PdfCompileStatus::Gap`].
pub fn try_compile_pdf(
    typst_source: &str,
    out_pdf: impl AsRef<Path>,
    scratch_typ: Option<&Path>,
) -> PdfCompileStatus {
    let out_pdf = out_pdf.as_ref();
    if !typst_available() {
        return PdfCompileStatus::Gap {
            reason: "typst binary not found on PATH -- Typst source emit OK; PDF compile GAP"
                .to_string(),
            typst_source_emitted: true,
        };
    }

    let typ_path: PathBuf = match scratch_typ {
        Some(p) => p.to_path_buf(),
        None => out_pdf.with_extension("typ"),
    };

    if let Err(e) = std::fs::write(&typ_path, typst_source) {
        return PdfCompileStatus::Gap {
            reason: format!("failed to write Typst scratch {}: {e}", typ_path.display()),
            typst_source_emitted: true,
        };
    }

    let version = Command::new("typst")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    let output = Command::new("typst")
        .arg("compile")
        .arg(&typ_path)
        .arg(out_pdf)
        .output();

    match output {
        Ok(o) if o.status.success() && out_pdf.is_file() => PdfCompileStatus::Compiled {
            pdf_path: out_pdf.display().to_string(),
            typst_version: version,
        },
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            PdfCompileStatus::Gap {
                reason: format!(
                    "typst compile failed (exit {:?}): {}",
                    o.status.code(),
                    stderr.trim()
                ),
                typst_source_emitted: true,
            }
        }
        Err(e) => PdfCompileStatus::Gap {
            reason: format!("typst spawn failed: {e}"),
            typst_source_emitted: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documents_gap_when_typst_missing_or_returns_status() {
        // Regardless of host typst presence, try_compile against a nonsense path
        // must never panic and must report compiled|gap honestly.
        let dir = std::env::temp_dir().join(format!(
            "castle-typst-pdf-test-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let pdf = dir.join("out.pdf");
        let status = try_compile_pdf("#set page(width: 10pt, height: 10pt)\nhello", &pdf, None);
        match status {
            PdfCompileStatus::Compiled { .. } => assert!(pdf.is_file()),
            PdfCompileStatus::Gap {
                typst_source_emitted,
                ..
            } => assert!(typst_source_emitted),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
