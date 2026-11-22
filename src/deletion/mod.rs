//! First-Party 45-Day Data-Deletion Receipt Tracker (CASTLE-SPEC-26.11.20).
//!
//! CCPA/CPRA deletion-request generator plus a `DeletionCase` JSON state
//! machine that tracks the two statutory clocks of
//! Cal. Civ. Code § 1798.130(a)(2):
//!
//! - **primary clock** — 45 days from receipt of the request;
//! - **extended clock** — 90 days total, reachable only when the business
//!   notified the consumer of a single extension within the primary window.
//!
//! Terminal transitions, from `PENDING_RECEIPT` only:
//!
//! - `PENDING_RECEIPT` -> `VERIFIED_ERASED` — a written deletion receipt
//!   whose BLAKE3-256 digest matches the pre-committed expected digest;
//! - `PENDING_RECEIPT` -> `STATUTORY_REFUSED` — a typed refusal
//!   (§ 1798.105(d) exception, verification failure, or deadline expiry).
//!
//! California SB 923 (signed 2026-09-27, effective 2027-01-01) is carried
//! as a first-class citation: from its effective date the § 1798.105
//! deletion right covers personal information collected *about* the
//! consumer from any source, not only information collected *from* the
//! consumer. The case records whether the request asserts that expanded
//! scope, and the generated letter cites the applicable branch.
//!
//! Determinism law (mirrors `crate::trust_chain` / `receipt_ocel`
//! discipline): the module reads no wall clock and performs no I/O. Every
//! date — including "today" for clock evaluation — arrives as a validated
//! [`CivilDate`]. Identical inputs produce byte-identical letters,
//! case identifiers, and state digests (asserted by test).
//!
//! Digest law:
//!
//! ```text
//! case_id      = blake3(canonical_json(intake))
//! state_digest = blake3(canonical_json(core_view(case)))   // excludes state_digest itself
//! receipt digest computed and compared, never trusted from text alone
//! ```
//!
//! Zero-emotion discipline: all rendered text is formal statutory prose.
//! No appeals, no adjectives of urgency, no persuasion. The letter states
//! the request, the citations, the clocks, and the required response.

use serde::{Deserialize, Serialize};
use serde_json_canonicalizer;
use std::fmt;

// ---------------------------------------------------------------------------
// Statutory constants
// ---------------------------------------------------------------------------

/// Primary statutory response window, Cal. Civ. Code § 1798.130(a)(2).
pub const PRIMARY_RESPONSE_DAYS: u32 = 45;

/// Extended statutory response window (45 additional days, one extension
/// only, extension notice due within the primary window).
pub const EXTENDED_RESPONSE_DAYS: u32 = 90;

/// § 1798.105(a) — the deletion right itself.
pub const CITATION_DELETE: &str = "Cal. Civ. Code § 1798.105(a)";

/// § 1798.130(a)(2) — 45-day response clock, single 45-day extension.
pub const CITATION_RESPONSE_CLOCK: &str = "Cal. Civ. Code § 1798.130(a)(2)";

/// § 1798.105(c) — business shall direct service providers, contractors,
/// and third parties to delete.
pub const CITATION_SERVICE_PROVIDERS: &str = "Cal. Civ. Code § 1798.105(c)";

/// § 1798.105(d) — enumerated statutory exceptions to deletion.
pub const CITATION_EXCEPTIONS: &str = "Cal. Civ. Code § 1798.105(d)";

/// § 1798.125 — non-discrimination for exercising CCPA rights.
pub const CITATION_NON_DISCRIMINATION: &str = "Cal. Civ. Code § 1798.125";

/// SB 923 (2026): § 1798.105 deletion right expanded to personal
/// information collected about the consumer from any source.
pub const CITATION_SB923: &str =
    "California SB 923 (2026), signed 2026-09-27, amending Cal. Civ. Code § 1798.105";

/// First day the SB 923 expanded deletion scope applies.
pub const SB923_EFFECTIVE: CivilDate = CivilDate {
    year: 2027,
    month: 1,
    day: 1,
};

// ---------------------------------------------------------------------------
// Civil date
// ---------------------------------------------------------------------------

/// Proleptic Gregorian civil date, validated at construction, serialized as
/// `"YYYY-MM-DD"`. Domain: years 1000..=9999.
///
/// `std` carries no calendar type and this lane may not add dependencies,
/// so the civil <-> epoch-day conversions below are the standard
/// days-from-civil algorithm (Hinnant 2017). Pure, total on the domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CivilDate {
    year: i32,
    month: u8,
    day: u8,
}

/// Returns whether `year` is a leap year in the proleptic Gregorian calendar.
#[must_use]
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Number of days in `month` (1..=12) of `year`; `None` if the month is out
/// of range.
#[must_use]
pub fn days_in_month(year: i32, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if is_leap_year(year) => Some(29),
        2 => Some(28),
        _ => None,
    }
}

/// Epoch-day conversion (days_from_civil, Hinnant 2017). Day 0 is
/// 1970-01-01.
#[must_use]
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let y = i64::from(if month <= 2 { year - 1 } else { year });
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = (i64::from(month) + 9) % 12; // [0, 11], March = 0
    let doy = (153 * mp + 2) / 5 + i64::from(day) - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100; // [0, 146096]
    era * 146_097 + doe + doy - 719_468
}

/// Inverse epoch-day conversion (civil_from_days, Hinnant 2017).
#[must_use]
fn civil_from_days(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u8, d)
}

impl CivilDate {
    /// Constructs a validated civil date.
    ///
    /// # Errors
    /// [`DeletionError::RefusedInvalidDate`] when the year is outside
    /// 1000..=9999, the month outside 1..=12, or the day outside the
    /// month's length.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, DeletionError> {
        if !(1000..=9999).contains(&year) {
            return Err(DeletionError::RefusedInvalidDate {
                reason: format!("year {year} outside 1000..=9999"),
            });
        }
        let month_len = days_in_month(year, month).ok_or(DeletionError::RefusedInvalidDate {
            reason: format!("month {month} outside 1..=12"),
        })?;
        if day < 1 || day > month_len {
            return Err(DeletionError::RefusedInvalidDate {
                reason: format!("day {day} invalid for {year}-{month:02} (length {month_len})"),
            });
        }
        Ok(Self { year, month, day })
    }

    /// Parses the canonical `"YYYY-MM-DD"` representation.
    ///
    /// # Errors
    /// [`DeletionError::RefusedInvalidDate`] on shape or calendar failure.
    pub fn parse(text: &str) -> Result<Self, DeletionError> {
        let parts: Vec<&str> = text.split('-').collect();
        if parts.len() != 3
            || parts[0].len() != 4
            || parts[1].len() != 2
            || parts[2].len() != 2
            || !parts
                .iter()
                .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err(DeletionError::RefusedInvalidDate {
                reason: format!("expected YYYY-MM-DD, got {text:?}"),
            });
        }
        let year: i32 = parts[0]
            .parse()
            .map_err(|_| DeletionError::RefusedInvalidDate { reason: format!("bad year in {text:?}") })?;
        let month: u8 = parts[1]
            .parse()
            .map_err(|_| DeletionError::RefusedInvalidDate { reason: format!("bad month in {text:?}") })?;
        let day: u8 = parts[2]
            .parse()
            .map_err(|_| DeletionError::RefusedInvalidDate { reason: format!("bad day in {text:?}") })?;
        Self::new(year, month, day)
    }

    /// Epoch-day number of this date (1970-01-01 = 0).
    #[must_use]
    pub fn epoch_day(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    /// The date `n` calendar days from this date; `n` may be negative.
    #[must_use]
    pub fn add_days(self, n: i64) -> Self {
        let (year, month, day) = civil_from_days(self.epoch_day() + n);
        Self { year, month, day }
    }

    /// Signed day count from `earlier` to `self` (`self - earlier`).
    #[must_use]
    pub fn days_since(self, earlier: Self) -> i64 {
        self.epoch_day() - earlier.epoch_day()
    }

    /// The canonical `"YYYY-MM-DD"` text.
    #[must_use]
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl fmt::Display for CivilDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.iso())
    }
}

impl From<CivilDate> for String {
    fn from(d: CivilDate) -> String {
        d.iso()
    }
}

impl TryFrom<String> for CivilDate {
    type Error = DeletionError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        CivilDate::parse(&text)
    }
}

// ---------------------------------------------------------------------------
// State machine vocabulary
// ---------------------------------------------------------------------------

/// The three states of a deletion case. Unit variants serialize as their
/// SCREAMING_SNAKE names in the case JSON; those names are the spec's
/// standing vocabulary, not Rust style, hence the allow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum DeletionState {
    /// Request on file, deletion receipt not yet verified.
    PENDING_RECEIPT,
    /// Verified deletion receipt on file; case closed erased.
    VERIFIED_ERASED,
    /// Typed statutory refusal on file; case closed refused.
    STATUTORY_REFUSED,
}

