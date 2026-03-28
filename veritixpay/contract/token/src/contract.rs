use crate::admin::{check_admin, has_admin, read_admin, transfer_admin, write_admin};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{
    decrease_supply, increase_supply, read_balance, read_total_supply, receive_balance,
    spend_balance,
};
use crate::freeze::{freeze_account, is_frozen as read_frozen_status, unfreeze_account};
use crate::metadata::{
    read_decimal, read_name, read_symbol, validate_metadata, write_metadata, TokenMetadata,
};
use crate::validation::{require_not_frozen_account, require_positive_amount};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String};

#[contract]
pub struct VeritixToken;

#[contractimpl]
impl VeritixToken {
    // --- Admin & metadata ---

    /// Sets admin and metadata. Panics if already initialized.
    pub fn initialize(e: Env, admin: Address, name: String, symbol: String, decimal: u32) {
        if has_admin(&e) {
            panic!("already initialized");
        }

        admin.require_auth();

        let meta = TokenMetadata {
            name,
            symbol,
            decimal,
        };
        validate_metadata(&meta);
        write_admin(&e, &admin);
        write_metadata(&e, meta);
    }

    /// Rotates the contract administrator. Requires current admin auth.
    pub fn set_admin(e: Env, new_admin: Address) {
        transfer_admin(&e, new_admin);
    }

    /// Admin-only. Reclaims tokens from an address and destroys them.
    pub fn clawback(e: Env, admin: Address, from: Address, amount: i128) {
        check_admin(&e, &admin);
        require_positive_amount(amount);

        // Deduct balance without redistributing, effectively burning the tokens
        spend_balance(&e, from.clone(), amount);
        decrease_supply(&e, amount);

        // Emit transparency event
        e.events()
            .publish((symbol_short!("clawback"), from), amount);
    }

    // --- Freeze controls ---

    pub fn freeze(e: Env, target: Address) {
        let admin = read_admin(&e);
        check_admin(&e, &admin);
        freeze_account(&e, admin, target);
    }

    pub fn unfreeze(e: Env, target: Address) {
        let admin = read_admin(&e);
        check_admin(&e, &admin);
        unfreeze_account(&e, admin, target);
    }

    // --- Mint / burn & supply tracking ---

    /// Admin-only. Mints new tokens to a specific address.
    pub fn mint(e: Env, admin: Address, to: Address, amount: i128) {
        check_admin(&e, &admin);
        require_positive_amount(amount);
        receive_balance(&e, to.clone(), amount);
        increase_supply(&e, amount);
        e.events().publish((symbol_short!("mint"), to), amount);
    }

    /// Caller burns their own tokens.
    pub fn burn(e: Env, from: Address, amount: i128) {
        require_not_frozen_account(&e, &from);
        require_positive_amount(amount);
        from.require_auth();
        spend_balance(&e, from.clone(), amount);
        decrease_supply(&e, amount);
        e.events().publish((symbol_short!("burn"), from), amount);
    }

    /// Spender burns tokens from an account using their allowance.
    pub fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        require_not_frozen_account(&e, &from);
        require_not_frozen_account(&e, &spender);
        require_positive_amount(amount);
        spender.require_auth();
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount);
        decrease_supply(&e, amount);
        e.events().publish((symbol_short!("burn"), from), amount);
    }

    // --- Transfers & allowance ---

    /// Standard token transfer between two addresses.
    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        require_not_frozen_account(&e, &from);
        require_positive_amount(amount);
        from.require_auth();
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        e.events()
            .publish((symbol_short!("transfer"), from, to), amount);
    }

    /// Transfer tokens on behalf of a user via allowance.
    pub fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        require_not_frozen_account(&e, &from);
        require_not_frozen_account(&e, &spender);
        require_positive_amount(amount);
        spender.require_auth();
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        e.events()
            .publish((symbol_short!("transfer"), from, to), amount);
    }

    /// Sets an allowance for a spender.
    /// Frozen accounts cannot create new approvals.
    pub fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        require_not_frozen_account(&e, &from);
        from.require_auth();
        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        e.events()
            .publish((symbol_short!("approve"), from, spender), amount);
    }

    // --- Read-only views ---

    pub fn total_supply(e: Env) -> i128 {
        read_total_supply(&e)
    }

    pub fn balance(e: Env, id: Address) -> i128 {
        read_balance(&e, id)
    }

    pub fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&e, from, spender).amount
    }

    pub fn admin(e: Env) -> Address {
        read_admin(&e)
    }

    pub fn is_frozen(e: Env, id: Address) -> bool {
        read_frozen_status(&e, &id)
    }

    pub fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    pub fn name(e: Env) -> String {
        read_name(&e)
    }

    pub fn symbol(e: Env) -> String {
        read_symbol(&e)
    }
}
