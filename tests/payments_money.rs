mod common;

use castle::payments::*;
use proptest::prelude::*;

const NOT_INT: &str = "REFUSED:PAYMENT_AMOUNT_NOT_INTEGER";
const ZERO: &str = "REFUSED:PAYMENT_AMOUNT_ZERO";

#[test]
fn parse_minor_rejects_every_non_canonical_form_with_exact_refusal() {
    for bad in ["", "01", "-5", "+5", "1.5", "1e3", " 5", "5 ", "0x10", "1_000", "٥", "18446744073709551616"] {
        assert_eq!(Money::parse_minor(bad, Currency::USD).unwrap_err(), NOT_INT, "input {bad:?}");
    }
}

#[test]
fn parse_minor_rejects_zero_distinctly() {
    assert_eq!(Money::parse_minor("0", Currency::USD).unwrap_err(), ZERO);
    assert_eq!(Money::parse_minor("00", Currency::USD).unwrap_err(), NOT_INT);
}

#[test]
fn parse_minor_accepts_u64_max_and_one() {
    let m = Money::parse_minor("18446744073709551615", Currency::JPY).unwrap();
    assert_eq!((m.minor, m.currency), (u64::MAX, Currency::JPY));
    assert_eq!(Money::parse_minor("1", Currency::EUR).unwrap().minor, 1);
}

#[test]
fn decimal_rendering_uses_currency_exponent() {
    let d = |minor, currency| Money { minor, currency }.to_decimal_string();
    assert_eq!(d(470_000, Currency::USD), "4700.00");
    assert_eq!(d(5, Currency::USD), "0.05");
    assert_eq!(d(5, Currency::JPY), "5");
    assert_eq!(d(5, Currency::KWD), "0.005");
    assert_eq!(d(100, Currency::EUR), "1.00");
    assert_eq!(d(u64::MAX, Currency::GBP), "184467440737095516.15");
}

fn any_currency() -> impl Strategy<Value = Currency> {
    prop_oneof![
        Just(Currency::USD),
        Just(Currency::EUR),
        Just(Currency::GBP),
        Just(Currency::JPY),
        Just(Currency::KWD)
    ]
}

proptest! {
    #[test]
    fn decimal_string_roundtrips_to_minor(minor in 1u64..=u64::MAX, currency in any_currency()) {
        let s = Money { minor, currency }.to_decimal_string();
        let exp = currency.exponent() as usize;
        if exp == 0 {
            prop_assert!(!s.contains('.'));
        } else {
            let dot = s.find('.').expect("has decimal point");
            prop_assert_eq!(s.len() - dot - 1, exp);
        }
        let stripped: String = s.chars().filter(|c| *c != '.').collect();
        prop_assert_eq!(stripped.parse::<u64>().unwrap(), minor);
    }

    #[test]
    fn canonical_minor_string_roundtrips_through_parse(minor in 1u64..=u64::MAX, currency in any_currency()) {
        let m = Money::parse_minor(&minor.to_string(), currency).unwrap();
        prop_assert_eq!(m, Money { minor, currency });
    }

    #[test]
    fn non_digit_strings_never_parse(s in "[^0-9]{1,8}[0-9]{0,4}") {
        prop_assert_eq!(Money::parse_minor(&s, Currency::USD).unwrap_err(), NOT_INT);
    }
}
