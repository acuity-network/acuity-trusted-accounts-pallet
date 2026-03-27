use crate as pallet_acuity_trusted_accounts;
use crate::Config;
use frame_support::derive_impl;
use frame_support::traits::ConstU32;
use polkadot_sdk::{frame_support, frame_system, sp_io};

pub type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        TrustedAccounts: pallet_acuity_trusted_accounts,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type Block = Block;
}

impl Config for Test {
    type MaxTrustedAccounts = ConstU32<4>;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = sp_io::TestExternalities::new(Default::default());
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}
