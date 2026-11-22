//! Fail-closed `--public-anonymize` redaction court (CASTLE-SPEC-26.11.20).
//!
//! Production surface is the **refuse court** ([`RedactionCourt`]): public
//! publish must *prove* zero presence of private client names, home paths,
//! unredacted `file:line` citations, bearer residuals, calendar links, emails,
//! and (when policy says so) external links. Law: refuse, do not strip-and-ship.
//! Refusal findings never echo the raw private string (so `Debug`/`inspect`
//! cannot re-leak). The publish wire seals via [`RedactionCourt::admit`].
//!
//! [`require_public_anonymize`] is the CLI flag gate. There is no production
//! strip-and-ship surface — residue refuses at [`RedactionCourt::admit`].
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ── Flag / residual refusal (CLI gate) ─────────────────────────────────────

/// Refusal when a public target is invoked without `--public-anonymize`.
pub const REFUSED_NO_PUBLIC_ANONYMIZE: &str = "REFUSED_NO_PUBLIC_ANONYMIZE";
/// Chatman term: publishing without the anonymize gate is unauthorized actuation.
pub const BROKEN_TERM_NO_AUTHORITY: &str = "REFUSED_NO_AUTHORITY";

/// Citation placeholder recognized by the refuse court (already-redacted).
const REDACTED_CITE: &str = "[REDACTED_CITE]";

// ── Refuse court (lane-9 / Chicago tests) ───────────────────────────────────

/// Typed refusal code for every redaction-court failure.
pub const REFUSED_REDACTION: &str = "REFUSED_REDACTION";
/// Chatman failure-taxonomy term: publishing unredacted private material is
/// manufacture from unadmitted input.
pub const BROKEN_TERM: &str = "mu_on_O";

pub const RULE_PRIVATE_CLIENT_NAME: &str = "PRIVATE_CLIENT_NAME";
pub const RULE_PRIVATE_HOME_PATH: &str = "PRIVATE_HOME_PATH";
pub const RULE_UNREDACTED_CITATION: &str = "UNREDACTED_CITATION";
pub const RULE_BEARER_TOKEN: &str = "BEARER_TOKEN";
pub const RULE_CALENDAR_LINK: &str = "CALENDAR_LINK";
pub const RULE_EXTERNAL_LINK: &str = "EXTERNAL_LINK";
pub const RULE_EMAIL: &str = "EMAIL";

/// Replacement token used inside refuse-court details (never the raw match).
pub const REDACTED_TOKEN: &str = "[REDACTED]";

/// Active commercial subjects that must never appear on a public exhibit.
/// Loaded when brand `private_client_names` is empty and the forbid floor is on.
pub const DEFAULT_PRIVATE_CLIENT_NAMES: &[&str] =
    &["Courtroom.ai", "Courtroom", "Vince Pilot", "Vince"];

/// Hosts treated as calendar / discovery-call booking surfaces.
const CALENDAR_HOST_MARKERS: &[&str] = &[
    "calendly.com",
    "calendar.google.com",
    "calendar.app.google",
    "outlook.office.com/calendar",
    "outlook.live.com/calendar",
    "cal.com/",
    "tidycal.com",
];

/// Intake / airlock hosts allowed even when external-link forbid is on.
const EXTERNAL_LINK_ALLOWLIST_HOSTS: &[&str] = &["cos.fnox.io", "fourthwatch.ai"];

/// Extensions treated as confidential source citations when paired with `:line`.
const CITATION_EXTENSIONS: &[&str] = &[
    "rs", "ex", "exs", "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "go", "toml", "md", "heex",
    "hrl", "erl", "java", "kt", "swift", "c", "h", "cc", "cpp", "hpp",
];

/// Policy derived from `fourth-watch.brand.toml` `[redaction_court]` (+ names).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RedactionPolicy {
    pub forbid_private_client_names: bool,
    pub forbid_local_user_paths: bool,
    pub forbid_bearer_tokens: bool,
    pub forbid_external_links_in_post_body: bool,
    pub forbid_calendar_links: bool,
    /// Extra private client / operator names to strip (case-insensitive).
    pub private_names: Vec<String>,
}

