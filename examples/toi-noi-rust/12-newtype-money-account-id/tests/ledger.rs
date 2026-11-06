//! Integration test: chỉ dùng API public của crate `ledger_lab`.

use std::collections::{BTreeSet, HashMap};

use ledger_lab::{
    AccountId, EntryId, IdError, JournalEntry, Ledger, LedgerError, Money, MoneyError, Posting,
};

fn acct(raw: &str) -> AccountId {
    raw.parse().expect("mã tài khoản hợp lệ")
}

fn entry(key: &str, lines: &[(&str, i64)]) -> Result<JournalEntry, LedgerError> {
    let postings = lines
        .iter()
        .map(|(raw, dong)| Posting::new(acct(raw), Money::from_dong(*dong)))
        .collect();
    JournalEntry::try_new(key, postings)
}

#[test]
fn capture_entry_gets_je_000001() -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();
    let posted = ledger.post(entry(
        "trip:TN-0001:capture",
        &[
            ("customer:CU-001", -25_500),
            ("platform:revenue", 5_100),
            ("driver:DR-001", 20_400),
        ],
    )?)?;
    assert_eq!(posted.id.to_string(), "JE-000001");
    let id: EntryId = "JE-000001".parse()?;
    assert_eq!(posted.id, id);
    Ok(())
}

#[test]
fn question_mark_converts_id_error_into_ledger_error() {
    fn parse_posting(raw: &str) -> Result<Posting, LedgerError> {
        // `?` dùng `From<IdError> for LedgerError`.
        let account: AccountId = raw.parse()?;
        Ok(Posting::new(account, Money::from_dong(1)))
    }
    assert_eq!(
        parse_posting("khong hop le").unwrap_err(),
        LedgerError::InvalidAccountId(IdError::InvalidChar { ch: ' ' })
    );
}

#[test]
fn money_overflow_becomes_ledger_overflow() {
    let result = entry("k", &[("test:a", i64::MAX), ("test:b", 1)]);
    assert_eq!(result, Err(LedgerError::Overflow));
    assert_eq!(
        Money::from_dong(i64::MAX) + Money::from_dong(1),
        Err(MoneyError::Overflow)
    );
}

#[test]
fn ids_work_as_hash_and_btree_keys() {
    let mut balances: HashMap<AccountId, Money> = HashMap::new();
    balances.insert(acct("driver:DR-001"), Money::from_dong(20_400));
    assert_eq!(
        balances.get(&acct("driver:DR-001")),
        Some(&Money::from_dong(20_400))
    );

    let ids: BTreeSet<EntryId> = [3_u64, 1, 2]
        .into_iter()
        .map(|n| EntryId::try_from(n).expect("số thứ tự hợp lệ"))
        .collect();
    let printed: Vec<String> = ids.iter().map(ToString::to_string).collect();
    assert_eq!(printed, ["JE-000001", "JE-000002", "JE-000003"]);
}
