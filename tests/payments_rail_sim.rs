use castle::payments::money::Currency;
use castle::payments::rail::{RailAck, RailActuator, RailError, RailInstruction, RailStatus};
use castle::payments::rail_sim::{SimMode, SimRail};

fn dir(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("castle-rail-{tag}-{}-{}", std::process::id(), std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos())));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn instr(effect: &str, payload: &str, amount: u64, payee: &str) -> RailInstruction {
    RailInstruction {
        effect_id: effect.into(),
        correlation_id: RailInstruction::correlation_id_for(effect),
        message_profile: "pain.001.001.09".into(),
        payload: payload.into(),
        amount_minor: amount,
        currency: Currency::USD,
        payer: "payer-1".into(),
        payee: payee.into(),
    }
}

const EFF: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn correlation_id_shape() {
    let c = RailInstruction::correlation_id_for(EFF);
    assert_eq!(c, "E2E-0123456789abcdef0123456789ab");
    assert!(instr(EFF, "<x/>", 1, "p").payload_digest().starts_with("sha256:"));
}

#[test]
fn accept_then_drop_ack_then_resubmit_no_second_settlement() {
    let rail = SimRail::open(&dir("drop"), SimMode::DropAckAfterAccept).unwrap();
    let i = instr(EFF, "<a/>", 500, "payee-1");
    assert_eq!(rail.submit(&i), Err(RailError::Timeout));
    assert_eq!(rail.status(&i.correlation_id), Ok(RailStatus::Accepted));
    assert_eq!(rail.submit(&i), Err(RailError::Timeout));
    // poll until settled
    let mut last = RailStatus::Unknown;
    for _ in 0..4 { last = rail.status(&i.correlation_id).unwrap(); }
    assert!(matches!(last, RailStatus::Settled { .. }));
    assert_eq!(rail.submissions_seen(&i.correlation_id), 2);
    assert_eq!(rail.settlement_count(&i.correlation_id), 1);
    assert_eq!(rail.accepted_correlations(), vec![i.correlation_id.clone()]);
}

#[test]
fn state_survives_reopen() {
    let d = dir("reopen");
    let i = instr(EFF, "<a/>", 500, "payee-1");
    {
        let rail = SimRail::open(&d, SimMode::SettleAfterPolls(1)).unwrap();
        assert_eq!(rail.submit(&i), Ok(RailAck::Accepted { correlation_id: i.correlation_id.clone() }));
        assert_eq!(rail.status(&i.correlation_id), Ok(RailStatus::Accepted));
    }
    let rail = SimRail::open(&d, SimMode::SettleAfterPolls(1)).unwrap();
    assert_eq!(rail.submissions_seen(&i.correlation_id), 1);
    assert!(matches!(rail.status(&i.correlation_id), Ok(RailStatus::Settled { .. })));
    assert_eq!(rail.settlement_count(&i.correlation_id), 1);
    // idempotent resubmit after restart
    assert!(matches!(rail.submit(&i), Ok(RailAck::Accepted { .. })));
    assert_eq!(rail.settlement_count(&i.correlation_id), 1);
}

#[test]
fn different_payload_duplicate_rejected() {
    let rail = SimRail::open(&dir("dup"), SimMode::Honest).unwrap();
    let a = instr(EFF, "<a/>", 500, "payee-1");
    assert!(matches!(rail.submit(&a), Ok(RailAck::Accepted { .. })));
    let b = instr(EFF, "<b/>", 500, "payee-1");
    let c = instr(EFF, "<a/>", 999, "payee-1");
    let d = instr(EFF, "<a/>", 500, "payee-2");
    for x in [b, c, d] {
        assert_eq!(rail.submit(&x), Ok(RailAck::Rejected {
            correlation_id: x.correlation_id.clone(),
            reason_code: "DUPLICATE_CORRELATION_DIFFERENT_PAYLOAD".into(),
        }));
    }
    assert_eq!(rail.settlement_count(&a.correlation_id), 0);
    assert_eq!(rail.submissions_seen(&a.correlation_id), 4);
}