/// The enumerated § 1798.105(d) statutory exceptions. Descriptors cite the
/// section without subsection indices so no enumeration drift is asserted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatutoryException {
    /// Exception for completing the requested transaction or service.
    TransactionCompletion,
    /// Exception for security and integrity purposes.
    SecurityIntegrity,
    /// Exception for identifying and repairing impairing errors.
    ErrorRepair,
    /// Exception for free speech or other legal rights.
    FreeSpeechRights,
    /// Exception for public or peer-reviewed research.
    ScientificResearch,
    /// Exception for compliance with a legal obligation.
    LegalObligation,
    /// Exception for solely internal uses compatible with the context.
    CompatibleInternalUse,
}

impl StatutoryException {
    /// Zero-emotion citation text for this exception.
    #[must_use]
    pub fn citation(&self) -> String {
        let descriptor = match self {
            Self::TransactionCompletion => "transaction completion exception",
            Self::SecurityIntegrity => "security and integrity exception",
            Self::ErrorRepair => "error repair exception",
            Self::FreeSpeechRights => "free speech and legal rights exception",
            Self::ScientificResearch => "public or peer-reviewed research exception",
            Self::LegalObligation => "legal obligation exception",
            Self::CompatibleInternalUse => "compatible internal use exception",
        };
        format!("{CITATION_EXCEPTIONS} — {descriptor}")
    }
}

/// Typed statutory refusal taxonomy for `STATUTORY_REFUSED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefusalKind {
    /// Business invokes an enumerated § 1798.105(d) exception.
    StatutoryException(StatutoryException),
    /// Consumer identity could not be verified under § 1798.130(a)(2).
    VerificationFailed,
    /// The applicable statutory clock expired with no deletion receipt.
    DeadlineExpired,
}

impl RefusalKind {
    /// Citation text owed to the consumer for this refusal, per
    /// § 1798.105(d) / § 1798.130(a)(2).
    #[must_use]
    pub fn citation(&self) -> String {
        match self {
            Self::StatutoryException(e) => e.citation(),
            Self::VerificationFailed => format!(
                "{CITATION_RESPONSE_CLOCK} — consumer identity not verifiable"
            ),
            Self::DeadlineExpired => format!(
                "{CITATION_RESPONSE_CLOCK} — applicable statutory response window expired without a deletion receipt"
            ),
        }
    }

    /// Machine-readable discriminator for the case JSON.
    #[must_use]
    pub fn kind_code(&self) -> &'static str {
        match self {
            Self::StatutoryException(_) => "STATUTORY_EXCEPTION",
            Self::VerificationFailed => "VERIFICATION_FAILED",
            Self::DeadlineExpired => "DEADLINE_EXPIRED",
        }
    }
}

/// Phase of the statutory clock on an observed date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockPhase {
    /// On or before the primary 45-day deadline.
    WithinPrimaryWindow {
        /// Days remaining to the primary deadline (0 on the deadline day).
        days_remaining: u32,
    },
    /// Past the primary deadline, extension was noticed in time, and on or
    /// before the extended 90-day deadline.
    ExtensionWindow {
        /// Days elapsed past the primary deadline.
        days_since_primary: u32,
        /// Days remaining to the extended deadline (0 on the deadline day).
        days_remaining_to_extended: u32,
    },
    /// Past the applicable statutory deadline with no verified receipt.
    StatutoryDeadlineBreached {
        /// Days past the applicable deadline.
        days_over: u32,
    },
}

// ---------------------------------------------------------------------------
// Case records
// ---------------------------------------------------------------------------

/// Consumer identity as stated on the request. Single-line fields only;
/// embedded newlines are refused at intake so rendered letters stay
/// line-deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumerIdentity {
    /// Full legal name of the requesting consumer.
    pub full_legal_name: String,
    /// Contact email address.
    pub email: String,
    /// Single-line mailing address.
    pub mailing_address: String,
    /// Optional telephone number.
    pub phone: Option<String>,
}

/// Channel through which the request is submitted or received.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestChannel {
    /// Business web form.
    WebForm,
    /// Email to the designated request address.
    Email,
    /// Postal mail.
    PostalMail,
    /// Toll-free or other designated telephone line.
    Telephone,
    /// Authorized agent acting on the consumer's behalf.
    AuthorizedAgent,
}

impl RequestChannel {
    /// Zero-emotion label used in rendered letters.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::WebForm => "web form",
            Self::Email => "email",
            Self::PostalMail => "postal mail",
            Self::Telephone => "telephone",
            Self::AuthorizedAgent => "authorized agent",
        }
    }
}

/// Intake record for a new deletion case. `intake_sequence` disambiguates
/// otherwise identical requests received the same day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletionIntake {
    /// Consumer identity as stated on the request.
    pub consumer: ConsumerIdentity,
    /// Data categories subject to deletion; non-empty, single-line items.
    pub data_categories: Vec<String>,
    /// Submission channels; non-empty.
    pub channels: Vec<RequestChannel>,
    /// Date the request was transmitted by the consumer.
    pub request_sent: CivilDate,
    /// Date the request was received by the first party (clock origin).
    pub received: CivilDate,
    /// Caller-supplied intake counter, feeding the case identifier.
    pub intake_sequence: u64,
    /// Whether the request asserts the SB 923 expanded scope (personal
    /// information collected *about* the consumer from any source).
    pub collected_about_scope: bool,
}

/// Extension notice record, § 1798.130(a)(2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionRecord {
    /// Date the extension notice was transmitted (must fall inside the
    /// primary window).
    pub notice_sent: CivilDate,
    /// BLAKE3-256 hex digest of the notice text.
    pub notice_digest: String,
    /// New statutory deadline (`received` + 90 days).
    pub new_deadline: CivilDate,
}

/// Verified deletion receipt closing the case as `VERIFIED_ERASED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletionReceipt {
    /// Date the deletion receipt was observed.
    pub observed_on: CivilDate,
    /// Response method of the receipt (e.g. `"email"`).
    pub method: String,
    /// Categories the receipt states as erased.
    pub erased_categories: Vec<String>,
    /// BLAKE3-256 hex digest of the receipt text, verified against the
    /// pre-committed expected digest at transition time.
    pub receipt_digest: String,
}

/// Append-only transition record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionRecord {
    /// State before the transition.
    pub from: DeletionState,
    /// State after the transition.
    pub to: DeletionState,
    /// Date the transition was recorded.
    pub at: CivilDate,
    /// Machine-readable basis code.
    pub basis: String,
}

/// Typed refusal record closing the case as `STATUTORY_REFUSED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefusalRecord {
    /// Typed refusal taxonomy entry.
    pub kind: RefusalKind,
    /// Citation text owed to the consumer.
    pub citation: String,
    /// Verbatim basis statement (single line).
    pub basis: String,
    /// Date the refusal was recorded.
    pub recorded_on: CivilDate,
}

/// Receipt input for the `VERIFIED_ERASED` transition. `expected_digest`
/// is the pre-committed BLAKE3-256 hex digest; the computed digest of
/// `receipt_text` must match it or the transition is refused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErasureReceiptInput {
    /// Verbatim deletion receipt text.
    pub receipt_text: String,
    /// Date the receipt was observed.
    pub observed_on: CivilDate,
    /// Response method (single line).
    pub method: String,
    /// Categories the receipt states as erased.
    pub erased_categories: Vec<String>,
    /// Pre-committed expected BLAKE3-256 hex digest of `receipt_text`.
    pub expected_digest: String,
}

/// The state machine. Serialized case JSON is the durable artifact; all
/// mutation goes through the typed transitions below, never through field
/// edits of a deserialized case without re-sealing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletionCase {
    /// BLAKE3-256 hex of the canonical intake JSON.
    pub case_id: String,
    /// Consumer identity.
    pub consumer: ConsumerIdentity,
    /// Data categories subject to deletion.
    pub data_categories: Vec<String>,
    /// Submission channels.
    pub channels: Vec<RequestChannel>,
    /// Whether the request asserts the SB 923 expanded scope.
    pub collected_about_scope: bool,
    /// Date the request was transmitted.
    pub request_sent: CivilDate,
    /// Date the request was received (clock origin).
    pub received: CivilDate,
    /// Primary statutory deadline: `received` + 45 days.
    pub primary_deadline: CivilDate,
    /// Extension record when the 45-day extension was noticed in time.
    pub extension: Option<ExtensionRecord>,
    /// Extended statutory deadline: `received` + 90 days.
    pub extended_deadline: Option<CivilDate>,
    /// Current state.
    pub state: DeletionState,
    /// Refusal record when state is `STATUTORY_REFUSED`.
    pub refusal: Option<RefusalRecord>,
    /// Deletion receipt when state is `VERIFIED_ERASED`.
    pub deletion_receipt: Option<DeletionReceipt>,
    /// Append-only transition history.
    pub history: Vec<TransitionRecord>,
    /// BLAKE3-256 hex over the canonical JSON of the core view (every
    /// field except this one). Tamper-evident seal.
    pub state_digest: String,
}

