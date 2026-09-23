# DUSD — Regulated Stablecoin on Solana (Token-2022)

> **Program ID:** `GyS3h3kXEHADYhzHXR2r9VBMYejFNeGwg7Ln66v2bqo5`
> **Token:** DHRUVIL USD (`DUSD`) · 6 decimals · 1% transfer fee (100 bps)

A regulated stablecoin built with Anchor 1.1 and SPL Token-2022, demonstrating
transfer fees, KYC-gated accounts, on-chain metadata, mint re-issuance with
confidential transfers, and seizure authority via the permanent delegate.

---

## Status

| # | Task | Status |
|---|---|---|
| 1 | Mint with 4 extensions, correct sizing, extension-init order before InitializeMint | ✅ Done |
| 2 | `transfer_checked_with_fee` + `calculate_epoch_fee(current_epoch, amount)` | ✅ Done |
| 3 | All state reads via `StateWithExtensions`, no raw unpack | ✅ Done |
| 4 | KYC unfreeze — `thaw_account` on individual account (not mint-level) | ✅ Done |
| 5a | V2 mint re-issued with `PermanentDelegate` + `ConfidentialTransferMint` (manual approve) | ✅ Done |
| 5b | `seize` instruction — permanent delegate transfers from sanctioned wallet | ❌ Not done |
| 6 | Full confidential transfer lifecycle (configure → deposit → apply → transfer → withdraw) | ❌ Not done |
| — | Tests for seize and confidential lifecycle | ❌ Not done |

---

## Architecture

```
programs/stablecoin-q3-2026/
├── src/
│   ├── lib.rs                        # Anchor program dispatcher
│   ├── constants.rs                  # DECIMALS, FEE_BPS, TOKEN_NAME ...
│   ├── error.rs                      # Custom error codes
│   ├── state.rs                      # On-chain account structs
│   └── instructions/
│       ├── initialize.rs             # Mint v1  — 4 extensions          ✅
│       ├── initialize_v2.rs          # Mint v2  — 6 extensions          ✅
│       ├── transfer.rs               # transfer_checked_with_fee        ✅
│       ├── unfreeze.rs               # KYC account-level thaw           ✅
│       ├── seize.rs                  # Permanent delegate seizure        ❌ TODO
│       └── confidential_transfer.rs  # Full CT lifecycle                ❌ TODO
└── tests/
    ├── integration.rs
    └── test_handler/
        ├── mod.rs
        ├── initialize.rs             # v1 mint helpers                  ✅
        ├── initialize_v2.rs          # v2 mint helpers                  ✅
        ├── transfer_fee.rs           # Token account + transfer helpers ✅
        ├── unfreeze.rs               # Frozen account helpers           ✅
        ├── seize.rs                  # Seizure helpers                  ❌ TODO
        └── confidential.rs           # Confidential transfer helpers    ❌ TODO
```

---

## Mint Versions

| Extension | Mint v1 | Mint v2 |
|---|---|---|
| `TransferFeeConfig` (1%, max = u64::MAX) | ✅ | ✅ |
| `MintCloseAuthority` | ✅ | ✅ |
| `DefaultAccountState` (Frozen) | ✅ | ✅ |
| `MetadataPointer` → self | ✅ | ✅ |
| Token Metadata (name, symbol, uri) | ✅ | ✅ |
| `PermanentDelegate` (seizure authority) | ❌ | ✅ |
| `ConfidentialTransferMint` (manual approve) | ❌ | ✅ |

**Why two mints?** `PermanentDelegate` and `ConfidentialTransferMint` cannot be added to
an existing mint after creation — they must be declared at genesis. When regulators
required seizure authority and hidden balances on the same instrument the mint had to be
re-issued, carrying forward all v1 extensions and adding the two new ones. This is the
gap between v1 and v2.

---

## Extension Design

### Sizing — `ExtensionType::try_calculate_account_len`

Extension space is computed before `CreateAccount` so the account is never
under-allocated:

```rust
let space = ExtensionType::try_calculate_account_len::<MintState>(fixed_extensions)?;
let lamports = Rent::get()?.minimum_balance(
    space + TLV_HEADER_LEN + metadata.get_packed_len()?,
);
```

`TLV_HEADER_LEN = 4` covers the 2-byte type tag + 2-byte length prefix on the inline
metadata entry.

### Extension Initialisation Order

All extension-specific inits must run **before** `InitializeMint2`:

```
CreateAccount (system)
  → InitializeTransferFeeConfig
  → InitializeMintCloseAuthority
  → InitializeDefaultAccountState  (Frozen)
  → InitializeMetadataPointer
  → InitializePermanentDelegate    (v2 only)
  → InitializeConfidentialTransfer (v2 only, auto_approve_new_accounts = false)
  → InitializeMint2
  → InitializeTokenMetadata        (after InitializeMint — needs authority set first)
```

