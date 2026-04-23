#[cfg(test)]
mod admin_test {
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::admin::{read_admin, transfer_admin, write_admin};

    #[test]
    fn test_transfer_admin() {
        let e = Env::default();
        let admin = Address::generate(&e);
        let new_admin = Address::generate(&e);

        write_admin(&e, &admin);

        e.mock_all_auths();
        transfer_admin(&e, new_admin.clone());

        assert_eq!(read_admin(&e), new_admin);
    }

    #[test]
    #[should_panic]
    fn test_transfer_admin_unauthorized_panics() {
        let e = Env::default();
        let admin = Address::generate(&e);
        let new_admin = Address::generate(&e);

        write_admin(&e, &admin);

        // No mock auths — transfer_admin requires the current admin to authorize
        e.set_auths(&[]);
        transfer_admin(&e, new_admin);
    }

    #[test]
    fn test_transfer_admin_to_same_address() {
        let e = Env::default();
        let admin = Address::generate(&e);

        write_admin(&e, &admin);
        e.mock_all_auths();
        transfer_admin(&e, admin.clone());
        assert_eq!(read_admin(&e), admin);
    }
}

#[cfg(test)]
mod admin_test {
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::admin::{read_admin, transfer_admin, write_admin};

    #[test]
    fn test_transfer_admin() {
        let e = Env::default();
        let admin = Address::generate(&e);
        let new_admin = Address::generate(&e);

        write_admin(&e, &admin);

        e.mock_all_auths();
        transfer_admin(&e, new_admin.clone());

        assert_eq!(read_admin(&e), new_admin);
    }

    #[test]
    #[should_panic]
    fn test_transfer_admin_unauthorized_panics() {
        let e = Env::default();
        let admin = Address::generate(&e);
        let new_admin = Address::generate(&e);

        write_admin(&e, &admin);

        // No mock auths — transfer_admin requires the current admin to authorize
        e.set_auths(&[]);
        transfer_admin(&e, new_admin);
    }
}
