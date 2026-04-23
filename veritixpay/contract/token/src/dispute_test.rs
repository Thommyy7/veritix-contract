use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::balance::read_balance;
use crate::contract::VeritixToken;
use crate::dispute::{get_dispute, open_dispute, resolve_dispute, DisputeStatus};
use crate::escrow::{create_escrow, get_escrow};

fn setup_env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e
}

fn setup_escrow(e: &Env, contract_id: &Address) -> (Address, Address, u32) {
    let depositor = Address::generate(e);
    let beneficiary = Address::generate(e);
    let amount = 1_000i128;
    let mut escrow_id = 0u32;
    e.as_contract(contract_id, || {
        crate::balance::receive_balance(e, depositor.clone(), amount);
        escrow_id = create_escrow(e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });
    (depositor, beneficiary, escrow_id)
}

#[test]
fn test_open_dispute_stores_record() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone());
        let record = get_dispute(&e, dispute_id);
        assert_eq!(record.escrow_id, escrow_id);
        assert_eq!(record.claimant, depositor);
        assert_eq!(record.resolver, resolver);
        assert_eq!(record.status, DisputeStatus::Open);
    });
}

#[test]
fn test_resolve_dispute_for_beneficiary() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (_depositor, beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id =
            open_dispute(&e, beneficiary.clone(), escrow_id, resolver.clone());
        resolve_dispute(&e, resolver.clone(), dispute_id, true);

        let record = get_dispute(&e, dispute_id);
        assert_eq!(record.status, DisputeStatus::ResolvedForBeneficiary);

        let escrow = get_escrow(&e, escrow_id);
        assert!(escrow.released);

        assert_eq!(read_balance(&e, beneficiary.clone()), 1_000);
    });
}

#[test]
fn test_resolve_dispute_for_depositor() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id =
            open_dispute(&e, depositor.clone(), escrow_id, resolver.clone());
        resolve_dispute(&e, resolver.clone(), dispute_id, false);

        let record = get_dispute(&e, dispute_id);
        assert_eq!(record.status, DisputeStatus::ResolvedForDepositor);

        let escrow = get_escrow(&e, escrow_id);
        assert!(escrow.refunded);

        assert_eq!(read_balance(&e, depositor.clone()), 1_000);
    });
}

#[test]
#[should_panic(expected = "UnauthorizedResolver")]
fn test_resolve_dispute_wrong_resolver_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let impostor = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone());
        resolve_dispute(&e, impostor.clone(), dispute_id, true);
    });
}

#[test]
#[should_panic(expected = "AlreadyResolved")]
fn test_double_resolve_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone());
        resolve_dispute(&e, resolver.clone(), dispute_id, true);
        resolve_dispute(&e, resolver.clone(), dispute_id, false);
    });
}

#[test]
#[should_panic(expected = "InvalidState")]
fn test_open_dispute_on_settled_escrow_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (_depositor, beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        crate::escrow::release_escrow(&e, beneficiary.clone(), escrow_id);
        open_dispute(&e, beneficiary.clone(), escrow_id, resolver.clone());
    });
}

#[test]
#[should_panic(expected = "DisputeAlreadyOpen")]
fn test_duplicate_open_dispute_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        open_dispute(&e, depositor.clone(), escrow_id, resolver.clone());
        // Second open on the same unresolved escrow must fail.
        open_dispute(&e, depositor.clone(), escrow_id, resolver.clone());
    });
}

#[test]
fn test_reopen_dispute_after_resolution() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    // Create a second escrow to reopen on (the first is settled after resolution).
    let (depositor2, _beneficiary2, escrow_id2) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone());
        resolve_dispute(&e, resolver.clone(), dispute_id, false);

        // After resolution the EscrowDispute pointer is cleared; a new dispute on a
        // different (still-open) escrow must succeed.
        let new_id = open_dispute(&e, depositor2.clone(), escrow_id2, resolver.clone());
        let record = get_dispute(&e, new_id);
        assert_eq!(record.status, DisputeStatus::Open);
    });
}
