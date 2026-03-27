use super::*;
use crate::Pallet;
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use polkadot_sdk::{frame_benchmarking, frame_support, frame_system};

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    pub fn trust_account() {
        let caller: T::AccountId = whitelisted_caller();
        let trusted: T::AccountId = account("trusted", 0, 0);

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            trusted.clone(),
        );

        assert!(AccountTrustedAccountIndex::<T>::contains_key(
            caller, trusted
        ));
    }

    #[benchmark]
    pub fn untrust_account() {
        let caller: T::AccountId = whitelisted_caller();
        let trusted: T::AccountId = account("trusted", 0, 0);

        assert_ok!(Pallet::<T>::trust_account(
            frame_system::RawOrigin::Signed(caller.clone()).into(),
            trusted.clone(),
        ));

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            trusted.clone(),
        );

        assert!(!AccountTrustedAccountIndex::<T>::contains_key(
            caller, trusted
        ));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
