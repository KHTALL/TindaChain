# TindaChain 🏪⛓️

> On-chain inventory management and supplier escrow for Manila's sari-sari store network — powered by Stellar Soroban.

---

## Problem

A sari-sari store owner in Tondo, Manila sells ₱15,000/day in FMCGs (fast-moving consumer goods: sardines, instant noodles, sugar, rice) from a 3sqm stall. She re-orders from her neighborhood distributor via text message, pays cash on delivery, and tracks stock in a paper notebook. When she miscounts and under-reorders, she runs out of her top sellers mid-week and loses 20–30% of daily revenue. She has no proof of payment, no dispute mechanism, and no credit history to grow her store.

## Solution

TindaChain puts her inventory ledger and supplier payments on Stellar. She logs every sale on a mobile app (GCash/Maya login), which calls a Soroban contract to decrement stock. When stock hits her reorder threshold, she funds an on-chain escrow with PHP-pegged anchor tokens in one tap. The supplier delivers and she confirms — the payment atomically releases. No cash disputes. No lost receipts. A verified on-chain payment history that doubles as her credit record.

**Why Stellar specifically:**
- Sub-cent transaction fees make ₱50–₱500 micro-escrows economically viable
- 5-second finality means the supplier gets paid the moment she taps "Delivered"
- Native USDC / PHP anchor token support via Stellar's Anchor network (Coins.ph, GCash Padala)
- Soroban's atomic execution guarantees payment + stock update never get out of sync

---

## Stellar Features Used

| Feature | Usage |
|---|---|
| **Soroban Smart Contracts** | Inventory state, escrow lock/release logic |
| **Custom Token (PHP Anchor)** | GCash/Maya peso ↔ on-chain PHP stablecoin via local anchor |
| **USDC Transfers** | Alternative settlement token for USDC-native suppliers |
| **Trustlines** | Store wallet opts in to the PHP anchor asset before first payment |

---

## Target Users

| | Details |
|---|---|
| **Who** | Sari-sari store operators (predominantly women, 30–55 yrs old) |
| **Where** | Metro Manila barangays: Tondo, Sampaloc, Caloocan, Marikina |
| **Income** | ₱500–₱2,000/day net margin |
| **Behavior** | Daily GCash users; comfortable with QR codes; no bank account required |
| **Suppliers** | Neighborhood distributors & small wholesale stores (Divisoria, Balintawak) |
| **Why they care** | Stop losing revenue from stockouts; build a payment history; no more cash disputes with suki suppliers |

---

## Core MVP Feature (Demo-able in < 2 minutes)

```
Owner logs in → taps "Record Sale" for 42 cans of sardines
      ↓
record_sale("sardines", 42) called on Soroban contract
      ↓
Stock: 50 → 8  (below threshold of 10)
      ↓
App shows: "⚠️ Reorder sardines — tap to fund escrow"
      ↓
Owner taps "Reorder" → create_reorder("sardines", supplier_wallet, PHP_token, 500_PHP)
      ↓
₱500 locked in contract escrow (Stellar transaction: owner → contract)
      ↓
Supplier delivers next morning → owner taps "Confirm Delivery (50 cans)"
      ↓
confirm_delivery("sardines", 50) fires atomically:
  • ₱500 released: contract → supplier wallet
  • Stock updated: 8 + 50 = 58
      ↓
Both parties see finality in 5 seconds. No cash. No receipt. No dispute.
```

---

## Why This Wins

Stellar judges see dozens of DeFi/trading dApps. TindaChain is different: **real users, real pesos, real daily commerce**. Sari-sari stores are the largest retail network in the Philippines (>1 million stores) — this is a massive underserved market where Stellar's low fees and anchor ecosystem create an unfair advantage over any other chain. The MVP is a live demo with tangible, relatable value.

---

## Optional Edge: AI Demand Forecasting

Integrate a lightweight AI layer that watches the on-chain sale history and auto-suggests reorder quantities and timing. *"Based on your last 4 weeks, you'll run out of sardines by Thursday — reorder 80 cans by Tuesday."* This turns the ledger into a business intelligence tool, not just a payment rail.

---

## Project Structure

```
tindachain/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs        ← Soroban smart contract (all on-chain logic)
    └── test.rs       ← 5 unit tests (imported via mod tests in lib.rs)
```

