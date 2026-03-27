use polkadot_sdk::frame_support::weights::Weight;

pub trait WeightInfo {
    fn trust_account() -> Weight;
    fn untrust_account() -> Weight;
}

impl WeightInfo for () {
    fn trust_account() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn untrust_account() -> Weight {
        Weight::from_parts(10_000, 0)
    }
}