/// Digest preimage: every field of [`DeletionCase`] except
/// `state_digest` itself.
#[derive(Serialize)]
struct CaseCore<'a> {
    case_id: &'a String,
    consumer: &'a ConsumerIdentity,
    data_categories: &'a Vec<String>,
    channels: &'a Vec<RequestChannel>,
    collected_about_scope: &'a bool,
    request_sent: &'a CivilDate,
    received: &'a CivilDate,
    primary_deadline: &'a CivilDate,
    extension: &'a Option<ExtensionRecord>,
    extended_deadline: &'a Option<CivilDate>,
    state: &'a DeletionState,
    refusal: &'a Option<RefusalRecord>,
    deletion_receipt: &'a Option<DeletionReceipt>,
    history: &'a Vec<TransitionRecord>,
}

/// Validates a 64-character lowercase BLAKE3 hex digest shape.
fn is_blake3_hex(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Canonical JSON of any serializable value.
fn canonical_json<T: Serialize>(value: &T) -> Result<String, DeletionError> {
    serde_json_canonicalizer::to_string(value)
        .map_err(|e| DeletionError::RefusedCanonicalization { source: e.to_string() })
}

/// BLAKE3-256 hex digest of a byte slice.
#[must_use]
fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

impl DeletionCase {
    /// Opens a case in state `PENDING_RECEIPT` and derives the statutory
    /// clocks. Pure: no wall clock, no I/O.
    ///
    /// # Errors
    /// Typed `REFUSED_*` on empty identity fields, empty categories or
    /// channels, embedded newlines, or a received date earlier than the
    /// transmitted date.
    pub fn open(intake: &DeletionIntake) -> Result<Self, DeletionError> {
        if intake.consumer.full_legal_name.trim().is_empty() {
            return Err(DeletionError::RefusedEmptyField { field: "full_legal_name" });
        }
        if intake.consumer.email.trim().is_empty() {
            return Err(DeletionError::RefusedEmptyField { field: "email" });
        }
        if intake.consumer.mailing_address.trim().is_empty() {
            return Err(DeletionError::RefusedEmptyField { field: "mailing_address" });
        }
        if intake.data_categories.is_empty() || intake.data_categories.iter().any(String::is_empty) {
            return Err(DeletionError::RefusedEmptyField { field: "data_categories" });
        }
        if intake.channels.is_empty() {
            return Err(DeletionError::RefusedEmptyField { field: "channels" });
        }
        for (field, value) in [
            ("full_legal_name", &intake.consumer.full_legal_name),
            ("email", &intake.consumer.email),
            ("mailing_address", &intake.consumer.mailing_address),
        ] {
            if value.contains('\n') || value.contains('\r') {
                return Err(DeletionError::RefusedEmbeddedNewline { field });
            }
        }
        for item in &intake.data_categories {
            if item.contains('\n') || item.contains('\r') {
                return Err(DeletionError::RefusedEmbeddedNewline { field: "data_categories" });
            }
        }
        if intake.received.days_since(intake.request_sent) < 0 {
            return Err(DeletionError::RefusedClockBeforeReceipt {
                received: intake.received,
                today: intake.request_sent,
            });
        }

        let intake_json = canonical_json(intake)?;
        let case_id = blake3_hex(intake_json.as_bytes());
        let primary_deadline = intake.received.add_days(i64::from(PRIMARY_RESPONSE_DAYS));

        let mut case = Self {
            case_id,
            consumer: intake.consumer.clone(),
            data_categories: intake.data_categories.clone(),
            channels: intake.channels.clone(),
            collected_about_scope: intake.collected_about_scope,
            request_sent: intake.request_sent,
            received: intake.received,
            primary_deadline,
            extension: None,
            extended_deadline: None,
            state: DeletionState::PENDING_RECEIPT,
            refusal: None,
            deletion_receipt: None,
            history: Vec::new(),
            state_digest: String::new(),
        };
        case.history.push(TransitionRecord {
            from: DeletionState::PENDING_RECEIPT,
            to: DeletionState::PENDING_RECEIPT,
            at: intake.received,
            basis: "INTAKE_OPENED".to_string(),
        });
        case.seal();
        Ok(case)
    }

    /// Applicable statutory deadline: the extended deadline when an
    /// extension was noticed in time, otherwise the primary deadline.
    #[must_use]
    pub fn applicable_deadline(&self) -> CivilDate {
        self.extended_deadline.unwrap_or(self.primary_deadline)
    }

    /// Evaluates the statutory clock on `today`.
    ///
    /// # Errors
    /// [`DeletionError::RefusedClockBeforeReceipt`] when `today` predates
    /// the received date; [`DeletionError::ClockClosedOnTerminal`] when the
    /// case already reached a terminal state.
    pub fn clock_phase(&self, today: CivilDate) -> Result<ClockPhase, DeletionError> {
        if self.state != DeletionState::PENDING_RECEIPT {
            return Err(DeletionError::ClockClosedOnTerminal { state: self.state });
        }
        if today.days_since(self.received) < 0 {
            return Err(DeletionError::RefusedClockBeforeReceipt {
                received: self.received,
                today,
            });
        }
        let elapsed = today.days_since(self.received);
        let primary = i64::from(PRIMARY_RESPONSE_DAYS);
        let extended = i64::from(EXTENDED_RESPONSE_DAYS);
        if elapsed <= primary {
            Ok(ClockPhase::WithinPrimaryWindow {
                days_remaining: u32::try_from(primary - elapsed).unwrap_or(0),
            })
        } else if self.extension.is_some() && elapsed <= extended {
            Ok(ClockPhase::ExtensionWindow {
                days_since_primary: u32::try_from(elapsed - primary).unwrap_or(0),
                days_remaining_to_extended: u32::try_from(extended - elapsed).unwrap_or(0),
            })
        } else {
            let applicable = i64::from(if self.extension.is_some() {
                EXTENDED_RESPONSE_DAYS
            } else {
                PRIMARY_RESPONSE_DAYS
            });
            Ok(ClockPhase::StatutoryDeadlineBreached {
                days_over: u32::try_from(elapsed - applicable).unwrap_or(0),
            })
        }
    }

    /// Records the single § 1798.130(a)(2) extension. The extension notice
    /// must be transmitted within the primary window (on or before the
    /// 45-day deadline) and its digest must match the pre-committed value.
    ///
    /// # Errors
    /// Typed `REFUSED_*` on wrong state, duplicate extension, notice
    /// outside the primary window, notice predating receipt, digest
    /// mismatch, or an observation date predating the notice.
    pub fn record_extension(
        &mut self,
        notice_sent: CivilDate,
        notice_text: &str,
        expected_digest: &str,
        today: CivilDate,
    ) -> Result<TransitionRecord, DeletionError> {
        if self.state != DeletionState::PENDING_RECEIPT {
            return Err(DeletionError::RefusedIllegalTransition {
                from: self.state,
                action: "record_extension",
            });
        }
        if self.extension.is_some() {
            return Err(DeletionError::RefusedDuplicateExtension);
        }
        if notice_sent.days_since(self.received) < 0 {
            return Err(DeletionError::RefusedClockBeforeReceipt {
                received: self.received,
                today: notice_sent,
            });
        }
        if notice_sent.days_since(self.primary_deadline) > 0 {
            return Err(DeletionError::RefusedExtensionNoticeOutsidePrimaryWindow {
                notice_sent,
                primary_deadline: self.primary_deadline,
            });
        }
        if !is_blake3_hex(expected_digest) {
            return Err(DeletionError::RefusedInvalidDigest {
                reason: "expected_digest is not 64-character lowercase blake3 hex",
            });
        }
        let computed = blake3_hex(notice_text.as_bytes());
        if computed != expected_digest {
            return Err(DeletionError::RefusedReceiptDigestMismatch {
                expected: expected_digest.to_string(),
                computed,
            });
        }
        if today.days_since(notice_sent) < 0 {
            return Err(DeletionError::RefusedClockBeforeReceipt {
                received: notice_sent,
                today,
            });
        }
        let new_deadline = self.received.add_days(i64::from(EXTENDED_RESPONSE_DAYS));
        self.extension = Some(ExtensionRecord {
            notice_sent,
            notice_digest: computed,
            new_deadline,
        });
        self.extended_deadline = Some(new_deadline);
        let record = self.append_transition(
            DeletionState::PENDING_RECEIPT,
            today,
            "EXTENSION_NOTICED_WITHIN_PRIMARY_WINDOW",
        );
        self.seal();
        Ok(record)
    }

    /// Transitions `PENDING_RECEIPT` -> `VERIFIED_ERASED` when the
    /// computed BLAKE3-256 digest of `input.receipt_text` matches
    /// `input.expected_digest`. A late receipt (past the applicable
    /// deadline) still closes the case as erased; the returned record
    /// carries `deadline_breach_days` so the clock history is not erased
    /// with the data.
    ///
    /// # Errors
    /// [`DeletionError::RefusedIllegalTransition`] on a terminal state;
    /// [`DeletionError::RefusedReceiptDigestMismatch`] when the computed
    /// digest differs; typed `REFUSED_*` on digest-shape failure, a
    /// receipt observed before the request was received, or an observation
    /// date in the future.
    pub fn verify_erased(
        &mut self,
        input: &ErasureReceiptInput,
        today: CivilDate,
    ) -> Result<VerifiedErasure, DeletionError> {
        if self.state != DeletionState::PENDING_RECEIPT {
            return Err(DeletionError::RefusedIllegalTransition {
                from: self.state,
                action: "verify_erased",
            });
        }
        if !is_blake3_hex(&input.expected_digest) {
            return Err(DeletionError::RefusedInvalidDigest {
                reason: "expected_digest is not 64-character lowercase blake3 hex",
            });
        }
        let computed = blake3_hex(input.receipt_text.as_bytes());
        if computed != input.expected_digest {
            return Err(DeletionError::RefusedReceiptDigestMismatch {
                expected: input.expected_digest.clone(),
                computed,
            });
        }
        if input.observed_on.days_since(self.received) < 0 {
            return Err(DeletionError::RefusedClockBeforeReceipt {
                received: self.received,
                today: input.observed_on,
            });
        }
        if today.days_since(input.observed_on) < 0 {
            return Err(DeletionError::RefusedClockBeforeReceipt {
                received: input.observed_on,
                today,
            });
        }
        let deadline_breach_days = match input.observed_on.days_since(self.applicable_deadline()) {
            d if d > 0 => u32::try_from(d).unwrap_or(0),
            _ => 0,
        };
        self.deletion_receipt = Some(DeletionReceipt {
            observed_on: input.observed_on,
            method: input.method.clone(),
            erased_categories: input.erased_categories.clone(),
            receipt_digest: computed,
        });
        let transition = self.append_transition(
            DeletionState::VERIFIED_ERASED,
            today,
            "DELETION_RECEIPT_VERIFIED",
        );
        self.seal();
        Ok(VerifiedErasure {
            transition,
            deadline_breach_days,
        })
    }

    /// Transitions `PENDING_RECEIPT` -> `STATUTORY_REFUSED` with a typed
    /// refusal. A `DeadlineExpired` refusal is admissible only once the
    /// applicable clock is actually breached on `today`; exception and
    /// verification refusals are admissible at any point in the window.
    ///
    /// # Errors
    /// [`DeletionError::RefusedIllegalTransition`] on a terminal state;
    /// [`DeletionError::RefusedPrematureDeadlineExpiry`] when
    /// `DeadlineExpired` is asserted before the clock is breached; typed
    /// `REFUSED_*` on an observation date predating receipt.
    pub fn statutory_refuse(
        &mut self,
        kind: RefusalKind,
        basis: &str,
        today: CivilDate,
    ) -> Result<TransitionRecord, DeletionError> {
        if self.state != DeletionState::PENDING_RECEIPT {
            return Err(DeletionError::RefusedIllegalTransition {
                from: self.state,
                action: "statutory_refuse",
            });
        }
        if today.days_since(self.received) < 0 {
            return Err(DeletionError::RefusedClockBeforeReceipt {
                received: self.received,
                today,
            });
        }
        if matches!(kind, RefusalKind::DeadlineExpired) {
            let breached = matches!(
                self.clock_phase(today)?,
                ClockPhase::StatutoryDeadlineBreached { .. }
            );
            if !breached {
                return Err(DeletionError::RefusedPrematureDeadlineExpiry {
                    today,
                    applicable_deadline: self.applicable_deadline(),
                });
            }
        }
        if basis.contains('\n') || basis.contains('\r') {
            return Err(DeletionError::RefusedEmbeddedNewline { field: "basis" });
        }
        self.refusal = Some(RefusalRecord {
            citation: kind.citation(),
            kind,
            basis: basis.to_string(),
            recorded_on: today,
        });
        let record = self.append_transition(
            DeletionState::STATUTORY_REFUSED,
            today,
            "STATUTORY_REFUSAL_RECORDED",
        );
        self.seal();
        Ok(record)
    }

    /// Whether the case reached a terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.state != DeletionState::PENDING_RECEIPT
    }

    /// Recomputes the BLAKE3 seal over the core view and compares it to
    /// the stored `state_digest`. A deserialized case with any edited
    /// field fails this check.
    #[must_use]
    pub fn integrity_ok(&self) -> bool {
        match self.core_digest() {
            Ok(digest) => digest == self.state_digest,
            Err(_) => false,
        }
    }

    /// Renders the formal CCPA/CPRA + SB 923 deletion request letter.
    /// Byte-identical for identical case content; zero-emotion formal
    /// prose only.
    #[must_use]
    pub fn render_deletion_request(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        let rule = "=".repeat(72);
        lines.push(rule.clone());
        lines.push("FORMAL REQUEST TO DELETE PERSONAL INFORMATION".to_string());
        lines.push(
            "Statutory basis: California Consumer Privacy Act, Cal. Civ. Code § 1798.105, \
             as amended by the California Privacy Rights Act and by"
                .to_string(),
        );
        lines.push(CITATION_SB923.to_string());
        lines.push(format!("Case identifier: {}", self.case_id));
        lines.push(rule);

        lines.push(String::new());
        lines.push("1. SUBMITTING PARTY".to_string());
        lines.push(format!("  1.1 Full legal name:  {}", self.consumer.full_legal_name));
        lines.push(format!("  1.2 Email address:    {}", self.consumer.email));
        lines.push(format!(
            "  1.3 Mailing address:  {}",
            self.consumer.mailing_address
        ));
        lines.push(format!(
            "  1.4 Telephone:        {}",
            self.consumer
                .phone
                .as_deref()
                .unwrap_or("not supplied")
        ));

        lines.push(String::new());
        lines.push("2. STATUTORY AUTHORITY".to_string());
        lines.push(format!("  2.1 {CITATION_DELETE}: the consumer requests deletion of the personal information described below."));
        lines.push(format!(
            "  2.2 {CITATION_SERVICE_PROVIDERS}: the business shall direct its service \
             providers, contractors, and applicable third parties to delete the same \
             personal information from their records."
        ));
        lines.push(format!(
            "  2.3 {CITATION_RESPONSE_CLOCK}: the business shall respond within \
             {PRIMARY_RESPONSE_DAYS} days of receipt; a single extension of up to 45 \
             additional days is permitted only where the business notifies the consumer \
             of the extension within the initial 45-day period (statutory maximum: \
             {EXTENDED_RESPONSE_DAYS} days)."
        ));
        lines.push(format!(
            "  2.4 {CITATION_EXCEPTIONS}: refusal is admissible only by enumerated \
             statutory exception, and each invoked exception requires a statement of \
             the statutory basis."
        ));
        lines.push(format!(
            "  2.5 {CITATION_NON_DISCRIMINATION}: the consumer may not be discriminated \
             against for exercising this right."
        ));
        lines.push(format!(
            "  2.6 {CITATION_SB923}: effective {} the § 1798.105 deletion right covers \
             personal information collected about the consumer from any source, \
             including information obtained from third parties.",
            SB923_EFFECTIVE.iso()
        ));

        lines.push(String::new());
        lines.push("3. REQUEST SCOPE".to_string());
        lines.push(format!("  {}", self.scope_sentence()));

        lines.push(String::new());
        lines.push("4. DATA CATEGORIES SUBJECT TO DELETION".to_string());
        for (index, category) in self.data_categories.iter().enumerate() {
            lines.push(format!("  4.{} {category}", index + 1));
        }

        lines.push(String::new());
        lines.push("5. SUBMISSION CHANNELS".to_string());
        for (index, channel) in self.channels.iter().enumerate() {
            lines.push(format!("  5.{} {}", index + 1, channel.label()));
        }

        lines.push(String::new());
        lines.push("6. DATES".to_string());
        lines.push(format!("  6.1 Request transmitted:  {}", self.request_sent.iso()));
        lines.push(format!("  6.2 Request received:     {}", self.received.iso()));
        lines.push(format!(
            "  6.3 Primary statutory deadline ({PRIMARY_RESPONSE_DAYS} days):  {}",
            self.primary_deadline.iso()
        ));
        lines.push(format!(
            "  6.4 Extended statutory deadline ({EXTENDED_RESPONSE_DAYS} days, if extension \
             noticed): {}",
            self.received.add_days(i64::from(EXTENDED_RESPONSE_DAYS)).iso()
        ));

        lines.push(String::new());
        lines.push("7. REQUIRED RESPONSE".to_string());
        lines.push(
            "  7.1 Deletion of the personal information described in section 4, including \
             deletion by service providers, contractors, and third parties."
                .to_string(),
        );
        lines.push(format!(
            "  7.2 If the business declines any portion of this request: the statutory \
             citation and basis for each {CITATION_EXCEPTIONS} exception invoked."
        ));
        lines.push(format!(
            "  7.3 If the business extends the response period: the extension notice \
             described in {CITATION_RESPONSE_CLOCK}, transmitted within the initial \
             {PRIMARY_RESPONSE_DAYS}-day period."
        ));
        lines.push(
            "  7.4 A written response referencing the case identifier above; the written \
             response constitutes the deletion receipt for the statutory clock tracked \
             under this case identifier."
                .to_string(),
        );

        lines.push(String::new());
        lines.push("8. VERIFICATION".to_string());
        lines.push(
            "  The consumer will complete reasonable identity verification as required by \
             § 1798.130 upon the business's request."
                .to_string(),
        );

        lines.push(String::new());
        lines.push(format!("Submitted by: {}", self.consumer.full_legal_name));
        let channel_list: Vec<&str> = self.channels.iter().map(|c| c.label()).collect();
        lines.push(format!("Channels of record: {}", channel_list.join(", ")));
        lines.join("\n")
    }

    /// Scope sentence for section 3, branching on the SB 923 expanded
    /// scope and its effective date.
    fn scope_sentence(&self) -> String {
        if !self.collected_about_scope {
            format!(
                "All personal information collected from the submitting party under \
                 {CITATION_DELETE}."
            )
        } else if self.received.days_since(SB923_EFFECTIVE) >= 0 {
            format!(
                "All personal information the business has collected about the submitting \
                 party, from any source, including information obtained from third \
                 parties, as authorized by {CITATION_SB923}."
            )
        } else {
            format!(
                "All personal information collected from the submitting party now, and, \
                 effective {} per {CITATION_SB923}, all personal information collected \
                 about the submitting party from any source.",
                SB923_EFFECTIVE.iso()
            )
        }
    }

    fn append_transition(
        &mut self,
        to: DeletionState,
        at: CivilDate,
        basis: &str,
    ) -> TransitionRecord {
        let record = TransitionRecord {
            from: self.state,
            to,
            at,
            basis: basis.to_string(),
        };
        self.state = to;
        self.history.push(record.clone());
        record
    }

    fn core_digest(&self) -> Result<String, DeletionError> {
        let core = CaseCore {
            case_id: &self.case_id,
            consumer: &self.consumer,
            data_categories: &self.data_categories,
            channels: &self.channels,
            collected_about_scope: &self.collected_about_scope,
            request_sent: &self.request_sent,
            received: &self.received,
            primary_deadline: &self.primary_deadline,
            extension: &self.extension,
            extended_deadline: &self.extended_deadline,
            state: &self.state,
            refusal: &self.refusal,
            deletion_receipt: &self.deletion_receipt,
            history: &self.history,
        };
        let json = canonical_json(&core)?;
        Ok(blake3_hex(json.as_bytes()))
    }

    fn seal(&mut self) {
        self.state_digest = self
            .core_digest()
            .expect("core digest is infallible over valid case state");
    }
}

