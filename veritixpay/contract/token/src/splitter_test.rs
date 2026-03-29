use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

use crate::balance::read_balance;
use crate::contract::VeritixToken;
use crate::splitter::{create_split, distribute, get_split, SplitRecipient};

fn setup_env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e
}

fn make_recipients(e: &Env, shares: &[(Address, u32)]) -> Vec<SplitRecipient> {
    let mut v = Vec::new(e);
    for (addr, bps) in shares {
        v.push_back(SplitRecipient {
            address: addr.clone(),
            share_bps: *bps,
        });
    }
    v
}

#[test]
fn test_create_split_stores_record() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);
    let r2 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 5000), (r2.clone(), 5000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);
        let record = get_split(&e, split_id);
        assert_eq!(record.sender, sender);
        assert_eq!(record.total_amount, 1000);
        assert!(!record.distributed);
    });
}

#[test]
fn test_distribute_two_recipients_equal_split() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);
    let r2 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 5000), (r2.clone(), 5000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);
        distribute(&e, sender.clone(), split_id);
        assert_eq!(read_balance(&e, r1.clone()), 500);
        assert_eq!(read_balance(&e, r2.clone()), 500);
        assert!(get_split(&e, split_id).distributed);
    });
}

#[test]
fn test_distribute_rounding_dust_goes_to_last_recipient() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);
    let r2 = Address::generate(&e);
    let r3 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 10);
        // 3333 + 3333 + 3334 = 10000 bps; 10 units → 3 + 3 + 4
        let recipients = make_recipients(
            &e,
            &[(r1.clone(), 3333), (r2.clone(), 3333), (r3.clone(), 3334)],
        );
        let split_id = create_split(&e, sender.clone(), recipients, 10);
        distribute(&e, sender.clone(), split_id);
        assert_eq!(read_balance(&e, r1.clone()), 3);
        assert_eq!(read_balance(&e, r2.clone()), 3);
        assert_eq!(read_balance(&e, r3.clone()), 4);
    });
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_distribute_unauthorized_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let hacker = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 10000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);
        distribute(&e, hacker.clone(), split_id);
    });
}

#[test]
#[should_panic(expected = "already distributed")]
fn test_double_distribute_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 10000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);
        distribute(&e, sender.clone(), split_id);
        distribute(&e, sender.clone(), split_id);
    });
}

#[test]
#[should_panic(expected = "recipients list cannot be empty")]
fn test_create_split_rejects_empty_recipients() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients: Vec<SplitRecipient> = Vec::new(&e);
        create_split(&e, sender.clone(), recipients, 1000);
    });
}

#[test]
#[should_panic(expected = "recipient share_bps cannot be zero")]
fn test_create_split_rejects_zero_share_recipient() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);
    let r2 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 10000), (r2.clone(), 0)]);
        create_split(&e, sender.clone(), recipients, 1000);
    });
}

#[test]
#[should_panic(expected = "duplicate recipient address")]
fn test_create_split_rejects_duplicate_recipients() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 5000), (r1.clone(), 5000)]);
        create_split(&e, sender.clone(), recipients, 1000);
    });
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_create_split_rejects_non_positive_amount() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        let recipients = make_recipients(&e, &[(r1.clone(), 10000)]);
        create_split(&e, sender.clone(), recipients, 0);
    });
}
