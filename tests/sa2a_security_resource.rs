use castle::sa2a_security::{BudgetLedger, ResourceEnvelope, SecurityRefusal};

#[test]
fn recursive_budget_cannot_amplify() {
    let mut ledger = BudgetLedger::new(ResourceEnvelope { compute_units: 10, io_bytes: 100, effects: 2 }).unwrap();
    ledger.allocate(ResourceEnvelope { compute_units: 6, io_bytes: 40, effects: 1 }).unwrap();
    assert_eq!(
        ledger.allocate(ResourceEnvelope { compute_units: 5, io_bytes: 20, effects: 1 }),
        Err(SecurityRefusal::ResourceAmplification)
    );
}