---

## Prerequisites

| Tool | Version |
|---|---|
| Rust | `>=1.74` (install via rustup) |
| `wasm32-unknown-unknown` target | `rustup target add wasm32-unknown-unknown` |
| Soroban CLI | `>=20.0.0` — `cargo install --locked soroban-cli` |
| Stellar Testnet account | Funded via `https://friendbot.stellar.org` |

---

## Build

```bash
# Compile to optimised Wasm
soroban contract build

# Output: target/wasm32-unknown-unknown/release/tindachain.wasm
```

---

## Test

```bash
# Run all 5 unit tests
cargo test

# Run with output (verbose)
cargo test -- --nocapture
```

Expected output:
```
running 5 tests
test tests::test_happy_path_full_reorder_flow ... ok
test tests::test_oversell_panics ... ok
test tests::test_stock_state_reflects_multiple_sales ... ok
test tests::test_supplier_receives_exact_payment_on_delivery ... ok
test tests::test_double_initialize_is_blocked ... ok

test result: ok. 5 passed; 0 failed
```

---

## Deploy to Testnet

```bash
# 1. Configure Stellar testnet identity
soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

soroban config identity generate owner
soroban config identity fund owner --network testnet

# 2. Deploy the contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/tindachain.wasm \
  --source owner \
  --network testnet
# Returns: CONTRACT_ID (save this)

# 3. Initialize the store
soroban contract invoke \
  --id $CONTRACT_ID \
  --source owner \
  --network testnet \
  -- initialize \
  --owner $(soroban config identity address owner)
```

---

## Sample CLI Invocations (MVP Flow)

```bash
# Register sardines: 50 in stock, reorder when ≤ 10
soroban contract invoke --id $CONTRACT_ID --source owner --network testnet \
  -- set_item \
  --item_id sardines \
  --qty 50 \
  --threshold 10

# Record a sale of 42 cans
soroban contract invoke --id $CONTRACT_ID --source owner --network testnet \
  -- record_sale \
  --item_id sardines \
  --qty 42
# Returns: true  ← reorder needed!

# Lock ₱500 for supplier (500 PHP anchor tokens = 500_0000000 stroops)
soroban contract invoke --id $CONTRACT_ID --source owner --network testnet \
  -- create_reorder \
  --item_id sardines \
  --supplier GSUPPLIER_WALLET_ADDRESS \
  --token_address PHP_ANCHOR_CONTRACT_ID \
  --amount 5000000000

# Confirm delivery of 50 cans → payment releases automatically
soroban contract invoke --id $CONTRACT_ID --source owner --network testnet \
  -- confirm_delivery \
  --item_id sardines \
  --received_qty 50

# Check current stock
soroban contract invoke --id $CONTRACT_ID --source owner --network testnet \
  -- get_stock \
  --item_id sardines
# Returns: 58
```

---

## Vision & Purpose

The Philippines has over **1 million sari-sari stores** generating an estimated ₱2.4 trillion in annual retail revenue. Nearly all of it flows through cash, paper notebooks, and informal trust. TindaChain doesn't replace that trust — it records it on a public ledger.

The long-term vision: every confirmed on-chain delivery builds a **merchant credit score**. After 6 months of transaction history, a store owner can apply for micro-financing directly through a Stellar-based lending protocol — collateralised by her verified sales record, not by assets she doesn't have.

This is financial infrastructure for the *actual* Filipino informal economy, built on a chain fast enough and cheap enough to handle ₱15 transactions.

---

## Timeline

| Phase | Scope | Duration |
|---|---|---|
| **Bootcamp MVP** | Soroban contract + CLI demo | 3 days |
| **Alpha** | Mobile PWA (React Native) + GCash/Maya login | 3 weeks |
| **Beta** | PHP anchor integration (Coins.ph) + multi-item dashboard | 6 weeks |
| **Launch** | 10 pilot stores in Tondo / Sampaloc | 3 months |

---
# TindaChain 🏪⛓️

> On-chain inventory management and supplier escrow for Manila's sari-sari store network — powered by Stellar Soroban.

---

## Problem

