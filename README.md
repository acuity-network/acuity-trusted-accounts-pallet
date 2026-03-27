# `pallet-acuity-trusted-accounts`

`pallet-acuity-trusted-accounts` is a lightweight FRAME pallet for managing account-to-account trust relationships inside a Substrate runtime.

It lets any signed account maintain its own on-chain list of trusted accounts, remove entries efficiently, and query both direct trust and a simple one-hop "deep" trust relationship.

## Features

- Direct trust graph managed by end users through dispatchable calls.
- Constant-time membership checks using an index map.
- Constant-time removal with swap-remove style list compaction.
- Runtime helper functions for direct and one-hop trust queries.
- Configurable maximum trust-list size per account.
- Unit tests and benchmarks included in the repository.

## Repository layout

- `src/lib.rs` - pallet implementation.
- `src/weights.rs` - weight trait and default weights.
- `src/benchmarking.rs` - benchmark definitions.
- `src/mock.rs` - mock runtime used by tests.
- `src/tests.rs` - pallet unit tests.

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
- Rejects inserts once the configured trust-list limit is reached.
- Appends the new trusted account to the end of the caller's list.
- Updates the count and reverse index.
- Emits `AccountTrusted(truster, trustee)`.

Possible errors:

- `TrustSelf`
- `AlreadyTrusted`
- `TooManyTrustedAccounts`

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
- `TooManyTrustedAccounts` - the caller reached the configured trust-list limit.
- `NotTrusted` - the trust edge does not exist.
- `BadStorageState` - the pallet detected inconsistent trust-list storage.

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

### Bounded trust list size

The pallet enforces a runtime-configured `MaxTrustedAccounts` limit for each account.

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
    type MaxTrustedAccounts = ConstU32<256>;
    type WeightInfo = pallet_acuity_trusted_accounts::weights::SubstrateWeight<Runtime>;
}

construct_runtime!(
    pub enum Runtime {
        // ...
        TrustedAccounts: pallet_acuity_trusted_accounts,
    }
);
```

If you generate benchmark-based weights for your runtime, replace the default `WeightInfo` implementation with your runtime-specific generated type.

## Weights and benchmarks

The pallet defines a `WeightInfo` trait in `src/weights.rs` with two functions:

- `trust_account()`
- `untrust_account()`

The default implementation includes explicit database read/write accounting. For production runtimes, benchmark the pallet and plug in generated weights.

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
- Function naming for `trusted_by` and `trusted_by_that_trust` reflects the current code, even though both operate on accounts trusted by the provided account.

## License

Apache-2.0
