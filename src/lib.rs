#![cfg_attr(not(feature = "std"), no_std)]

use polkadot_sdk::frame_support;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use polkadot_sdk::frame_system::pallet_prelude::*;
    use polkadot_sdk::{frame_support::pallet_prelude::*, sp_std};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        polkadot_sdk::frame_system::Config<
        RuntimeEvent: From<Event<Self>>
                          + IsType<<Self as polkadot_sdk::frame_system::Config>::RuntimeEvent>,
    >
    {
    }

    #[pallet::storage]
    #[pallet::getter(fn account_trusted_account_list_count)]
    // Mapping of account to count of accounts that it trusts.
    pub type AccountTrustedAccountListCount<T: Config> =
        StorageMap<_, Identity, T::AccountId, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_trusted_account_list)]
    // Mapping of account to array of trusted accounts.
    pub type AccountTrustedAccountList<T: Config> =
        StorageDoubleMap<_, Identity, T::AccountId, Twox64Concat, u32, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn account_trusted_account_index)]
    // Mapping of account1 to mapping of account2 to index + 1 in AccountTrustedAccountList.
    pub type AccountTrustedAccountIndex<T: Config> =
        StorageDoubleMap<_, Identity, T::AccountId, Blake2_128Concat, T::AccountId, u32>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An account has trusted another. [truster, trustee]
        AccountTrusted(T::AccountId, T::AccountId),
        /// An account has untrusted another. [truster, trustee]
        AccountUntrusted(T::AccountId, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// It is not possible to trust self
        TrustSelf,
        /// The account is already trusted.
        AlreadyTrusted,
        /// The account is not trusted.
        NotTrusted,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn trust_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let sender = ensure_signed(origin)?;
            // Check that the sender is not attempting to trust themselves.
            if sender == account {
                Err(Error::<T>::TrustSelf)?;
            }
            // Check that the account is not already trusted.
            if <AccountTrustedAccountIndex<T>>::contains_key(&sender, &account) {
                Err(Error::<T>::AlreadyTrusted)?;
            }
            // Get the total number of accounts the sender already trusts.
            let count = <AccountTrustedAccountListCount<T>>::get(&sender);

            //----------------------------------------

            // Insert the new account at the end of the list.
            <AccountTrustedAccountList<T>>::insert(&sender, count, &account);
            // Update the size of the list.
            <AccountTrustedAccountListCount<T>>::insert(&sender, count + 1);
            // Store index + 1 for this trust pair.
            <AccountTrustedAccountIndex<T>>::insert(&sender, &account, count + 1);
            // Emit the event.
            Self::deposit_event(Event::AccountTrusted(sender, account));
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn untrust_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let sender = ensure_signed(origin)?;
            // Get the index + 1 of the account to be removed
            let i = match <AccountTrustedAccountIndex<T>>::get(&sender, &account) {
                Some(i) => i,
                None => return Err(Error::<T>::NotTrusted.into()),
            };

            //----------------------------------------

            // Delete the index from state.
            <AccountTrustedAccountIndex<T>>::remove(&sender, &account);
            // Get the list length.
            let count = <AccountTrustedAccountListCount<T>>::get(&sender);
            // Check if this is not the last account.
            if i != count {
                // Get the last account.
                let moving_account =
                    <AccountTrustedAccountList<T>>::get(&sender, count - 1).unwrap();
                // Overwrite the account being untrusted with the last account.
                <AccountTrustedAccountList<T>>::insert(&sender, i - 1, &moving_account);
                // Update the index + 1 of the last account.
                <AccountTrustedAccountIndex<T>>::insert(&sender, moving_account, i);
            }
            // Remove the last account.
            <AccountTrustedAccountList<T>>::remove(&sender, count - 1);
            <AccountTrustedAccountListCount<T>>::insert(&sender, count - 1);
            // Emit the event.
            Self::deposit_event(Event::AccountUntrusted(sender, account));
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn is_trusted(account: T::AccountId, trustee: T::AccountId) -> bool {
            AccountTrustedAccountIndex::<T>::contains_key(&account, &trustee)
        }

        pub fn is_trusted_only_deep(account: T::AccountId, trustee: T::AccountId) -> bool {
            let count = AccountTrustedAccountListCount::<T>::get(&account);
            let mut i = 0;
            while i < count {
                if AccountTrustedAccountIndex::<T>::contains_key(
                    AccountTrustedAccountList::<T>::get(&account, i).unwrap(),
                    &trustee,
                ) {
                    return true;
                }

                i += 1;
            }

            false
        }

        pub fn is_trusted_deep(account: T::AccountId, trustee: T::AccountId) -> bool {
            if AccountTrustedAccountIndex::<T>::contains_key(&account, &trustee) {
                return true;
            }

            Self::is_trusted_only_deep(account, trustee)
        }

        pub fn trusted_by(account: T::AccountId) -> sp_std::prelude::Vec<T::AccountId> {
            let mut accounts = sp_std::prelude::Vec::new();
            let count = AccountTrustedAccountListCount::<T>::get(&account);

            let mut i = 0;
            while i < count {
                accounts.push(AccountTrustedAccountList::<T>::get(&account, i).unwrap());
                i += 1;
            }

            accounts
        }

        pub fn trusted_by_that_trust(
            account: T::AccountId,
            account_is_trusted_by_trusted: T::AccountId,
        ) -> sp_std::prelude::Vec<T::AccountId> {
            let mut accounts_trusted_that_trust = sp_std::prelude::Vec::new();
            let accounts_trusted = Self::trusted_by(account);

            for account_trusted in accounts_trusted {
                if Self::is_trusted(
                    account_trusted.clone(),
                    account_is_trusted_by_trusted.clone(),
                ) {
                    accounts_trusted_that_trust.push(account_trusted);
                }
            }

            accounts_trusted_that_trust
        }
    }
}
