# `pallet-acuity-trusted-accounts`

`pallet-acuity-trusted-accounts` is a lightweight FRAME pallet for managing account-to-account trust relationships inside a Substrate runtime.

It lets any signed account maintain its own on-chain list of trusted accounts, remove entries efficiently, and query both direct trust and a simple one-hop "deep" trust relationship.

## Features

- Direct trust graph managed by end users through dispatchable calls.
- Constant-time membership checks using an index map.
- Constant-time removal with swap-remove style list compaction.
- Runtime helper functions for direct and one-hop trust queries.
- Optional runtime API and JSON-RPC crates for exposing trust queries to off-chain clients.
- Unit tests and benchmarks included in the repository.

## Repository layout

- `src/lib.rs` - pallet implementation.
- `src/weights.rs` - weight trait and default weights.
- `src/benchmarking.rs` - benchmark definitions.
- `src/mock.rs` - mock runtime used by tests.
- `src/tests.rs` - pallet unit tests.
- `rpc/runtime-api` - runtime API definitions for off-chain queries.
- `rpc` - JSON-RPC server implementation built on `jsonrpsee`.

## What the pallet does

Each account can maintain a personal list of accounts that it trusts.

The pallet currently supports:

1. Adding a trusted account.
2. Removing a trusted account.
3. Checking whether an account directly trusts another account.
4. Checking whether an account is trusted by any account in the caller's direct trust list.
5. Listing all accounts directly trusted by an account.
6. Filtering that direct trust list by whether those trusted accounts also trust another account.

This makes the pallet useful for social trust, reputation overlays, curation systems, delegated discovery, or any runtime logic that needs a compact trust graph with cheap direct lookups.

## Storage layout

The pallet stores trust relationships using three storage items:

### `AccountTrustedAccountListCount`

```rust
StorageMap<AccountId, u32>
```

Tracks how many accounts a given account currently trusts.

### `AccountTrustedAccountList`

```rust
StorageDoubleMap<AccountId, u32, AccountId>
```

Stores the trusted accounts for each owner as a dense zero-based list.

### `AccountTrustedAccountIndex`

```rust
StorageDoubleMap<AccountId, AccountId, u32>
```

Stores `index + 1` for each `(truster, trustee)` pair. Using `index + 1` makes `0` available as the implicit "not present" value when the key is missing.

## Dispatchable calls

### `trust_account(origin, account)`

Adds `account` to the signed origin's trust list.

Behavior:

- Requires a signed origin.
- Rejects trusting yourself.
- Rejects duplicate trust edges.
- Appends the new trusted account to the end of the caller's list.
- Updates the count and reverse index.
- Emits `AccountTrusted(truster, trustee)`.

Possible errors:

- `TrustSelf`
- `AlreadyTrusted`

### `untrust_account(origin, account)`

Removes `account` from the signed origin's trust list.

Behavior:

- Requires a signed origin.
- Fails if the account is not currently trusted.
- Removes the trust edge in O(1) time by moving the last list item into the removed slot when needed.
- Updates the count and reverse index.
- Emits `AccountUntrusted(truster, trustee)`.

Possible errors:

- `NotTrusted`

## Events

- `AccountTrusted(T::AccountId, T::AccountId)` - a trust edge was created.
- `AccountUntrusted(T::AccountId, T::AccountId)` - a trust edge was removed.

## Errors

- `TrustSelf` - an account attempted to trust itself.
- `AlreadyTrusted` - the trust edge already exists.
- `NotTrusted` - the trust edge does not exist.

## Runtime helper functions

In addition to extrinsics, the pallet exposes helper functions from `Pallet<T>`.

### `is_trusted(account, trustee) -> bool`

Returns `true` if `account` directly trusts `trustee`.

Complexity: O(1)

### `is_trusted_only_deep(account, trustee) -> bool`

Returns `true` if any account directly trusted by `account` directly trusts `trustee`.

This is a one-hop transitive check. It does not recurse beyond that second level.

Complexity: O(n), where `n` is the number of accounts directly trusted by `account`.

### `is_trusted_deep(account, trustee) -> bool`

Returns `true` if either:

- `account` directly trusts `trustee`, or
- any directly trusted account of `account` directly trusts `trustee`.

Complexity: O(1) for the direct hit path, otherwise O(n).

### `trusted_by(account) -> Vec<AccountId>`

Returns the accounts directly trusted by `account`.

Important: despite the name, this function returns the accounts that `account` trusts, not the accounts that trust `account`.

Complexity: O(n)

### `trusted_by_that_trust(account, other) -> Vec<AccountId>`

Returns the subset of `account`'s directly trusted accounts that also directly trust `other`.

Important: the naming follows the existing code, but the behavior is best read as "accounts trusted by `account` that also trust `other`".

Complexity: O(n)

## Implementation notes

### Efficient removals

The pallet uses a dense list plus an index map. On removal, it:

1. Deletes the `(truster, trustee)` index entry.
2. Reads the last account in the list.
3. Moves that account into the removed position when necessary.
4. Updates the moved account's stored index.
5. Shrinks the list count by one.

This keeps storage compact and avoids shifting every later element.

