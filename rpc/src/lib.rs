use codec::Codec;
use jsonrpsee::{core::RpcResult, proc_macros::rpc, types::ErrorObjectOwned};
use polkadot_sdk::{sp_api, sp_blockchain, sp_runtime};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

use std::sync::Arc;

pub use pallet_acuity_trusted_accounts_rpc_runtime_api::TrustedAccountsApi as TrustedAccountsRuntimeApi;

#[rpc(client, server)]
pub trait TrustedAccountsApi<AccountId, BlockHash> {
    #[method(name = "trustedAccounts_isTrusted")]
    fn is_trusted(
        &self,
        account: AccountId,
        trustee: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<bool>;

    #[method(name = "trustedAccounts_isTrustedOnlyDeep")]
    fn is_trusted_only_deep(
        &self,
        account: AccountId,
        trustee: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<bool>;

    #[method(name = "trustedAccounts_isTrustedDeep")]
    fn is_trusted_deep(
        &self,
        account: AccountId,
        trustee: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<bool>;

    #[method(name = "trustedAccounts_trustedBy")]
    fn trusted_by(&self, account: AccountId, at: Option<BlockHash>) -> RpcResult<Vec<AccountId>>;

    #[method(name = "trustedAccounts_trustedByThatTrust")]
    fn trusted_by_that_trust(
        &self,
        account: AccountId,
        account_is_trusted_by_trusted: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<AccountId>>;
}

pub struct TrustedAccounts<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> TrustedAccounts<C, P> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }

    fn runtime_error(e: impl ToString) -> ErrorObjectOwned {
        ErrorObjectOwned::owned(
            Error::RuntimeError.into(),
            "Unable to query dispatch info.",
            Some(e.to_string()),
        )
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

impl<C, AccountId, Block> TrustedAccountsApiServer<AccountId, <Block as BlockT>::Hash>
    for TrustedAccounts<C, Block>
where
    AccountId: Codec,
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: TrustedAccountsRuntimeApi<Block, AccountId>,
{
    fn is_trusted(
        &self,
        account: AccountId,
        trustee: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<bool> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.is_trusted(at_hash, account, trustee)
            .map_err(Self::runtime_error)
    }

    fn is_trusted_only_deep(
        &self,
        account: AccountId,
        trustee: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<bool> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.is_trusted_only_deep(at_hash, account, trustee)
            .map_err(Self::runtime_error)
    }

    fn is_trusted_deep(
        &self,
        account: AccountId,
        trustee: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<bool> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.is_trusted_deep(at_hash, account, trustee)
            .map_err(Self::runtime_error)
    }

    fn trusted_by(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<AccountId>> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.trusted_by(at_hash, account)
            .map_err(Self::runtime_error)
    }

    fn trusted_by_that_trust(
        &self,
        account: AccountId,
        account_is_trusted_by_trusted: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<AccountId>> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.trusted_by_that_trust(at_hash, account, account_is_trusted_by_trusted)
            .map_err(Self::runtime_error)
    }
}
