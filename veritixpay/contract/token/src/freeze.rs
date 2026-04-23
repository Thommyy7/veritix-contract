use crate::storage_types::DataKey;
use soroban_sdk::{symbol_short, Address, Env};

pub fn is_frozen(e: &Env, addr: &Address) -> bool {
    e.storage()
        .persistent()
        .get(&DataKey::Freeze(addr.clone()))
        .unwrap_or(false)
}

pub fn freeze_account(e: &Env, _admin: Address, target: Address) {
    let admin = _admin;
    e.storage()
        .persistent()
        .set(&DataKey::Freeze(target.clone()), &true);
    e.events().publish((symbol_short!("frozen"), target), admin);
}

pub fn unfreeze_account(e: &Env, _admin: Address, target: Address) {
    let admin = _admin;
    e.storage()
        .persistent()
        .remove(&DataKey::Freeze(target.clone()));
    e.events()
        .publish((symbol_short!("unfrozen"), target), admin);
}