#[test]
fn reject_after_delay() {
    let rail = SimRail::open(&dir("rej"), SimMode::RejectAfterPolls(2)).unwrap();
    let i = instr(EFF, "<a/>", 5, "p");
    rail.submit(&i).unwrap();
    assert_eq!(rail.status(&i.correlation_id), Ok(RailStatus::Accepted));
    assert_eq!(rail.status(&i.correlation_id), Ok(RailStatus::Accepted));
    let r = rail.status(&i.correlation_id).unwrap();
    assert!(matches!(r, RailStatus::Rejected { .. }));
    assert_eq!(rail.status(&i.correlation_id), Ok(r));
    assert_eq!(rail.settlement_count(&i.correlation_id), 0);
}

#[test]
fn settle_after_polls() {
    let rail = SimRail::open(&dir("settle"), SimMode::SettleAfterPolls(3)).unwrap();
    let i = instr(EFF, "<a/>", 5, "p");
    rail.submit(&i).unwrap();
    for _ in 0..3 { assert_eq!(rail.status(&i.correlation_id), Ok(RailStatus::Accepted)); }
    let s = rail.status(&i.correlation_id).unwrap();
    assert!(matches!(&s, RailStatus::Settled { final_ref } if !final_ref.is_empty()));
    assert_eq!(rail.settlement_count(&i.correlation_id), 1);
}

#[test]
fn return_after_settle_history() {
    let rail = SimRail::open(&dir("ret"), SimMode::ReturnAfterSettle).unwrap();
    let i = instr(EFF, "<a/>", 5, "p");
    rail.submit(&i).unwrap();
    for _ in 0..5 { rail.status(&i.correlation_id).unwrap(); }
    let h = rail.status_history(&i.correlation_id);
    assert_eq!(h.len(), 3);
    assert_eq!(h[0], RailStatus::Accepted);
    assert!(matches!(h[1], RailStatus::Settled { .. }));
    assert!(matches!(&h[2], RailStatus::Returned { return_ref, .. } if !return_ref.is_empty()));
    assert_eq!(rail.settlement_count(&i.correlation_id), 1);
}

#[test]
fn down_is_unavailable_and_recovers() {
    let rail = SimRail::open(&dir("down"), SimMode::Down).unwrap();
    let i = instr(EFF, "<a/>", 5, "p");
    assert!(matches!(rail.submit(&i), Err(RailError::Unavailable(_))));
    assert!(matches!(rail.status(&i.correlation_id), Err(RailError::Unavailable(_))));
    assert_eq!(rail.submissions_seen(&i.correlation_id), 0);
    rail.set_mode(SimMode::Honest).unwrap();
    assert!(matches!(rail.submit(&i), Ok(RailAck::Accepted { .. })));
}

#[test]
fn duplicate_reports_identical() {
    let rail = SimRail::open(&dir("dupr"), SimMode::DuplicateReports).unwrap();
    let i = instr(EFF, "<a/>", 5, "p");
    rail.submit(&i).unwrap();
    rail.status(&i.correlation_id).unwrap();
    let first = rail.status(&i.correlation_id).unwrap();
    assert!(matches!(first, RailStatus::Settled { .. }));
    for _ in 0..5 { assert_eq!(rail.status(&i.correlation_id).unwrap(), first); }
    assert_eq!(rail.status_history(&i.correlation_id), vec![RailStatus::Accepted, first]);
    assert_eq!(rail.settlement_count(&i.correlation_id), 1);
}

#[test]
fn unknown_correlation() {
    let rail = SimRail::open(&dir("unk"), SimMode::Honest).unwrap();
    assert_eq!(rail.status("E2E-nope"), Ok(RailStatus::Unknown));
    assert_eq!(rail.settlement_count("E2E-nope"), 0);
    assert_eq!(rail.submissions_seen("E2E-nope"), 0);
    assert!(rail.accepted_correlations().is_empty());
    assert!(rail.status_history("E2E-nope").is_empty());
}