impl RedactionPolicy {
    /// Strict Fourth Watch defaults when brand omits the section.
    pub fn fourth_watch_defaults() -> Self {
        Self {
            forbid_private_client_names: true,
            forbid_local_user_paths: true,
            forbid_bearer_tokens: true,
            forbid_external_links_in_post_body: true,
            forbid_calendar_links: true,
            private_names: DEFAULT_PRIVATE_CLIENT_NAMES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        }
    }

    /// When private-name forbid is on and the denylist is empty, load
    /// [`DEFAULT_PRIVATE_CLIENT_NAMES`] so brand omission cannot strip-and-ship.
    pub fn with_default_denylist_if_empty(mut self) -> Self {
        if self.forbid_private_client_names && self.private_names.is_empty() {
            self.private_names = DEFAULT_PRIVATE_CLIENT_NAMES
                .iter()
                .map(|s| (*s).to_string())
                .collect();
        }
        self
    }
}

/// Typed anonymize-court refusal (CLI flag / residual).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnonymizeRefusal {
    pub code: &'static str,
    pub class: &'static str,
    pub broken_term: &'static str,
    pub detail: String,
}

impl std::fmt::Display for AnonymizeRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]: {}", self.code, self.broken_term, self.detail)
    }
}

impl std::error::Error for AnonymizeRefusal {}

/// One typed refuse-court finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RedactionFinding {
    /// Stable rule id.
    pub rule: &'static str,
    /// Evidence naming the violated law without echoing the raw private string.
    pub detail: String,
}

/// Typed refusal to publish: the candidate text carries private material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RedactionRefusal {
    /// Always [`REFUSED_REDACTION`].
    pub code: &'static str,
    /// Chatman failure-taxonomy term ([`BROKEN_TERM`]).
    pub broken_term: &'static str,
    /// Every rule the candidate failed; never empty.
    pub findings: Vec<RedactionFinding>,
}

impl std::fmt::Display for RedactionRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}[{}]: {} finding(s)",
            self.code,
            self.broken_term,
            self.findings.len()
        )
    }
}

impl std::error::Error for RedactionRefusal {}

/// Fail-closed redaction court over brand policy floors + private-name denylist.
#[derive(Debug, Clone)]
pub struct RedactionCourt {
    private_names: Vec<String>,
    forbid_private_client_names: bool,
    forbid_local_user_paths: bool,
    forbid_bearer_tokens: bool,
    forbid_external_links_in_post_body: bool,
    forbid_calendar_links: bool,
}

impl Default for RedactionCourt {
    fn default() -> Self {
        Self::with_default_clients()
    }
}

impl RedactionCourt {
    /// Court seeded with [`DEFAULT_PRIVATE_CLIENT_NAMES`] and Fourth Watch floors.
    pub fn with_default_clients() -> Self {
        Self::from_policy(&RedactionPolicy::fourth_watch_defaults())
    }

    /// Court seeded with an explicit denylist (case-insensitive match).
    /// Home paths, citations, bearer, calendar, and external-link floors stay on
    /// (Fourth Watch refuse defaults) so empty-denylist probes still refuse paths.
    /// An explicitly empty denylist turns the name floor off (no default fill).
    pub fn with_private_names<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut policy = RedactionPolicy::fourth_watch_defaults();
        policy.private_names = names
            .into_iter()
            .map(|s| s.as_ref().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        policy.forbid_private_client_names = !policy.private_names.is_empty();
        Self::from_policy_raw(&policy)
    }

    /// Build the wired refuse court from a brand [`RedactionPolicy`].
    /// Empty `private_names` + forbid-on loads [`DEFAULT_PRIVATE_CLIENT_NAMES`].
    pub fn from_policy(policy: &RedactionPolicy) -> Self {
        let policy = policy.clone().with_default_denylist_if_empty();
        Self::from_policy_raw(&policy)
    }

    fn from_policy_raw(policy: &RedactionPolicy) -> Self {
        let mut private_names = policy.private_names.clone();
        // Longer names first so "Vince Pilot" wins over "Vince".
        private_names.sort_by_key(|s| std::cmp::Reverse(s.len()));
        private_names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        Self {
            private_names,
            forbid_private_client_names: policy.forbid_private_client_names,
            forbid_local_user_paths: policy.forbid_local_user_paths,
            forbid_bearer_tokens: policy.forbid_bearer_tokens,
            forbid_external_links_in_post_body: policy.forbid_external_links_in_post_body,
            forbid_calendar_links: policy.forbid_calendar_links,
        }
    }