A sari-sari store owner in Tondo, Manila sells ₱15,000/day in FMCGs (fast-moving consumer goods: sardines, instant noodles, sugar, rice) from a 3sqm stall. She re-orders from her neighborhood distributor via text message, pays cash on delivery, and tracks stock in a paper notebook. When she miscounts and under-reorders, she runs out of her top sellers mid-week and loses 20–30% of daily revenue. She has no proof of payment, no dispute mechanism, and no credit history to grow her store.

## Solution

TindaChain puts her inventory ledger and supplier payments on Stellar. She logs every sale on a mobile app (GCash/Maya login), which calls a Soroban contract to decrement stock. When stock hits her reorder threshold, she funds an on-chain escrow with PHP-pegged anchor tokens in one tap. The supplier delivers and she confirms — the payment atomically releases. No cash disputes. No lost receipts. A verified on-chain payment history that doubles as her credit record.

**Why Stellar specifically:**
- Sub-cent transaction fees make ₱50–₱500 micro-escrows economically viable
- 5-second finality means the supplier gets paid the moment she taps "Delivered"
- Native USDC / PHP anchor token support via Stellar's Anchor network (Coins.ph, GCash Padala)
- Soroban's atomic execution guarantees payment + stock update never get out of sync

---

## Stellar Features Used

| Feature | Usage |
|---|---|
| **Soroban Smart Contracts** | Inventory state, escrow lock/release logic |
| **Custom Token (PHP Anchor)** | GCash/Maya peso ↔ on-chain PHP stablecoin via local anchor |
| **USDC Transfers** | Alternative settlement token for USDC-native suppliers |
| **Trustlines** | Store wallet opts in to the PHP anchor asset before first payment |

---

## Target Users

| | Details |
|---|---|
| **Who** | Sari-sari store operators (predominantly women, 30–55 yrs old) |
| **Where** | Metro Manila barangays: Tondo, Sampaloc, Caloocan, Marikina |
| **Income** | ₱500–₱2,000/day net margin |
| **Behavior** | Daily GCash users; comfortable with QR codes; no bank account required |
| **Suppliers** | Neighborhood distributors & small wholesale stores (Divisoria, Balintawak) |
| **Why they care** | Stop losing revenue from stockouts; build a payment history; no more cash disputes with suki suppliers |

---

## Core MVP Feature (Demo-able in < 2 minutes)

```
Owner logs in → taps "Record Sale" for 42 cans of sardines
      ↓
record_sale("sardines", 42) called on Soroban contract
      ↓
Stock: 50 → 8  (below threshold of 10)
      ↓
App shows: "⚠️ Reorder sardines — tap to fund escrow"
      ↓
Owner taps "Reorder" → create_reorder("sardines", supplier_wallet, PHP_token, 500_PHP)
      ↓
₱500 locked in contract escrow (Stellar transaction: owner → contract)
      ↓
Supplier delivers next morning → owner taps "Confirm Delivery (50 cans)"
      ↓
confirm_delivery("sardines", 50) fires atomically:
  • ₱500 released: contract → supplier wallet
  • Stock updated: 8 + 50 = 58
      ↓
Both parties see finality in 5 seconds. No cash. No receipt. No dispute.
```

---

## Why This Wins

Stellar judges see dozens of DeFi/trading dApps. TindaChain is different: **real users, real pesos, real daily commerce**. Sari-sari stores are the largest retail network in the Philippines (>1 million stores) — this is a massive underserved market where Stellar's low fees and anchor ecosystem create an unfair advantage over any other chain. The MVP is a live demo with tangible, relatable value.

---

## Optional Edge: AI Demand Forecasting

Integrate a lightweight AI layer that watches the on-chain sale history and auto-suggests reorder quantities and timing. *"Based on your last 4 weeks, you'll run out of sardines by Thursday — reorder 80 cans by Tuesday."* This turns the ledger into a business intelligence tool, not just a payment rail.

---

## Project Structure

```
tindachain/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs        ← Soroban smart contract (all on-chain logic)
    └── test.rs       ← 5 unit tests (imported via mod tests in lib.rs)
```

---

## Prerequisites

| Tool | Version |
|---|---|
| Rust | `>=1.74` (install via rustup) |
| `wasm32-unknown-unknown` target | `rustup target add wasm32-unknown-unknown` |
| Soroban CLI | `>=20.0.0` — `cargo install --locked soroban-cli` |
| Stellar Testnet account | Funded via `https://friendbot.stellar.org` |

---

