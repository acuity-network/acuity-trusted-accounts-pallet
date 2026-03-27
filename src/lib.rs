#![cfg_attr(not(feature = "std"), no_std)]

use polkadot_sdk::frame_support;

pub use pallet::*;
pub use weights::WeightInfo;

pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[frame_support::pallet]
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
        #[pallet::constant]
        type MaxTrustedAccounts: Get<u32>;

        type WeightInfo: crate::WeightInfo;
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

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An account has trusted another. [truster, trustee]
        AccountTrusted(T::AccountId, T::AccountId),
        /// An account has untrusted another. [truster, trustee]
        AccountUntrusted(T::AccountId, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// It is not possible to trust self.
        TrustSelf,
        /// The account is already trusted.
        AlreadyTrusted,
        /// The account has reached the maximum number of trusted accounts.
        TooManyTrustedAccounts,
        /// The account is not trusted.
        NotTrusted,
        /// The trust list storage is internally inconsistent.
        BadStorageState,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<<T as Config>::WeightInfo as crate::WeightInfo>::trust_account())]
        pub fn trust_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            if sender == account {
                Err(Error::<T>::TrustSelf)?;
            }

            if <AccountTrustedAccountIndex<T>>::contains_key(&sender, &account) {
                Err(Error::<T>::AlreadyTrusted)?;
            }

            let count = <AccountTrustedAccountListCount<T>>::get(&sender);
            ensure!(
                count < T::MaxTrustedAccounts::get(),
                Error::<T>::TooManyTrustedAccounts
            );

            let next_count = count.checked_add(1).ok_or(Error::<T>::BadStorageState)?;

            <AccountTrustedAccountList<T>>::insert(&sender, count, &account);
            <AccountTrustedAccountListCount<T>>::insert(&sender, next_count);
            <AccountTrustedAccountIndex<T>>::insert(&sender, &account, next_count);
            Self::deposit_event(Event::AccountTrusted(sender, account));
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<<T as Config>::WeightInfo as crate::WeightInfo>::untrust_account())]
        pub fn untrust_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let index = match <AccountTrustedAccountIndex<T>>::get(&sender, &account) {
                Some(index) => index,
                None => return Err(Error::<T>::NotTrusted.into()),
            };

            let count = <AccountTrustedAccountListCount<T>>::get(&sender);
            ensure!(count > 0, Error::<T>::BadStorageState);
            ensure!(index <= count, Error::<T>::BadStorageState);

            let remove_index = index.checked_sub(1).ok_or(Error::<T>::BadStorageState)?;
            let new_count = count.checked_sub(1).ok_or(Error::<T>::BadStorageState)?;

            <AccountTrustedAccountIndex<T>>::remove(&sender, &account);

            if index != count {
                let moving_account = <AccountTrustedAccountList<T>>::get(&sender, new_count)
                    .ok_or(Error::<T>::BadStorageState)?;

                <AccountTrustedAccountList<T>>::insert(&sender, remove_index, &moving_account);
                <AccountTrustedAccountIndex<T>>::insert(&sender, moving_account, index);
            }

            <AccountTrustedAccountList<T>>::remove(&sender, new_count);
            <AccountTrustedAccountListCount<T>>::insert(&sender, new_count);
            Self::deposit_event(Event::AccountUntrusted(sender, account));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn trusted_accounts(account: &T::AccountId) -> sp_std::prelude::Vec<T::AccountId> {
            let mut accounts = sp_std::prelude::Vec::new();
            let count = AccountTrustedAccountListCount::<T>::get(account);

            let mut i = 0;
            while i < count {
                if let Some(trusted_account) = AccountTrustedAccountList::<T>::get(account, i) {
                    accounts.push(trusted_account);
                }

                i += 1;
            }

            accounts
        }

        pub fn is_trusted(account: T::AccountId, trustee: T::AccountId) -> bool {
            AccountTrustedAccountIndex::<T>::contains_key(&account, &trustee)
        }

        pub fn is_trusted_only_deep(account: T::AccountId, trustee: T::AccountId) -> bool {
            for trusted_account in Self::trusted_accounts(&account) {
                if AccountTrustedAccountIndex::<T>::contains_key(trusted_account, &trustee) {
                    return true;
                }
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
            Self::trusted_accounts(&account)
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