    /// Collect every redaction finding in `text` (may be empty when clean).
    pub fn findings(&self, text: &str) -> Vec<RedactionFinding> {
        let mut out = Vec::new();
        if self.forbid_private_client_names {
            self.collect_private_names(text, &mut out);
        }
        if self.forbid_local_user_paths {
            self.collect_home_paths(text, &mut out);
        }
        self.collect_citations(text, &mut out);
        if self.forbid_bearer_tokens {
            self.collect_bearer_tokens(text, &mut out);
        }
        if self.forbid_calendar_links {
            self.collect_calendar_links(text, &mut out);
        }
        if self.forbid_external_links_in_post_body {
            self.collect_external_links(text, &mut out);
        }
        self.collect_emails(text, &mut out);
        out
    }

    /// Admit `text` for public publish, or refuse with typed findings.
    pub fn admit(&self, text: &str) -> Result<(), RedactionRefusal> {
        let findings = self.findings(text);
        if findings.is_empty() {
            Ok(())
        } else {
            Err(RedactionRefusal {
                code: REFUSED_REDACTION,
                broken_term: BROKEN_TERM,
                findings,
            })
        }
    }

    fn collect_private_names(&self, text: &str, out: &mut Vec<RedactionFinding>) {
        let lower = text.to_ascii_lowercase();
        for name in &self.private_names {
            let needle = name.to_ascii_lowercase();
            if needle.is_empty() {
                continue;
            }
            let mut search_from = 0usize;
            while let Some(rel) = lower[search_from..].find(&needle) {
                let start = search_from + rel;
                let end = start + needle.len();
                if is_name_boundary(text, start, end) {
                    out.push(RedactionFinding {
                        rule: RULE_PRIVATE_CLIENT_NAME,
                        detail: format!(
                            "private client name present ({REDACTED_TOKEN}; denylist entry length {})",
                            name.len()
                        ),
                    });
                    return; // one finding per rule class is enough to refuse
                }
                search_from = start + 1;
            }
        }
    }

    fn collect_home_paths(&self, text: &str, out: &mut Vec<RedactionFinding>) {
        for marker in ["/Users/", "/home/"] {
            if text.contains(marker) {
                out.push(RedactionFinding {
                    rule: RULE_PRIVATE_HOME_PATH,
                    detail: format!(
                        "private home path present ({REDACTED_TOKEN}; prefix {marker})"
                    ),
                });
                return;
            }
        }
    }

    fn collect_citations(&self, text: &str, out: &mut Vec<RedactionFinding>) {
        // Match path-ish tokens ending in .<ext>:<digits> (optional :<col>).
        let bytes = text.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            if let Some((start, end)) = scan_citation_at(text, i) {
                let token = &text[start..end];
                if citation_extension_ok(token) && !is_already_redacted_citation(token) {
                    out.push(RedactionFinding {
                        rule: RULE_UNREDACTED_CITATION,
                        detail: format!(
                            "unredacted file:line citation present ({REDACTED_TOKEN}; {} bytes)",
                            token.len()
                        ),
                    });
                    return;
                }
                i = end;
            } else {
                i += 1;
            }
        }
    }

    fn collect_bearer_tokens(&self, text: &str, out: &mut Vec<RedactionFinding>) {
        if contains_bearer_residual(text) {
            out.push(RedactionFinding {
                rule: RULE_BEARER_TOKEN,
                detail: format!("bearer token residual present ({REDACTED_TOKEN})"),
            });
        }
    }

    fn collect_calendar_links(&self, text: &str, out: &mut Vec<RedactionFinding>) {
        let lower = text.to_ascii_lowercase();
        for marker in CALENDAR_HOST_MARKERS {
            if lower.contains(marker) {
                out.push(RedactionFinding {
                    rule: RULE_CALENDAR_LINK,
                    detail: format!(
                        "calendar booking link present ({REDACTED_TOKEN}; host class refused)"
                    ),
                });
                return;
            }
        }
    }

    fn collect_external_links(&self, text: &str, out: &mut Vec<RedactionFinding>) {
        if let Some(marker) = first_disallowed_external_link_marker(text) {
            out.push(RedactionFinding {
                rule: RULE_EXTERNAL_LINK,
                detail: format!(
                    "external link marker {marker:?} present ({REDACTED_TOKEN}; post-body forbid)"
                ),
            });
        }
    }

    fn collect_emails(&self, text: &str, out: &mut Vec<RedactionFinding>) {
        if contains_email(text) {
            out.push(RedactionFinding {
                rule: RULE_EMAIL,
                detail: format!("email address present ({REDACTED_TOKEN}; refuse, never strip-and-ship)"),
            });
        }
    }
}