### Ordering is not stable

Because removal uses a swap-remove pattern, the order of trusted accounts can change after `untrust_account` is called. If downstream code depends on stable ordering, it must sort or otherwise normalize results outside the pallet.

### No explicit max trust list size

The pallet does not currently enforce an upper bound on how many accounts one account may trust. Runtime integrators should consider whether they want to wrap or extend this pallet with limits, deposits, or custom weights if very large trust lists are possible in their environment.

### One-hop deep trust only

`is_trusted_only_deep` and `is_trusted_deep` do not perform arbitrary graph traversal. They only inspect direct trust and one additional hop.

## Runtime integration

Add the crate to your runtime dependencies and include it in `construct_runtime!`.

### `Cargo.toml`

```toml
[dependencies]
pallet-acuity-trusted-accounts = { git = "https://github.com/acuity-social/acuity-trusted-accounts-pallet", default-features = false }

[features]
std = [
  "pallet-acuity-trusted-accounts/std",
]
```

### Runtime configuration

```rust
impl pallet_acuity_trusted_accounts::Config for Runtime {
    type WeightInfo = ();
}

construct_runtime!(
    pub enum Runtime {
        // ...
        TrustedAccounts: pallet_acuity_trusted_accounts,
    }
);
```

If you generate benchmark-based weights for your runtime, replace the default `WeightInfo` implementation with your runtime-specific generated type.

## Exposing the runtime API

The repository includes an optional runtime API crate at `rpc/runtime-api`:

- `is_trusted(account, trustee) -> bool`
- `is_trusted_only_deep(account, trustee) -> bool`
- `is_trusted_deep(account, trustee) -> bool`
- `trusted_by(account) -> Vec<AccountId>`
- `trusted_by_that_trust(account, other) -> Vec<AccountId>`

To expose these methods from your runtime, implement the runtime API by forwarding to the pallet helpers.

Example shape:

```rust
impl pallet_acuity_trusted_accounts_rpc_runtime_api::TrustedAccountsApi<Block, AccountId>
    for Runtime
{
    fn is_trusted(account: AccountId, trustee: AccountId) -> bool {
        TrustedAccounts::is_trusted(account, trustee)
    }

    fn is_trusted_only_deep(account: AccountId, trustee: AccountId) -> bool {
        TrustedAccounts::is_trusted_only_deep(account, trustee)
    }

    fn is_trusted_deep(account: AccountId, trustee: AccountId) -> bool {
        TrustedAccounts::is_trusted_deep(account, trustee)
    }

    fn trusted_by(account: AccountId) -> Vec<AccountId> {
        TrustedAccounts::trusted_by(account)
    }

    fn trusted_by_that_trust(account: AccountId, other: AccountId) -> Vec<AccountId> {
        TrustedAccounts::trusted_by_that_trust(account, other)
    }
}
```

## JSON-RPC integration

The `rpc` crate exposes the runtime API through `jsonrpsee`.

RPC methods:

- `trustedAccounts_isTrusted`
- `trustedAccounts_isTrustedOnlyDeep`
- `trustedAccounts_isTrustedDeep`
- `trustedAccounts_trustedBy`
- `trustedAccounts_trustedByThatTrust`

These methods accept an optional block hash so clients can query either the latest state or historical state at a specific block.

Typical node wiring looks like this:

```rust
use pallet_acuity_trusted_accounts_rpc::TrustedAccounts;

module.merge(TrustedAccounts::new(client.clone()).into_rpc())?;
```

## Weights and benchmarks

The pallet defines a `WeightInfo` trait in `src/weights.rs` with two functions:

- `trust_account()`
- `untrust_account()`

The default implementation returns a fixed placeholder weight of `10_000` for each call. For production runtimes, benchmark the pallet and plug in generated weights.

Benchmark support is provided behind the `runtime-benchmarks` feature.

## Testing

Run the pallet tests with:

```bash
cargo test
```

The included unit tests cover:

- successful trust creation,
- prevention of self-trust,
- prevention of duplicate trust,
- failure when untrusting a missing edge,
- successful removal from different list positions.

## Example behavior

Assume the following trust graph:

- `Alice -> Bob`
- `Alice -> Charlie`
- `Bob -> Dave`

Then:

- `is_trusted(Alice, Bob)` returns `true`
- `is_trusted(Alice, Dave)` returns `false`
- `is_trusted_only_deep(Alice, Dave)` returns `true`
- `is_trusted_deep(Alice, Dave)` returns `true`
- `trusted_by(Alice)` returns `[Bob, Charlie]`
- `trusted_by_that_trust(Alice, Dave)` returns `[Bob]`

## Compatibility

- Rust edition: `2021`
- Polkadot SDK tag: `polkadot-stable2512-3`
- License: `Apache-2.0`

## Limitations and considerations

- No deposits or economic spam controls are built in.
- No origin restrictions beyond requiring a signed caller.
- No recursive or weighted trust model is included.
- No pagination helpers are provided for very large trust lists.
- Function naming for `trusted_by` and `trusted_by_that_trust` reflects the current code, even though both operate on accounts trusted by the provided account.

## License

Apache-2.0
