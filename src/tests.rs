use crate::{mock::*, Error};
use polkadot_sdk::frame_support::{assert_noop, assert_ok};

#[test]
fn trust_account_cant_trust_self_control() {
    new_test_ext().execute_with(|| {
        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 2));
    });
}

#[test]
fn trust_account_cant_trust_self() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 1),
            Error::<Test>::TrustSelf
        );
    });
}

#[test]
fn trust_account_already_trusted_control() {
    new_test_ext().execute_with(|| {
        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 2));
        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 3));
    });
}

#[test]
fn trust_account_already_trusted() {
    new_test_ext().execute_with(|| {
        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 2));
        assert_noop!(
            TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 2),
            Error::<Test>::AlreadyTrusted
        );
    });
}

#[test]
fn trust_account() {
    new_test_ext().execute_with(|| {
        assert_eq!(TrustedAccounts::account_trusted_account_list_count(1), 0);

        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 2));
        assert_eq!(TrustedAccounts::account_trusted_account_list_count(1), 1);
        let i = TrustedAccounts::account_trusted_account_index(1, 2).unwrap() - 1;
        assert_eq!(
            TrustedAccounts::account_trusted_account_list(1, i).unwrap(),
            2
        );

        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 3));
        assert_eq!(TrustedAccounts::account_trusted_account_list_count(1), 2);
        let i = TrustedAccounts::account_trusted_account_index(1, 3).unwrap() - 1;
        assert_eq!(
            TrustedAccounts::account_trusted_account_list(1, i).unwrap(),
            3
        );

        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 4));
        assert_eq!(TrustedAccounts::account_trusted_account_list_count(1), 3);
        let i = TrustedAccounts::account_trusted_account_index(1, 4).unwrap() - 1;
        assert_eq!(
            TrustedAccounts::account_trusted_account_list(1, i).unwrap(),
            4
        );

        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 5));
        assert_eq!(TrustedAccounts::account_trusted_account_list_count(1), 4);
        let i = TrustedAccounts::account_trusted_account_index(1, 5).unwrap() - 1;
        assert_eq!(
            TrustedAccounts::account_trusted_account_list(1, i).unwrap(),
            5
        );
    });
}

#[test]
fn untrust_account_not_trusted_control() {
    new_test_ext().execute_with(|| {
        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 2));
        assert_ok!(TrustedAccounts::untrust_account(
            RuntimeOrigin::signed(1),
            2
        ));
    });
}

#[test]
fn untrust_account_not_trusted() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TrustedAccounts::untrust_account(RuntimeOrigin::signed(1), 2),
            Error::<Test>::NotTrusted
        );
    });
}

#[test]
fn untrust_account() {
    new_test_ext().execute_with(|| {
        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 2));
        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 3));
        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 4));
        assert_ok!(TrustedAccounts::trust_account(RuntimeOrigin::signed(1), 5));
        assert_eq!(TrustedAccounts::account_trusted_account_list_count(1), 4);

        assert_ok!(TrustedAccounts::untrust_account(
            RuntimeOrigin::signed(1),
            3
        ));
        assert_eq!(TrustedAccounts::account_trusted_account_list_count(1), 3);
        assert_eq!(TrustedAccounts::account_trusted_account_index(1, 3), None);

        assert_ok!(TrustedAccounts::untrust_account(
            RuntimeOrigin::signed(1),
            5
        ));
        assert_eq!(TrustedAccounts::account_trusted_account_list_count(1), 2);
        assert_eq!(TrustedAccounts::account_trusted_account_index(1, 5), None);

        assert_ok!(TrustedAccounts::untrust_account(
            RuntimeOrigin::signed(1),
            2
        ));
        assert_eq!(TrustedAccounts::account_trusted_account_list_count(1), 1);
        assert_eq!(TrustedAccounts::account_trusted_account_index(1, 2), None);

        assert_ok!(TrustedAccounts::untrust_account(
            RuntimeOrigin::signed(1),
            4
        ));
        assert_eq!(TrustedAccounts::account_trusted_account_list_count(1), 0);
        assert_eq!(TrustedAccounts::account_trusted_account_index(1, 4), None);
    });
}
