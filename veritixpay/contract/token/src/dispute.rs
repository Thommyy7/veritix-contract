use crate::balance::{receive_balance, spend_balance};
use crate::escrow::get_escrow;
use crate::storage_types::{increment_counter, write_persistent_record, DataKey};
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Open,
    ResolvedForBeneficiary,
    ResolvedForDepositor,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRecord {
    pub id: u32,
    pub escrow_id: u32,
    pub claimant: Address,
    pub resolver: Address,
    pub status: DisputeStatus,
}

/// Opens a dispute against an existing escrow.
pub fn open_dispute(
    e: &Env,
    claimant: Address,
    escrow_id: u32,
    resolver: Address,
) -> u32 {
    claimant.require_auth();

    let escrow = get_escrow(e, escrow_id);

    if escrow.released || escrow.refunded {
        panic!("InvalidState: Cannot open dispute on a settled escrow");
    }

    if claimant != escrow.depositor && claimant != escrow.beneficiary {
        panic!("Unauthorized: Only depositor or beneficiary can open a dispute");
    }

    // Prevent multiple open disputes for the same escrow.
    // NOTE: All validation must complete before incrementing the counter so that
    // rejected calls do not consume a dispute ID and leave gaps in the sequence.
    if e.storage()
        .persistent()
        .has(&DataKey::EscrowDispute(escrow_id))
    {
        panic!("DisputeAlreadyOpen: An open dispute already exists for this escrow");
    }

    // Increment only after all validation passes — counter must not advance on rejected calls.
    let count = increment_counter(e, &DataKey::DisputeCount);

    let record = DisputeRecord {
        id: count,
        escrow_id,
        claimant: claimant.clone(),
        resolver,
        status: DisputeStatus::Open,
    };

    e.storage().persistent().set(&DataKey::Dispute(count), &record);
    e.storage()
        .persistent()
        .set(&DataKey::EscrowDispute(escrow_id), &count);

    e.events().publish(
        (symbol_short!("dispute_opened"), escrow_id, claimant.clone()),
        (),
    );

    count
}

/// Private helper: settle an escrow by outcome without requiring depositor/beneficiary auth.
/// The resolver has already been authenticated by `resolve_dispute`.
fn settle_escrow_by_outcome(e: &Env, escrow_id: u32, release_to_beneficiary: bool) {
    let mut escrow = get_escrow(e, escrow_id);

    if escrow.released || escrow.refunded {
        panic!("AlreadySettled: escrow is already settled");
    }

    if release_to_beneficiary {
        escrow.released = true;
        write_persistent_record(e, &DataKey::Escrow(escrow_id), &escrow);
        spend_balance(e, e.current_contract_address(), escrow.amount);
        receive_balance(e, escrow.beneficiary.clone(), escrow.amount);
        e.events().publish(
            (symbol_short!("escrow_released"), escrow_id, escrow.beneficiary.clone()),
            escrow.amount,
        );
    } else {
        escrow.refunded = true;
        write_persistent_record(e, &DataKey::Escrow(escrow_id), &escrow);
        spend_balance(e, e.current_contract_address(), escrow.amount);
        receive_balance(e, escrow.depositor.clone(), escrow.amount);
        e.events().publish(
            (symbol_short!("escrow_refunded"), escrow_id, escrow.depositor.clone()),
            escrow.amount,
        );
    }
}

/// Resolves an open dispute. Only the designated resolver can call this.
/// Settlement does not require beneficiary/depositor auth.
pub fn resolve_dispute(
    e: &Env,
    resolver: Address,
    dispute_id: u32,
    release_to_beneficiary: bool,
) {
    resolver.require_auth();

    let mut dispute: DisputeRecord = e
        .storage()
        .persistent()
        .get(&DataKey::Dispute(dispute_id))
        .expect("Dispute not found");

    if dispute.status != DisputeStatus::Open {
        panic!("AlreadyResolved: This dispute has already been resolved");
    }

    if dispute.resolver != resolver {
        panic!("UnauthorizedResolver: Only the designated resolver can resolve this");
    }

    settle_escrow_by_outcome(e, dispute.escrow_id, release_to_beneficiary);

    dispute.status = if release_to_beneficiary {
        DisputeStatus::ResolvedForBeneficiary
    } else {
        DisputeStatus::ResolvedForDepositor
    };

    e.storage().persistent().set(&DataKey::Dispute(dispute_id), &dispute);
    e.storage()
        .persistent()
        .remove(&DataKey::EscrowDispute(dispute.escrow_id));

    e.events().publish(
        (symbol_short!("dispute_resolved"), dispute_id, resolver),
        release_to_beneficiary,
    );
}

/// Helper to read a dispute record.
pub fn get_dispute(e: &Env, dispute_id: u32) -> DisputeRecord {
    e.storage()
        .persistent()
        .get(&DataKey::Dispute(dispute_id))
        .expect("Dispute not found")
}