### Reading State — `StateWithExtensions`

Every mint/account read uses `StateWithExtensions::unpack`, never raw `Mint::unpack`:

```rust
let data = self.mint.try_borrow_data()?;
let mint = StateWithExtensions::<MintState>::unpack(&data)?;
let fee = mint
    .get_extension::<TransferFeeConfig>()?
    .calculate_epoch_fee(Clock::get()?.epoch, amount)
    .ok_or(ProgramError::InvalidArgument)?;
```

---

## Instructions

### `initalize` — Mint v1 ✅
Creates the v1 mint. Payer becomes mint authority and freeze authority.

### `initalize_v2` — Mint v2 ✅
Creates the v2 mint. Payer becomes permanent delegate (seizure) and CT authority
(manual-approve policy).

### `transfer` ✅
Calls `transfer_checked_with_fee`. Fee is computed live via
`calculate_epoch_fee(current_epoch, amount)` — never a cached rate.

| Parameter | Type | Description |
|---|---|---|
| `amount` | u64 | Gross amount to send |
| `decimals` | u8 | Must match mint decimals (6) |

### `unfreeze` ✅
Freeze authority calls `thaw_account` on a single token account post-KYC.
`DefaultAccountState` stays `Frozen` — every new account remains frozen until
individually approved.

### `seize` ❌ — not yet implemented
Will use the permanent delegate authority to transfer tokens out of a sanctioned
wallet without the owner's signature.

### Confidential Transfer Lifecycle ❌ — not yet implemented

Planned five-step flow:

| Step | Instruction | Who signs |
|---|---|---|
| 1 | `configure_account` (owner-only, separate from ATA creation) | Account owner |
| 2 | `deposit_confidential` | Account owner |
| 3 | `apply_pending_balance` | Account owner |
| 4 | `confidential_transfer_ix` (with ZK proof) | Sender |
| 5 | `withdraw_confidential` (apply pending first, then withdraw) | Account owner |

---

## Building and Testing

### Prerequisites
- Rust toolchain defined in `rust-toolchain.toml`
- Solana CLI with `cargo build-sbf`
- No local validator needed — tests run in **LiteSVM**

### Build

```bash
cd stablecoin-q3-2026
cargo build-sbf
```

### Run Tests

```bash
cargo test -- --nocapture
```

---

## Test Coverage

| Test | Status | What it verifies |
|---|---|---|
| `test_initialize_mint_v1_extensions` | ✅ | V1 mint has all 4 extensions |
| `test_transfer_fee_via_program` | ✅ | 1% fee (100 bps) deducted correctly |
| `test_new_accounts_frozen_by_default` | ✅ | DefaultAccountState = Frozen enforced |
| `test_unfreeze_after_kyc` | ✅ | Freeze authority thaws one account; others frozen |
| `test_transfer_after_thaw` | ✅ | Transfer works only after KYC thaw |
| `test_mint_v2_extensions` | ✅ | V2 mint has all 6 extensions |
| `test_state_read_via_state_with_extensions` | ✅ | State read via StateWithExtensions |
| `test_seize_via_permanent_delegate` | ❌ | Permanent delegate seizes from sanctioned wallet |
| `test_confidential_lifecycle` | ❌ | Full 5-step CT flow end-to-end |

---

## Policy Finding

### What happens if a sanctioned user moves their balance into the confidential system before the permanent delegate acts?

Once tokens are in the **confidential (encrypted) balance**, the permanent delegate can
only act on the **visible (public) balance**. The encrypted amount is stored as a
ciphertext under the owner's ElGamal key — the permanent delegate has no way to read or
transfer it. A sanctioned user who deposits everything into the confidential state before
the issuer acts has effectively put those funds beyond the reach of the seizure
mechanism.

**This is a material compliance gap.**

### Mitigations

| Mitigation | How |
|---|---|
| Issuer-controlled CT approval | `auto_approve_new_accounts = false`. Revoke CT approval for flagged wallets before seizure to block `ApplyPendingBalance`. |
| Auditor ElGamal key | Set an auditor key on the mint at creation time so the issuer can decrypt balances for reporting even when protocol-level seizure is impossible. |
| Off-chain monitoring | Watch for `DepositConfidential` from watchlisted wallets; revoke CT approval immediately to freeze the pending encrypted amount before it becomes spendable. |
| Restricted CT eligibility | Require enhanced KYC before granting confidential-transfer access, limiting who can enter the privacy layer. |
