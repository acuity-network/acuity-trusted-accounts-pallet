#![cfg_attr(not(feature = "std"), no_std)]

use polkadot_sdk::sp_api;

#[cfg(not(feature = "std"))]
use polkadot_sdk::sp_std::prelude::Vec;

sp_api::decl_runtime_apis! {
    pub trait TrustedAccountsApi<AccountId> where
        AccountId: codec::Codec,
    {
         fn is_trusted(account: AccountId, trustee: AccountId) -> bool;
         fn is_trusted_only_deep(account: AccountId, trustee: AccountId) -> bool;
         fn is_trusted_deep(account: AccountId, trustee: AccountId) -> bool;
         fn trusted_by(account: AccountId) -> Vec<AccountId>;
         fn trusted_by_that_trust(account: AccountId, account_is_trusted_by_trusted: AccountId) -> Vec<AccountId>;
    }
}