fn is_name_boundary(text: &str, start: usize, end: usize) -> bool {
    let before_ok = start == 0
        || !text
            .get(start - 1..start)
            .and_then(|s| s.chars().next())
            .is_some_and(is_name_char);
    let after_ok = end >= text.len()
        || !text
            .get(end..end + 1)
            .and_then(|s| s.chars().next())
            .is_some_and(is_name_char);
    before_ok && after_ok
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-'
}

fn scan_citation_at(text: &str, start: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    if start >= bytes.len() {
        return None;
    }
    // Citation path chars: letters, digits, _ . / -
    let b = bytes[start];
    if !(b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'/' || b == b'-') {
        return None;
    }
    let mut i = start;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric()
            || bytes[i] == b'_'
            || bytes[i] == b'.'
            || bytes[i] == b'/'
            || bytes[i] == b'-')
    {
        i += 1;
    }
    // Require :<digits> after the path token.
    if i >= bytes.len() || bytes[i] != b':' {
        return None;
    }
    let line_start = i + 1;
    let mut j = line_start;
    while j < bytes.len() && bytes[j].is_ascii_digit() {
        j += 1;
    }
    if j == line_start {
        return None;
    }
    // Optional :<col>
    if j + 1 < bytes.len() && bytes[j] == b':' {
        let col_start = j + 1;
        let mut k = col_start;
        while k < bytes.len() && bytes[k].is_ascii_digit() {
            k += 1;
        }
        if k > col_start {
            j = k;
        }
    }
    Some((start, j))
}

fn citation_extension_ok(token: &str) -> bool {
    // token looks like path.ext:line or path.ext:line:col — find last '.' before first ':'.
    let before_colon = match token.find(':') {
        Some(i) => &token[..i],
        None => return false,
    };
    let ext = match before_colon.rsplit_once('.') {
        Some((_, e)) => e,
        None => return false,
    };
    let ext_l = ext.to_ascii_lowercase();
    CITATION_EXTENSIONS.iter().any(|e| *e == ext_l)
}

fn is_already_redacted_citation(token: &str) -> bool {
    let before_colon = token.split(':').next().unwrap_or(token);
    before_colon.contains(REDACTED_TOKEN)
        || before_colon.eq_ignore_ascii_case("REDACTED")
        || before_colon == "*"
        || before_colon.contains(REDACTED_CITE)
}

/// Fail closed when `--public-anonymize` is absent on a public target.
pub fn require_public_anonymize(flag: bool) -> Result<(), AnonymizeRefusal> {
    if flag {
        return Ok(());
    }
    Err(AnonymizeRefusal {
        code: REFUSED_NO_PUBLIC_ANONYMIZE,
        class: "refused",
        broken_term: BROKEN_TERM_NO_AUTHORITY,
        detail: "castle publish public targets require --public-anonymize (redaction court)"
            .to_string(),
    })
}

fn contains_bearer_residual(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if lower.contains("bearer ") {
        return true;
    }
    // sk-… API keys / eyJ… JWT blobs (same surface the strip helper targets).
    let bytes = text.as_bytes();
    let lower_bytes = lower.as_bytes();
    for prefix in [b"sk-".as_slice(), b"eyj".as_slice()] {
        let mut i = 0usize;
        while i < lower_bytes.len() {
            if lower_bytes[i..].starts_with(prefix) {
                let mut j = i + prefix.len();
                let mut n = 0usize;
                while j < bytes.len() && !bytes[j].is_ascii_whitespace() && bytes[j] != b'"' {
                    n += 1;
                    j += 1;
                }
                if n >= 8 {
                    return true;
                }
                i = j.max(i + 1);
                continue;
            }
            i += 1;
        }
    }
    false
}