/// Result of a successful `VERIFIED_ERASED` transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedErasure {
    /// The appended transition record.
    pub transition: TransitionRecord,
    /// Days the receipt arrived past the applicable statutory deadline
    /// (0 when the receipt landed inside the window). The clock history
    /// survives the erasure.
    pub deadline_breach_days: u32,
}

// ---------------------------------------------------------------------------
// Typed refusals
// ---------------------------------------------------------------------------

/// Typed refusal taxonomy. Every variant names the transition it refuses;
/// `Display` renders the machine-readable `REFUSED_*` code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionError {
    /// State does not admit the attempted transition.
    RefusedIllegalTransition {
        /// Current state.
        from: DeletionState,
        /// Attempted action name.
        action: &'static str,
    },
    /// Date construction or parse failed.
    RefusedInvalidDate {
        /// Failure reason.
        reason: String,
    },
    /// Computed digest of supplied text differs from the pre-committed
    /// digest.
    RefusedReceiptDigestMismatch {
        /// Pre-committed expected digest.
        expected: String,
        /// Computed digest.
        computed: String,
    },
    /// The single statutory extension was already recorded.
    RefusedDuplicateExtension,
    /// Extension notice transmitted after the primary window closed.
    RefusedExtensionNoticeOutsidePrimaryWindow {
        /// Notice date.
        notice_sent: CivilDate,
        /// Primary 45-day deadline.
        primary_deadline: CivilDate,
    },
    /// `DeadlineExpired` asserted while the clock was not breached.
    RefusedPrematureDeadlineExpiry {
        /// Evaluation date.
        today: CivilDate,
        /// Applicable (primary or extended) deadline.
        applicable_deadline: CivilDate,
    },
    /// Supplied digest does not have BLAKE3 hex shape.
    RefusedInvalidDigest {
        /// Failure reason.
        reason: &'static str,
    },
    /// Required field empty.
    RefusedEmptyField {
        /// Field name.
        field: &'static str,
    },
    /// Field carries a newline, breaking letter line-determinism.
    RefusedEmbeddedNewline {
        /// Field name.
        field: &'static str,
    },
    /// Observed date predates the clock origin.
    RefusedClockBeforeReceipt {
        /// Clock origin date.
        received: CivilDate,
        /// Offending observation date.
        today: CivilDate,
    },
    /// Clock evaluated on a terminal case.
    ClockClosedOnTerminal {
        /// Terminal state.
        state: DeletionState,
    },
    /// Canonical JSON serialization failed.
    RefusedCanonicalization {
        /// Underlying error text.
        source: String,
    },
}

