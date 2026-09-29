use serde::{Deserialize, Serialize};

use super::refusal::{self, refuse, PayResult};

/// Closed currency set. No FX in v1: one currency per transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    KWD,
}

impl Currency {
    #[must_use]
    pub const fn exponent(self) -> u32 {
        match self {
            Self::USD | Self::EUR | Self::GBP => 2,
            Self::JPY => 0,
            Self::KWD => 3,
        }
    }

    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::USD => "USD",
            Self::EUR => "EUR",
            Self::GBP => "GBP",
            Self::JPY => "JPY",
            Self::KWD => "KWD",
        }
    }
}

/// Exact money: unsigned integer minor units. No floats anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money {
    pub minor: u64,
    pub currency: Currency,
}

impl Money {
    /// Parse a canonical base-10 minor-unit string: digits only, no sign, no
    /// leading zeros, no separators, fits `u64`, non-zero.
    pub fn parse_minor(text: &str, currency: Currency) -> PayResult<Self> {
        let canonical = !text.is_empty()
            && text.bytes().all(|b| b.is_ascii_digit())
            && (text == "0" || !text.starts_with('0'));
        if !canonical {
            return refuse(refusal::AMOUNT_NOT_INTEGER);
        }
        let minor: u64 = text.parse().map_err(|_| refusal::AMOUNT_NOT_INTEGER.to_string())?;
        if minor == 0 {
            return refuse(refusal::AMOUNT_ZERO);
        }
        Ok(Self { minor, currency })
    }

    /// Human decimal rendering using the currency exponent (ISO 20022 `InstdAmt`).
    #[must_use]
    pub fn to_decimal_string(self) -> String {
        let exp = self.currency.exponent() as usize;
        if exp == 0 {
            return self.minor.to_string();
        }
        let raw = format!("{:0>width$}", self.minor, width = exp + 1);
        let (whole, frac) = raw.split_at(raw.len() - exp);
        format!("{whole}.{frac}")
    }
}
