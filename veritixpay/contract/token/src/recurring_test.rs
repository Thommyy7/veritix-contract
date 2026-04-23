#[cfg(test)]
mod recurring_tests {
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::balance::read_balance;
    use crate::contract::VeritixToken;
    use crate::recurring::{cancel_recurring, execute_recurring, get_recurring, setup_recurring};

    fn setup_env() -> Env {
        let e = Env::default();
        e.mock_all_auths();
        e
    }

    fn fund_and_setup(
        e: &Env,
        contract_id: &Address,
        amount: i128,
        interval: u32,
    ) -> (Address, Address, u32) {
        let payer = Address::generate(e);
        let payee = Address::generate(e);
        let mut id = 0u32;
        e.as_contract(contract_id, || {
            crate::balance::receive_balance(e, payer.clone(), amount);
            id = setup_recurring(e, payer.clone(), payee.clone(), amount, interval);
        });
        (payer, payee, id)
    }

    #[test]
    #[should_panic(expected = "InvalidRecurring: payer and payee cannot be the same address")]
    fn test_setup_recurring_same_address_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let addr = Address::generate(&e);

        e.as_contract(&contract_id, || {
            crate::balance::receive_balance(&e, addr.clone(), 500);
            setup_recurring(&e, addr.clone(), addr.clone(), 500, 100);
        });
    }

    #[test]
    fn test_setup_stores_record() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        e.as_contract(&contract_id, || {
            let r = get_recurring(&e, id);
            assert_eq!(r.payer, payer);
            assert_eq!(r.payee, payee);
            assert_eq!(r.amount, 500);
            assert_eq!(r.interval, 100);
            assert!(r.active);
        });
    }

    #[test]
    fn test_execute_transfers_funds() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        e.as_contract(&contract_id, || {
            // Advance ledger past the interval.
            e.ledger().set_sequence_number(e.ledger().sequence() + 101);
            execute_recurring(&e, id);
            assert_eq!(read_balance(&e, payee.clone()), 500);
            assert_eq!(read_balance(&e, payer.clone()), 0);
        });
    }

    #[test]
    #[should_panic(expected = "interval has not elapsed")]
    fn test_execute_too_early_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (_payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        e.as_contract(&contract_id, || {
            // Only advance by 50 — not enough.
            e.ledger().set_sequence_number(e.ledger().sequence() + 50);
            execute_recurring(&e, id);
        });
    }

    #[test]
    fn test_cancel_deactivates_record() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        e.as_contract(&contract_id, || {
            cancel_recurring(&e, payer.clone(), id);
            let r = get_recurring(&e, id);
            assert!(!r.active);
        });
    }

    #[test]
    #[should_panic(expected = "unauthorized")]
    fn test_cancel_unauthorized_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (_payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);
        let hacker = Address::generate(&e);

        e.as_contract(&contract_id, || {
            cancel_recurring(&e, hacker, id);
        });
    }

    #[test]
    #[should_panic(expected = "recurring payment is not active")]
    fn test_execute_after_cancel_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        e.as_contract(&contract_id, || {
            cancel_recurring(&e, payer.clone(), id);
            e.ledger().set_sequence_number(e.ledger().sequence() + 200);
            execute_recurring(&e, id);
        });
    }

    #[test]
    #[should_panic(expected = "InsufficientBalance")]
    fn test_execute_recurring_insufficient_balance_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        // Fund payer with less than the recurring amount.
        let (payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);
        e.as_contract(&contract_id, || {
            // Drain the payer balance so they can no longer cover the charge.
            crate::balance::spend_balance(&e, payer.clone(), 500);
            e.ledger().set_sequence_number(e.ledger().sequence() + 101);
            execute_recurring(&e, id);
        });
    }

    #[test]
    fn test_multiple_executions() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        // Fund payer with enough for two charges.
        let (payer, payee, id) = fund_and_setup(&e, &contract_id, 1_000, 100);
        // Give payer extra balance for second charge.
        e.as_contract(&contract_id, || {
            crate::balance::receive_balance(&e, payer.clone(), 1_000);
        });

        e.as_contract(&contract_id, || {
            let start = e.ledger().sequence();

            e.ledger().set_sequence_number(start + 101);
            execute_recurring(&e, id);
            assert_eq!(read_balance(&e, payee.clone()), 1_000);

            e.ledger().set_sequence_number(start + 202);
            execute_recurring(&e, id);
            assert_eq!(read_balance(&e, payee.clone()), 2_000);
        });
    }
}