/// First `https://` / `http://` / `www.` marker whose host is not allowlisted.
fn first_disallowed_external_link_marker(text: &str) -> Option<&'static str> {
    const MARKERS: [&str; 3] = ["https://", "http://", "www."];
    let lower = text.to_ascii_lowercase();
    for marker in MARKERS {
        let mut search_from = 0usize;
        while let Some(rel) = lower[search_from..].find(marker) {
            let abs = search_from + rel;
            let boundary_free = !lower[..abs]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '.');
            if boundary_free {
                let after = &lower[abs + marker.len()..];
                let host: String = after
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
                    .collect();
                if !EXTERNAL_LINK_ALLOWLIST_HOSTS
                    .iter()
                    .any(|allowed| host == *allowed || host.ends_with(&format!(".{allowed}")))
                {
                    return Some(marker);
                }
            }
            search_from = abs + marker.len();
        }
    }
    None
}

fn match_email(chars: &[char], i: usize) -> Option<usize> {
    // local@domain.tld — conservative ASCII scanner
    if !chars[i].is_ascii_alphanumeric() {
        return None;
    }
    let mut j = i;
    while j < chars.len()
        && (chars[j].is_ascii_alphanumeric()
            || chars[j] == '.'
            || chars[j] == '_'
            || chars[j] == '%'
            || chars[j] == '+'
            || chars[j] == '-')
    {
        j += 1;
    }
    if j >= chars.len() || chars[j] != '@' {
        return None;
    }
    j += 1;
    let domain_start = j;
    while j < chars.len()
        && (chars[j].is_ascii_alphanumeric() || chars[j] == '.' || chars[j] == '-')
    {
        j += 1;
    }
    if j <= domain_start + 2 {
        return None;
    }
    let domain: String = chars[domain_start..j].iter().collect();
    if !domain.contains('.') {
        return None;
    }
    Some(j)
}