impl fmt::Display for DeletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RefusedIllegalTransition { from, action } => write!(
                f,
                "REFUSED_ILLEGAL_TRANSITION: {action} not admissible in state {from:?}"
            ),
            Self::RefusedInvalidDate { reason } => {
                write!(f, "REFUSED_INVALID_DATE: {reason}")
            }
            Self::RefusedReceiptDigestMismatch { expected, computed } => write!(
                f,
                "REFUSED_RECEIPT_DIGEST_MISMATCH: expected {expected}, computed {computed}"
            ),
            Self::RefusedDuplicateExtension => write!(
                f,
                "REFUSED_DUPLICATE_EXTENSION: the single § 1798.130(a)(2) extension is already recorded"
            ),
            Self::RefusedExtensionNoticeOutsidePrimaryWindow {
                notice_sent,
                primary_deadline,
            } => write!(
                f,
                "REFUSED_EXTENSION_NOTICE_OUTSIDE_PRIMARY_WINDOW: notice {notice_sent} after primary deadline {primary_deadline}"
            ),
            Self::RefusedPrematureDeadlineExpiry {
                today,
                applicable_deadline,
            } => write!(
                f,
                "REFUSED_PREMATURE_DEADLINE_EXPIRY: clock not breached on {today} (applicable deadline {applicable_deadline})"
            ),
            Self::RefusedInvalidDigest { reason } => {
                write!(f, "REFUSED_INVALID_DIGEST: {reason}")
            }
            Self::RefusedEmptyField { field } => {
                write!(f, "REFUSED_EMPTY_FIELD: {field}")
            }
            Self::RefusedEmbeddedNewline { field } => {
                write!(f, "REFUSED_EMBEDDED_NEWLINE: {field}")
            }
            Self::RefusedClockBeforeReceipt { received, today } => write!(
                f,
                "REFUSED_CLOCK_BEFORE_RECEIPT: observation {today} predates received date {received}"
            ),
            Self::ClockClosedOnTerminal { state } => write!(
                f,
                "REFUSED_CLOCK_CLOSED_ON_TERMINAL: state {state:?}"
            ),
            Self::RefusedCanonicalization { source } => {
                write!(f, "REFUSED_CANONICALIZATION: {source}")
            }
        }
    }
}

impl std::error::Error for DeletionError {}

