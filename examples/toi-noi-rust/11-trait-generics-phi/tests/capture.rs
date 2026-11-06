//! Integration test: chỉ dùng API pub, và tự định nghĩa chính sách phí ngoài thư viện.

use ledger_lab::{
    AccountId, FeePolicy, FlatFee, Ledger, LedgerError, Money, TripCharge, capture_entry,
    default_policy, fare_for_km,
};

fn trip() -> TripCharge {
    TripCharge {
        trip_id: "TN-0001".to_string(),
        customer: AccountId::new("customer:CU-001").unwrap(),
        driver: AccountId::new("driver:DR-001").unwrap(),
        fare: fare_for_km(5),
    }
}

/// Chính sách sai hợp đồng: phí lớn hơn cước.
struct GreedyFee;

impl FeePolicy for GreedyFee {
    fn name(&self) -> String {
        "tham".to_string()
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        fare.checked_add(Money::from_dong(1))
            .ok_or(LedgerError::Overflow)
    }
}

#[test]
fn greedy_policy_is_rejected_before_posting() {
    let err = capture_entry(&trip(), &GreedyFee).unwrap_err();
    assert_eq!(
        err,
        LedgerError::InvalidFee {
            fee: Money::from_dong(25_501),
            fare: Money::from_dong(25_500)
        }
    );
}

#[test]
fn swapping_policy_keeps_the_ledger_balanced() -> Result<(), LedgerError> {
    let policies: Vec<Box<dyn FeePolicy>> = vec![
        Box::new(default_policy()),
        Box::new(FlatFee::new(Money::from_dong(3_000))),
    ];
    for policy in &policies {
        let mut ledger = Ledger::new();
        ledger.post(capture_entry(&trip(), policy.as_ref())?)?;
        assert_eq!(ledger.grand_total()?, Money::ZERO);
    }
    Ok(())
}

#[test]
fn driver_net_follows_the_policy() -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();
    ledger.post(capture_entry(
        &trip(),
        &FlatFee::new(Money::from_dong(3_000)),
    )?)?;
    assert_eq!(
        ledger.balance(&AccountId::new("driver:DR-001")?)?,
        Money::from_dong(22_500)
    );
    Ok(())
}