fn contains_email(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    (0..chars.len()).any(|i| match_email(&chars, i).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn court(names: &[&str]) -> RedactionCourt {
        RedactionCourt::with_private_names(names.iter().copied())
    }

    #[test]
    fn refuses_without_flag() {
        let err = require_public_anonymize(false).unwrap_err();
        assert_eq!(err.code, REFUSED_NO_PUBLIC_ANONYMIZE);
        assert_eq!(err.class, "refused");
        assert!(err.detail.contains("public-anonymize"));
    }

    #[test]
    fn admits_with_flag() {
        assert!(require_public_anonymize(true).is_ok());
    }

    #[test]
    fn admits_clean_public_copy() {
        let text = "Stop shipping unverified claims. Seal the ledger, then publish the receipt.";
        assert!(court(&["AcmeCorp"]).admit(text).is_ok());
        assert!(RedactionCourt::with_default_clients().admit(text).is_ok());
    }

    #[test]
    fn refuses_private_client_name_without_echoing_it() {
        let secret = "AcmeHoldingsLLC";
        let text = format!("This week {secret} cleared Monday prepaid.");
        let err = court(&[secret])
            .admit(&text)
            .expect_err("private client name must refuse");
        assert_eq!(err.code, REFUSED_REDACTION);
        assert_eq!(err.broken_term, BROKEN_TERM);
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_PRIVATE_CLIENT_NAME));
        let inspected = format!("{err:?}");
        assert!(
            !inspected.contains(secret),
            "refusal must not echo private client name: {inspected}"
        );
        assert!(
            inspected.contains(REDACTED_TOKEN)
                || err
                    .findings
                    .iter()
                    .any(|f| f.detail.contains(REDACTED_TOKEN))
        );
    }

    #[test]
    fn refuses_default_denylist_client_names() {
        for name in ["Courtroom.ai", "Vince Pilot"] {
            let text = format!("Pilot update from {name} this week.");
            let err = RedactionCourt::with_default_clients()
                .admit(&text)
                .expect_err("default denylist must refuse");
            assert!(err
                .findings
                .iter()
                .any(|f| f.rule == RULE_PRIVATE_CLIENT_NAME));
            let inspected = format!("{err:?}");
            assert!(
                !inspected.contains(name),
                "refusal must not echo {name}: {inspected}"
            );
            assert!(err
                .findings
                .iter()
                .any(|f| f.detail.contains(REDACTED_TOKEN)));
        }
    }

    #[test]
    fn refuses_users_home_path_without_echoing_it() {
        let path = "/Users/sac/castle/src/publish/linkedin.rs";
        let text = format!("See the deck at {path} for the carousel law.");
        let err = court(&[])
            .admit(&text)
            .expect_err("/Users/ path must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_PRIVATE_HOME_PATH));
        let inspected = format!("{err:?}");
        assert!(
            !inspected.contains("/Users/sac"),
            "refusal must not echo /Users/ path: {inspected}"
        );
        assert!(err.findings.iter().any(|f| f.detail.contains("/Users/")));
    }

    #[test]
    fn refuses_linux_home_path() {
        let text = "Artifact landed under /home/ops/secrets/board.pdf before seal.";
        let err = court(&[]).admit(text).expect_err("/home/ path must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_PRIVATE_HOME_PATH));
        assert!(!format!("{err:?}").contains("/home/ops"));
    }

    #[test]
    fn refuses_unredacted_file_line_citation_without_echoing_it() {
        let cite = "src/publish/linkedin.rs:42";
        let text = format!("The lint court lives at {cite}.");
        let err = court(&[])
            .admit(&text)
            .expect_err("unredacted citation must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_UNREDACTED_CITATION));
        let inspected = format!("{err:?}");
        assert!(
            !inspected.contains(cite),
            "refusal must not echo citation: {inspected}"
        );
        assert!(err
            .findings
            .iter()
            .any(|f| f.detail.contains(REDACTED_TOKEN)));
    }

    #[test]
    fn admits_already_redacted_citation_placeholder() {
        let text = "Evidence pointer: [REDACTED]:12 — details in the sealed receipt.";
        assert!(court(&[]).admit(text).is_ok());
    }

    #[test]
    fn name_match_is_boundary_aware() {
        // "Vince" must not fire inside "Vincent".
        let text = "Vincent sealed the drop lease before outbound.";
        assert!(RedactionCourt::with_default_clients().admit(text).is_ok());
    }

    #[test]
    fn from_policy_loads_default_denylist_when_empty() {
        let policy = RedactionPolicy {
            forbid_private_client_names: true,
            private_names: Vec::new(),
            ..RedactionPolicy::fourth_watch_defaults()
        };
        // Clear names after defaults to simulate empty brand projection.
        let policy = RedactionPolicy {
            private_names: Vec::new(),
            ..policy
        };
        let court = RedactionCourt::from_policy(&policy);
        let err = court
            .admit("Board packet mentions Courtroom.ai by name.")
            .expect_err("default denylist must load");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_PRIVATE_CLIENT_NAME));
    }

    #[test]
    fn refuses_calendar_bearer_and_external_floors() {
        let court = RedactionCourt::with_default_clients();
        let cal = court
            .admit("Book via https://calendly.com/acme/intro")
            .expect_err("calendar");
        assert!(cal.findings.iter().any(|f| f.rule == RULE_CALENDAR_LINK));

        let bearer = court
            .admit("Authorization: Bearer sk-live-abcdefghijklmnopqrstuvwxyz")
            .expect_err("bearer");
        assert!(bearer.findings.iter().any(|f| f.rule == RULE_BEARER_TOKEN));

        let ext = court
            .admit("See https://evil.example/leak for details")
            .expect_err("external");
        assert!(ext.findings.iter().any(|f| f.rule == RULE_EXTERNAL_LINK));
    }

    #[test]
    fn refuses_email_without_echoing_it() {
        let addr = "ops@evil.example";
        let text = format!("Contact {addr} for the drop.");
        let err = RedactionCourt::with_default_clients()
            .admit(&text)
            .expect_err("email must refuse");
        assert!(err.findings.iter().any(|f| f.rule == RULE_EMAIL));
        let inspected = format!("{err:?}");
        assert!(
            !inspected.contains(addr),
            "refusal must not echo email: {inspected}"
        );
        assert!(inspected.contains(REDACTED_TOKEN));
    }

    #[test]
    fn allows_airlock_host_external_link() {
        let court = RedactionCourt::with_default_clients();
        assert!(court
            .admit("Featured drop: https://cos.fnox.io/drop/lease-public")
            .is_ok());
    }

}