// ---------------------------------------------------------------------------
// Tests — pure, zero mocks, real assertions
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> ConsumerIdentity {
        ConsumerIdentity {
            full_legal_name: "Sean Chatman".to_string(),
            email: "sean@example.com".to_string(),
            mailing_address: "1 Market St, San Francisco, CA 94105".to_string(),
            phone: Some("+1-415-555-0100".to_string()),
        }
    }

    fn intake(sent: &str, received: &str, about_scope: bool) -> DeletionIntake {
        DeletionIntake {
            consumer: identity(),
            data_categories: vec![
                "purchase history".to_string(),
                "browsing history".to_string(),
            ],
            channels: vec![RequestChannel::Email, RequestChannel::WebForm],
            request_sent: CivilDate::parse(sent).expect("sent date"),
            received: CivilDate::parse(received).expect("received date"),
            intake_sequence: 1,
            collected_about_scope: about_scope,
        }
    }

    fn open_case(sent: &str, received: &str) -> DeletionCase {
        DeletionCase::open(&intake(sent, received, true)).expect("open case")
    }

    fn receipt_input(observed_on: &str, text: &str) -> ErasureReceiptInput {
        ErasureReceiptInput {
            receipt_text: text.to_string(),
            observed_on: CivilDate::parse(observed_on).expect("observed date"),
            method: "email".to_string(),
            erased_categories: vec![
                "purchase history".to_string(),
                "browsing history".to_string(),
            ],
            expected_digest: blake3_hex(text.as_bytes()),
        }
    }

    const RECEIPT_TEXT: &str =
        "Deletion receipt: all requested categories erased per case identifier.";

    // -- civil date math ----------------------------------------------------

    #[test]
    fn leap_year_and_month_lengths() {
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2023));
        assert_eq!(days_in_month(2024, 2), Some(29));
        assert_eq!(days_in_month(2023, 2), Some(28));
        assert_eq!(days_in_month(2026, 4), Some(30));
        assert_eq!(days_in_month(2026, 13), None);
    }

    #[test]
    fn date_construction_rejects_invalid_days() {
        assert!(CivilDate::new(2026, 2, 30).is_err());
        assert!(CivilDate::new(2024, 2, 29).is_ok());
        assert!(CivilDate::new(2026, 0, 1).is_err());
        assert!(CivilDate::new(2026, 1, 0).is_err());
        assert!(CivilDate::new(999, 1, 1).is_err());
        assert!(CivilDate::new(10_000, 1, 1).is_err());
    }

    #[test]
    fn add_days_crosses_month_and_year_and_leap() {
        assert_eq!(
            CivilDate::parse("2026-12-31").unwrap().add_days(1),
            CivilDate::new(2027, 1, 1).unwrap()
        );
        assert_eq!(
            CivilDate::parse("2024-02-28").unwrap().add_days(1),
            CivilDate::new(2024, 2, 29).unwrap()
        );
        assert_eq!(
            CivilDate::parse("2023-02-28").unwrap().add_days(1),
            CivilDate::new(2023, 3, 1).unwrap()
        );
        assert_eq!(
            CivilDate::parse("2026-11-20").unwrap().add_days(45),
            CivilDate::new(2027, 1, 4).unwrap()
        );
    }

    #[test]
    fn days_since_is_signed_and_inverse_of_add_days() {
        let a = CivilDate::parse("2026-11-20").unwrap();
        let b = a.add_days(45);
        assert_eq!(b.days_since(a), 45);
        assert_eq!(a.days_since(b), -45);
        assert_eq!(b.add_days(-45), a);
        assert_eq!(a.days_since(a), 0);
    }

    #[test]
    fn parse_format_round_trip_and_rejections() {
        let a = CivilDate::parse("2026-11-20").unwrap();
        assert_eq!(a.iso(), "2026-11-20");
        assert_eq!(CivilDate::parse(&a.iso()).unwrap(), a);
        for bad in [
            "2026-1-20", "2026-11-2", "26-11-20", "2026/11/20", "2026-13-01", "2026-00-10",
            "2026-11-00", "-026-11-20", "2026-02-30", "",
        ] {
            assert!(CivilDate::parse(bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn serde_date_is_iso_string() {
        let a = CivilDate::new(2026, 11, 20).unwrap();
        let json = serde_json::to_string(&a).unwrap();
        assert_eq!(json, "\"2026-11-20\"");
        assert_eq!(serde_json::from_str::<CivilDate>(&json).unwrap(), a);
        assert!(serde_json::from_str::<CivilDate>("\"2026-02-30\"").is_err());
    }

    #[test]
    fn statutory_constants() {
        assert_eq!(PRIMARY_RESPONSE_DAYS, 45);
        assert_eq!(EXTENDED_RESPONSE_DAYS, 90);
        assert_eq!(SB923_EFFECTIVE.iso(), "2027-01-01");
        assert!(CivilDate::new(
            SB923_EFFECTIVE.year,
            SB923_EFFECTIVE.month,
            SB923_EFFECTIVE.day
        )
        .is_ok());
    }

    // -- case opening -------------------------------------------------------

    #[test]
    fn open_case_derives_clocks_and_seal() {
        let case = open_case("2026-11-20", "2026-11-20");
        assert_eq!(case.state, DeletionState::PENDING_RECEIPT);
        assert_eq!(case.case_id.len(), 64);
        assert!(case.case_id.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(case.primary_deadline.iso(), "2027-01-04");
        assert_eq!(case.extended_deadline, None);
        assert_eq!(case.history.len(), 1);
        assert_eq!(case.history[0].basis, "INTAKE_OPENED");
        assert!(case.integrity_ok());
        assert!(!case.is_terminal());
    }

    #[test]
    fn open_case_rejects_bad_intake() {
        let mut bad = intake("2026-11-20", "2026-11-20", false);
        bad.consumer.full_legal_name = "  ".to_string();
        assert_eq!(
            DeletionCase::open(&bad),
            Err(DeletionError::RefusedEmptyField { field: "full_legal_name" })
        );

        let mut no_cats = intake("2026-11-20", "2026-11-20", false);
        no_cats.data_categories = Vec::new();
        assert!(matches!(
            DeletionCase::open(&no_cats),
            Err(DeletionError::RefusedEmptyField { field: "data_categories" })
        ));

        let mut newline = intake("2026-11-20", "2026-11-20", false);
        newline.consumer.email = "a@b.com\nBcc: x".to_string();
        assert_eq!(
            DeletionCase::open(&newline),
            Err(DeletionError::RefusedEmbeddedNewline { field: "email" })
        );

        let backwards = intake("2026-11-21", "2026-11-20", false);
        assert!(matches!(
            DeletionCase::open(&backwards),
            Err(DeletionError::RefusedClockBeforeReceipt { .. })
        ));
    }

    #[test]
    fn case_id_is_deterministic_and_sequence_sensitive() {
        let a = DeletionCase::open(&intake("2026-11-20", "2026-11-20", true)).unwrap();
        let b = DeletionCase::open(&intake("2026-11-20", "2026-11-20", true)).unwrap();
        assert_eq!(a.case_id, b.case_id);
        let mut c = intake("2026-11-20", "2026-11-20", true);
        c.intake_sequence = 2;
        let d = DeletionCase::open(&c).unwrap();
        assert_ne!(a.case_id, d.case_id);
    }

    // -- clock phases -------------------------------------------------------

    #[test]
    fn clock_phases_without_extension() {
        let case = open_case("2026-11-19", "2026-11-20");
        let at = |offset: i64| case.received.add_days(offset);
        assert_eq!(
            case.clock_phase(at(0)).unwrap(),
            ClockPhase::WithinPrimaryWindow { days_remaining: 45 }
        );
        assert_eq!(
            case.clock_phase(at(44)).unwrap(),
            ClockPhase::WithinPrimaryWindow { days_remaining: 1 }
        );
        assert_eq!(
            case.clock_phase(at(45)).unwrap(),
            ClockPhase::WithinPrimaryWindow { days_remaining: 0 }
        );
        // No extension noticed: past day 45 the case is breached, the
        // extension is not retroactively available.
        assert_eq!(
            case.clock_phase(at(46)).unwrap(),
            ClockPhase::StatutoryDeadlineBreached { days_over: 1 }
        );
        assert_eq!(
            case.clock_phase(at(60)).unwrap(),
            ClockPhase::StatutoryDeadlineBreached { days_over: 15 }
        );
    }

    #[test]
    fn clock_errors_before_receipt_and_on_terminal() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        assert!(matches!(
            case.clock_phase(case.received.add_days(-1)),
            Err(DeletionError::RefusedClockBeforeReceipt { .. })
        ));
        case.statutory_refuse(
            RefusalKind::VerificationFailed,
            "identity not verifiable",
            CivilDate::parse("2026-11-25").unwrap(),
        )
        .unwrap();
        assert!(matches!(
            case.clock_phase(case.received.add_days(1)),
            Err(DeletionError::ClockClosedOnTerminal { .. })
        ));
    }

    // -- extension ----------------------------------------------------------

    #[test]
    fn extension_within_primary_window_extends_clock() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let notice_date = case.received.add_days(40);
        let notice_text = format!("Extension notice for case {}", case.case_id);
        let expected = blake3_hex(notice_text.as_bytes());
        case.record_extension(notice_date, &notice_text, &expected, notice_date)
            .expect("extension");
        assert_eq!(case.extended_deadline.unwrap().iso(), "2027-02-18");
        assert_eq!(case.applicable_deadline().iso(), "2027-02-18");
        assert_eq!(case.history.len(), 2);
        assert_eq!(case.history[1].basis, "EXTENSION_NOTICED_WITHIN_PRIMARY_WINDOW");
        assert!(case.integrity_ok());

        // Extension window phase: day 46..=90.
        assert_eq!(
            case.clock_phase(case.received.add_days(46)).unwrap(),
            ClockPhase::ExtensionWindow {
                days_since_primary: 1,
                days_remaining_to_extended: 44,
            }
        );
        assert_eq!(
            case.clock_phase(case.received.add_days(90)).unwrap(),
            ClockPhase::ExtensionWindow {
                days_since_primary: 45,
                days_remaining_to_extended: 0,
            }
        );
        assert_eq!(
            case.clock_phase(case.received.add_days(91)).unwrap(),
            ClockPhase::StatutoryDeadlineBreached { days_over: 1 }
        );
    }

    #[test]
    fn extension_notices_on_primary_deadline_day_are_admissible() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let notice_date = case.primary_deadline;
        let notice_text = "extension notice";
        let expected = blake3_hex(notice_text.as_bytes());
        case
            .record_extension(notice_date, notice_text, &expected, notice_date)
            .expect("notice on deadline day is within the window");
        assert_eq!(case.extended_deadline.unwrap().iso(), "2027-02-18");
    }

    #[test]
    fn extension_refusals() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let notice_date = case.received.add_days(40);
        let text = "extension notice";
        let good = blake3_hex(text.as_bytes());

        // Duplicate extension refused.
        case
            .record_extension(notice_date, text, &good, notice_date)
            .unwrap();
        assert_eq!(
            case.record_extension(notice_date, text, &good, notice_date),
            Err(DeletionError::RefusedDuplicateExtension)
        );
        assert_eq!(case.history.len(), 2);

        // Notice after the primary window refused (fresh case).
        let mut late = open_case("2026-11-20", "2026-11-20");
        let late_notice = late.primary_deadline.add_days(1);
        assert_eq!(
            late.record_extension(late_notice, text, &good, late_notice),
            Err(DeletionError::RefusedExtensionNoticeOutsidePrimaryWindow {
                notice_sent: late_notice,
                primary_deadline: late.primary_deadline,
            })
        );

        // Digest mismatch refused; state and history unchanged.
        let mut tamper = open_case("2026-11-20", "2026-11-20");
        let forged = blake3_hex(b"forged notice");
        assert_eq!(
            tamper.record_extension(notice_date, "actual notice", &forged, notice_date),
            Err(DeletionError::RefusedReceiptDigestMismatch {
                expected: forged,
                computed: blake3_hex(b"actual notice"),
            })
        );
        assert_eq!(tamper.history.len(), 1);
        assert_eq!(tamper.extension, None);
    }

    // -- verified erasure ---------------------------------------------------

    #[test]
    fn verify_erased_happy_path_inside_window() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let observed = case.received.add_days(44);
        let input = receipt_input(&observed.iso(), RECEIPT_TEXT);
        let result = case.verify_erased(&input, observed).expect("verified");
        assert_eq!(result.transition.from, DeletionState::PENDING_RECEIPT);
        assert_eq!(result.transition.to, DeletionState::VERIFIED_ERASED);
        assert_eq!(result.transition.basis, "DELETION_RECEIPT_VERIFIED");
        assert_eq!(result.deadline_breach_days, 0);
        assert_eq!(case.state, DeletionState::VERIFIED_ERASED);
        let receipt = case.deletion_receipt.as_ref().unwrap();
        assert_eq!(receipt.receipt_digest, input.expected_digest);
        assert_eq!(receipt.observed_on, observed);
        assert_eq!(case.history.len(), 2);
        assert!(case.integrity_ok());
        assert!(case.is_terminal());
    }

    #[test]
    fn verify_erased_digest_mismatch_is_refused_and_case_untouched() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let observed = case.received.add_days(10);
        let mut input = receipt_input(&observed.iso(), RECEIPT_TEXT);
        input.expected_digest = blake3_hex(b"altered receipt text");
        assert_eq!(
            case.verify_erased(&input, observed),
            Err(DeletionError::RefusedReceiptDigestMismatch {
                expected: blake3_hex(b"altered receipt text"),
                computed: blake3_hex(RECEIPT_TEXT.as_bytes()),
            })
        );
        assert_eq!(case.state, DeletionState::PENDING_RECEIPT);
        assert_eq!(case.history.len(), 1);
        assert_eq!(case.deletion_receipt, None);
    }

    #[test]
    fn late_receipt_closes_case_and_records_breach() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let observed = case.received.add_days(60);
        let input = receipt_input(&observed.iso(), RECEIPT_TEXT);
        let result = case.verify_erased(&input, observed).expect("late but real");
        assert_eq!(result.deadline_breach_days, 15);
        assert_eq!(case.state, DeletionState::VERIFIED_ERASED);
    }

    #[test]
    fn verify_erased_illegal_transitions() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let observed = case.received.add_days(10);
        let input = receipt_input(&observed.iso(), RECEIPT_TEXT);
        case.verify_erased(&input, observed).unwrap();
        // Terminal: no second verify, no extension, no refusal.
        assert_eq!(
            case.verify_erased(&input, observed),
            Err(DeletionError::RefusedIllegalTransition {
                from: DeletionState::VERIFIED_ERASED,
                action: "verify_erased",
            })
        );
        assert_eq!(
            case.statutory_refuse(
                RefusalKind::DeadlineExpired,
                "late",
                case.received.add_days(100)
            ),
            Err(DeletionError::RefusedIllegalTransition {
                from: DeletionState::VERIFIED_ERASED,
                action: "statutory_refuse",
            })
        );
        let text = "extension notice";
        let digest = blake3_hex(text.as_bytes());
        assert_eq!(
            case.record_extension(case.primary_deadline, text, &digest, case.primary_deadline),
            Err(DeletionError::RefusedIllegalTransition {
                from: DeletionState::VERIFIED_ERASED,
                action: "record_extension",
            })
        );
    }

    #[test]
    fn verify_erased_receipt_before_received_is_refused() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let early = case.received.add_days(-1);
        let input = receipt_input(&early.iso(), RECEIPT_TEXT);
        assert!(matches!(
            case.verify_erased(&input, early),
            Err(DeletionError::RefusedClockBeforeReceipt { .. })
        ));
    }

    #[test]
    fn verify_erased_forged_digest_shape_refused() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let observed = case.received.add_days(10);
        let mut input = receipt_input(&observed.iso(), RECEIPT_TEXT);
        input.expected_digest = "deadbeef".to_string();
        assert_eq!(
            case.verify_erased(&input, observed),
            Err(DeletionError::RefusedInvalidDigest {
                reason: "expected_digest is not 64-character lowercase blake3 hex",
            })
        );
    }

    // -- statutory refusal --------------------------------------------------

    #[test]
    fn statutory_exception_refusal_closes_case_with_citation() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let day20 = case.received.add_days(20);
        let kind = RefusalKind::StatutoryException(StatutoryException::LegalObligation);
        let record = case
            .statutory_refuse(kind.clone(), "records retained under 26 U.S.C. § 6001", day20)
            .expect("exception refusal");
        assert_eq!(record.from, DeletionState::PENDING_RECEIPT);
        assert_eq!(record.to, DeletionState::STATUTORY_REFUSED);
        let refusal = case.refusal.as_ref().unwrap();
        assert_eq!(refusal.kind, kind);
        assert_eq!(
            refusal.citation,
            format!(
                "{CITATION_EXCEPTIONS} — legal obligation exception"
            )
        );
        assert_eq!(case.state, DeletionState::STATUTORY_REFUSED);
        assert_eq!(kind.citation(), refusal.kind.citation());
        assert_eq!(kind.kind_code(), "STATUTORY_EXCEPTION");
        assert!(case.integrity_ok());
    }

    #[test]
    fn exception_citations_are_enumerated_and_stable() {
        let kinds = [
            StatutoryException::TransactionCompletion,
            StatutoryException::SecurityIntegrity,
            StatutoryException::ErrorRepair,
            StatutoryException::FreeSpeechRights,
            StatutoryException::ScientificResearch,
            StatutoryException::LegalObligation,
            StatutoryException::CompatibleInternalUse,
        ];
        let citations: Vec<String> = kinds.iter().map(StatutoryException::citation).collect();
        assert_eq!(citations.len(), 7);
        for (index, citation) in citations.iter().enumerate() {
            assert!(citation.starts_with(CITATION_EXCEPTIONS), "{citation}");
            for (other_index, other) in citations.iter().enumerate() {
                if index != other_index {
                    assert_ne!(citation, other);
                }
            }
        }
        assert_eq!(
            RefusalKind::VerificationFailed.citation(),
            format!(
                "{CITATION_RESPONSE_CLOCK} — consumer identity not verifiable"
            )
        );
        assert_eq!(RefusalKind::VerificationFailed.kind_code(), "VERIFICATION_FAILED");
        assert_eq!(RefusalKind::DeadlineExpired.kind_code(), "DEADLINE_EXPIRED");
    }

    #[test]
    fn premature_deadline_expiry_refusal_is_refused() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let day45 = case.received.add_days(45);
        assert_eq!(
            case.statutory_refuse(RefusalKind::DeadlineExpired, "expired", day45),
            Err(DeletionError::RefusedPrematureDeadlineExpiry {
                today: day45,
                applicable_deadline: case.primary_deadline,
            })
        );
        assert_eq!(case.state, DeletionState::PENDING_RECEIPT);

        let mut extended = open_case("2026-11-20", "2026-11-20");
        let notice = extended.received.add_days(40);
        let text = "extension notice";
        let digest = blake3_hex(text.as_bytes());
        extended.record_extension(notice, text, &digest, notice).unwrap();
        // Day 50: past primary but inside the extended window — not yet
        // expired against the applicable deadline.
        let day50 = extended.received.add_days(50);
        assert!(matches!(
            extended.statutory_refuse(RefusalKind::DeadlineExpired, "expired", day50),
            Err(DeletionError::RefusedPrematureDeadlineExpiry { .. })
        ));
        // Day 91: applicable (90-day) clock breached — admissible.
        let day91 = extended.received.add_days(91);
        extended
            .statutory_refuse(RefusalKind::DeadlineExpired, "clock expired without receipt", day91)
            .expect("breached clock admits deadline refusal");
        assert_eq!(extended.state, DeletionState::STATUTORY_REFUSED);
    }

    #[test]
    fn deadline_refusal_without_extension_admissible_after_day_45() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let day46 = case.received.add_days(46);
        case
            .statutory_refuse(RefusalKind::DeadlineExpired, "no receipt by statutory deadline", day46)
            .expect("breached primary clock admits deadline refusal");
        let refusal = case.refusal.as_ref().unwrap();
        assert_eq!(refusal.kind.kind_code(), "DEADLINE_EXPIRED");
        assert!(refusal.citation.contains(CITATION_RESPONSE_CLOCK));
    }

    // -- JSON state machine -------------------------------------------------

    #[test]
    fn json_round_trip_preserves_case_and_digest() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let notice = case.received.add_days(30);
        let notice_text = "Extension notice: additional 45 days per § 1798.130(a)(2).";
        let notice_digest = blake3_hex(notice_text.as_bytes());
        case.record_extension(notice, notice_text, &notice_digest, notice)
            .unwrap();
        let observed = case.received.add_days(80);
        let input = receipt_input(&observed.iso(), RECEIPT_TEXT);
        case.verify_erased(&input, observed).unwrap();

        let json = serde_json::to_string(&case).unwrap();
        assert!(json.contains("\"state\":\"VERIFIED_ERASED\""));
        assert!(json.contains("\"PENDING_RECEIPT\"")); // history ledger
        let restored: DeletionCase = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, case);
        assert!(restored.integrity_ok());
        assert_eq!(restored.state_digest, case.state_digest);
    }

    #[test]
    fn tampered_case_json_fails_integrity_seal() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        let observed = case.received.add_days(10);
        let input = receipt_input(&observed.iso(), RECEIPT_TEXT);
        case.verify_erased(&input, observed).unwrap();
        let mut json = serde_json::to_string(&case).unwrap();
        let original_digest = case.state_digest.clone();
        // Flip one character of the stored seal.
        let needle = format!("\"state_digest\":\"{original_digest}\"");
        assert!(json.contains(&needle));
        let forged_digest = format!("{}{}", &original_digest[..63],
            if original_digest.ends_with('0') { "1" } else { "0" });
        json = json.replace(&needle, &format!("\"state_digest\":\"{forged_digest}\""));
        let tampered: DeletionCase = serde_json::from_str(&json).unwrap();
        assert_ne!(tampered.state_digest, original_digest);
        assert!(!tampered.integrity_ok());
    }

    #[test]
    fn state_names_serialize_as_screaming_snake() {
        assert_eq!(
            serde_json::to_string(&DeletionState::PENDING_RECEIPT).unwrap(),
            "\"PENDING_RECEIPT\""
        );
        assert_eq!(
            serde_json::to_string(&DeletionState::VERIFIED_ERASED).unwrap(),
            "\"VERIFIED_ERASED\""
        );
        assert_eq!(
            serde_json::to_string(&DeletionState::STATUTORY_REFUSED).unwrap(),
            "\"STATUTORY_REFUSED\""
        );
    }

    // -- request letter -----------------------------------------------------

    #[test]
    fn letter_is_byte_identical_across_renders_and_cases() {
        let a = open_case("2026-11-20", "2026-11-20");
        let b = open_case("2026-11-20", "2026-11-20");
        let letter_a = a.render_deletion_request();
        assert_eq!(letter_a, a.render_deletion_request());
        assert_eq!(letter_a, b.render_deletion_request());
    }

    #[test]
    fn letter_carries_statutory_citations_and_clocks() {
        let case = open_case("2026-11-20", "2026-11-20");
        let letter = case.render_deletion_request();
        for needle in [
            CITATION_DELETE,
            CITATION_RESPONSE_CLOCK,
            CITATION_SERVICE_PROVIDERS,
            CITATION_EXCEPTIONS,
            CITATION_NON_DISCRIMINATION,
            CITATION_SB923,
            "FORMAL REQUEST TO DELETE PERSONAL INFORMATION",
            "45 days",
            "90 days",
            "2027-01-04",  // primary deadline
            "2027-02-18", // extended deadline
            &case.case_id,
            "purchase history",
            "email",
            "web form",
        ] {
            assert!(letter.contains(needle), "letter missing {needle:?}");
        }
        // Zero-emotion discipline: no appeals or urgency adjectives.
        for banned in ["please", "Please", "urgent", "Urgent", "kindly", "sorry"] {
            assert!(!letter.to_lowercase().contains(&banned.to_lowercase()) || banned == "sorry",
                "letter contains emotional token {banned:?}");
        }
        assert!(!letter.contains("sorry"));
    }

    #[test]
    fn letter_branches_on_sb923_scope_and_effective_date() {
        // Pre-effective-date request asserting expanded scope: anticipatory
        // language.
        let pre = open_case("2026-11-20", "2026-11-20");
        let pre_letter = pre.render_deletion_request();
        assert!(pre_letter.contains("effective 2027-01-01 per"));
        assert!(pre_letter.contains("collected about the submitting party from any source"));

        // Post-effective-date request: expanded scope asserted directly.
        let post = open_case("2027-02-01", "2027-02-01");
        let post_letter = post.render_deletion_request();
        assert!(post_letter.contains("has collected about the submitting party"));
        assert!(!post_letter.contains("now, and, effective"));

        // Scope not asserted: statutory baseline only.
        let baseline = DeletionCase::open(&intake("2027-02-01", "2027-02-01", false)).unwrap();
        let baseline_letter = baseline.render_deletion_request();
        assert!(baseline_letter.contains("collected from the submitting party under"));
        assert!(!baseline_letter.contains("has collected about the submitting party"));
    }

    #[test]
    fn letter_without_phone_states_absence_formally() {
        let mut intake = intake("2026-11-20", "2026-11-20", false);
        intake.consumer.phone = None;
        let case = DeletionCase::open(&intake).unwrap();
        assert!(case.render_deletion_request().contains("not supplied"));
    }

    #[test]
    fn error_display_carries_typed_codes() {
        let e = DeletionError::RefusedIllegalTransition {
            from: DeletionState::VERIFIED_ERASED,
            action: "verify_erased",
        };
        assert!(e.to_string().starts_with("REFUSED_ILLEGAL_TRANSITION"));
        let e = DeletionError::RefusedReceiptDigestMismatch {
            expected: "a".repeat(64),
            computed: "b".repeat(64),
        };
        assert!(e.to_string().starts_with("REFUSED_RECEIPT_DIGEST_MISMATCH"));
        let e = DeletionError::RefusedPrematureDeadlineExpiry {
            today: CivilDate::new(2026, 11, 20).unwrap(),
            applicable_deadline: CivilDate::new(2027, 1, 4).unwrap(),
        };
        assert!(e.to_string().starts_with("REFUSED_PREMATURE_DEADLINE_EXPIRY"));
    }

    #[test]
    fn full_lifecycle_pending_to_refused_round_trips() {
        let mut case = open_case("2026-11-20", "2026-11-20");
        // No extension; day 46 breach; deadline refusal; JSON round trip.
        let day46 = case.received.add_days(46);
        case
            .statutory_refuse(RefusalKind::DeadlineExpired, "no deletion receipt received", day46)
            .unwrap();
        let json = serde_json::to_string(&case).unwrap();
        assert!(json.contains("\"state\":\"STATUTORY_REFUSED\""));
        let restored: DeletionCase = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, case);
        assert!(restored.integrity_ok());
        assert_eq!(restored.history.len(), 2);
        assert_eq!(restored.history[1].basis, "STATUTORY_REFUSAL_RECORDED");
    }
}
