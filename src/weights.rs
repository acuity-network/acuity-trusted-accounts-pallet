use core::marker::PhantomData;
use polkadot_sdk::frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

pub trait WeightInfo {
    fn trust_account() -> Weight;
    fn untrust_account() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: polkadot_sdk::frame_system::Config> WeightInfo for SubstrateWeight<T>
where
    T::DbWeight: Get<polkadot_sdk::frame_support::weights::RuntimeDbWeight>,
{
    fn trust_account() -> Weight {
        Weight::from_parts(12_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(2, 3))
    }

    fn untrust_account() -> Weight {
        Weight::from_parts(18_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(3, 5))
    }
}

impl WeightInfo for () {
    fn trust_account() -> Weight {
        Weight::from_parts(12_000_000, 0).saturating_add(RocksDbWeight::get().reads_writes(2, 3))
    }

    fn untrust_account() -> Weight {
        Weight::from_parts(18_000_000, 0).saturating_add(RocksDbWeight::get().reads_writes(3, 5))
    }
}