## Build

```bash
# Compile to optimised Wasm
soroban contract build

# Output: target/wasm32-unknown-unknown/release/tindachain.wasm
```

---

## Test

```bash
# Run all 5 unit tests
cargo test

# Run with output (verbose)
cargo test -- --nocapture
```

Expected output:
```
running 5 tests
test tests::test_happy_path_full_reorder_flow ... ok
test tests::test_oversell_panics ... ok
test tests::test_stock_state_reflects_multiple_sales ... ok
test tests::test_supplier_receives_exact_payment_on_delivery ... ok
test tests::test_double_initialize_is_blocked ... ok

test result: ok. 5 passed; 0 failed
```

---

## Deploy to Testnet

```bash
# 1. Configure Stellar testnet identity
soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

soroban config identity generate owner
soroban config identity fund owner --network testnet

# 2. Deploy the contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/tindachain.wasm \
  --source owner \
  --network testnet
# Returns: CONTRACT_ID (save this)

# 3. Initialize the store
soroban contract invoke \
  --id $CONTRACT_ID \
  --source owner \
  --network testnet \
  -- initialize \
  --owner $(soroban config identity address owner)
```

---

## Sample CLI Invocations (MVP Flow)

```bash
# Register sardines: 50 in stock, reorder when ≤ 10
soroban contract invoke --id $CONTRACT_ID --source owner --network testnet \
  -- set_item \
  --item_id sardines \
  --qty 50 \
  --threshold 10

# Record a sale of 42 cans
soroban contract invoke --id $CONTRACT_ID --source owner --network testnet \
  -- record_sale \
  --item_id sardines \
  --qty 42
# Returns: true  ← reorder needed!

# Lock ₱500 for supplier (500 PHP anchor tokens = 500_0000000 stroops)
soroban contract invoke --id $CONTRACT_ID --source owner --network testnet \
  -- create_reorder \
  --item_id sardines \
  --supplier GSUPPLIER_WALLET_ADDRESS \
  --token_address PHP_ANCHOR_CONTRACT_ID \
  --amount 5000000000

# Confirm delivery of 50 cans → payment releases automatically
soroban contract invoke --id $CONTRACT_ID --source owner --network testnet \
  -- confirm_delivery \
  --item_id sardines \
  --received_qty 50

# Check current stock
soroban contract invoke --id $CONTRACT_ID --source owner --network testnet \
  -- get_stock \
  --item_id sardines
# Returns: 58
```

---

## Vision & Purpose

The Philippines has over **1 million sari-sari stores** generating an estimated ₱2.4 trillion in annual retail revenue. Nearly all of it flows through cash, paper notebooks, and informal trust. TindaChain doesn't replace that trust — it records it on a public ledger.

The long-term vision: every confirmed on-chain delivery builds a **merchant credit score**. After 6 months of transaction history, a store owner can apply for micro-financing directly through a Stellar-based lending protocol — collateralised by her verified sales record, not by assets she doesn't have.

This is financial infrastructure for the *actual* Filipino informal economy, built on a chain fast enough and cheap enough to handle ₱15 transactions.

---

## Timeline

| Phase | Scope | Duration |
|---|---|---|
| **Bootcamp MVP** | Soroban contract + CLI demo | 3 days |
| **Alpha** | Mobile PWA (React Native) + GCash/Maya login | 3 weeks |
| **Beta** | PHP anchor integration (Coins.ph) + multi-item dashboard | 6 weeks |
| **Launch** | 10 pilot stores in Tondo / Sampaloc | 3 months |

---

## Deployed Contract Link:
[1] https://stellar.expert/explorer/testnet/tx/af344313961dfe405c2bf713365c4270d20e7923fd1255ef91ff4934cfb2330a
[2] https://lab.stellar.org/smart-contracts/contract-explorer?$=network$id=testnet&label=Testnet&horizonUrl=https:////horizon-testnet.stellar.org&rpcUrl=https:////soroban-testnet.stellar.org&passphrase=Test%20SDF%20Network%20/;%20September%202015;&smartContracts$explorer$contractId=CBR3T3AUT6QOHP4EJXNDD36WZ5JWWJ7LJDVMFIU7W4NNDOLOGKT23ZSQ;;

## License

MIT © 2025 TindaChain. Free to use, fork, and build on.
