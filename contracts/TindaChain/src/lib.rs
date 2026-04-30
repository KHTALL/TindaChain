//! TindaChain — Sari-Sari Store Inventory & Supplier Escrow Contract
//!
//! Tests are in src/test.rs — included below via mod declaration.
//!
//! MVP Flow:
//!   Owner records a sale → stock decrements → if below threshold, flags reorder
//!   Owner funds escrow → supplier delivers → owner confirms → payment releases
//!
//! Deployed on Stellar Testnet. Payment token: USDC or PHP-pegged anchor asset.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, Symbol, token,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

/// All keys used in persistent contract storage.
/// Keyed by item Symbol for per-product stock/threshold/escrow data.
#[contracttype]
pub enum DataKey {
    /// Address of the store owner (set once on initialize)
    Owner,
    /// Current stock quantity for an item  (DataKey::Stock(item_id) → u32)
    Stock(Symbol),
    /// Reorder threshold for an item       (DataKey::Threshold(item_id) → u32)
    Threshold(Symbol),
    /// Active escrow details for an item   (DataKey::Escrow(item_id) → EscrowInfo)
    Escrow(Symbol),
}

// ─── Data Structures ─────────────────────────────────────────────────────────

/// Holds everything needed to release an escrowed payment to a supplier.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct EscrowInfo {
    /// Supplier wallet (receives payment on delivery confirmation)
    pub supplier: Address,
    /// Amount locked in escrow (in token's smallest denomination, e.g. stroops)
    pub amount: i128,
    /// Stellar asset contract address (USDC or PHP anchor token)
    pub token: Address,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct TindaChain;

#[contractimpl]
impl TindaChain {

    // ── Setup ────────────────────────────────────────────────────────────────

    /// Initialize the store. Can only be called once.
    /// The owner is the sari-sari store operator's Stellar wallet.
    pub fn initialize(env: Env, owner: Address) {
        // Prevent re-initialization — store is immutably owned once set
        if env.storage().instance().has(&DataKey::Owner) {
            panic!("Contract already initialized");
        }
        env.storage().instance().set(&DataKey::Owner, &owner);
    }

    /// Register or restock an item with an opening quantity and a reorder threshold.
    /// Example: sardines → qty 50, threshold 10 (reorder when ≤ 10 left)
    pub fn set_item(env: Env, item_id: Symbol, qty: u32, threshold: u32) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("Contract not initialized");
        // Only the store owner may manage inventory
        owner.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Stock(item_id.clone()), &qty);
        env.storage()
            .instance()
            .set(&DataKey::Threshold(item_id), &threshold);
    }

    // ── Sales & Reorder Flow ─────────────────────────────────────────────────

    /// Record a sale: decrement stock by `qty` sold.
    /// Returns `true` if remaining stock is at or below the reorder threshold,
    /// signalling the owner that a supplier reorder should be initiated.
    ///
    /// On-chain event: stock storage updated atomically — no partial states.
    pub fn record_sale(env: Env, item_id: Symbol, qty: u32) -> bool {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("Contract not initialized");
        // Require the store owner's signature to prevent unauthorized writes
        owner.require_auth();

        let current: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Stock(item_id.clone()))
            .unwrap_or(0);

        // Hard stop — cannot sell what you don't have
        if current < qty {
            panic!("Insufficient stock");
        }

        let new_qty = current - qty;
        env.storage()
            .instance()
            .set(&DataKey::Stock(item_id.clone()), &new_qty);

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold(item_id))
            .unwrap_or(0);

        // true → caller should trigger create_reorder next
        new_qty <= threshold
    }

    /// Lock payment in escrow for a supplier reorder.
    /// Transfers `amount` of `token_address` from owner → contract address.
    /// Payment is frozen until confirm_delivery is called.
    ///
    /// Why Stellar: near-zero fees make locking small peso amounts (₱50–₱500)
    /// economically viable — impossible on Ethereum or Solana for micro-commerce.
    pub fn create_reorder(
        env: Env,
        item_id: Symbol,
        supplier: Address,
        token_address: Address,
        amount: i128,
    ) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("Contract not initialized");
        owner.require_auth();

        // Move funds from owner wallet → this contract (held in escrow)
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&owner, &env.current_contract_address(), &amount);

        // Record escrow so confirm_delivery knows who to pay and how much
        let escrow = EscrowInfo {
            supplier,
            amount,
            token: token_address,
        };
        env.storage()
            .instance()
            .set(&DataKey::Escrow(item_id), &escrow);
    }

    /// Confirm supplier delivery: release escrowed payment and update stock.
    /// `received_qty` is added to the item's current stock count.
    ///
    /// On-chain: atomic — payment fires and stock updates in the same transaction.
    /// If this call fails, no partial state is written (Soroban ACID guarantee).
    pub fn confirm_delivery(env: Env, item_id: Symbol, received_qty: u32) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("Contract not initialized");
        owner.require_auth();

        // Retrieve escrow record — panics cleanly if no reorder was created
        let escrow: EscrowInfo = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(item_id.clone()))
            .expect("No active escrow for this item");

        // Release payment: contract → supplier wallet
        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.supplier,
            &escrow.amount,
        );

        // Add delivered goods to on-hand stock
        let current: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Stock(item_id.clone()))
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::Stock(item_id.clone()), &(current + received_qty));

        // Clear escrow entry — prevents double-payment
        env.storage()
            .instance()
            .remove(&DataKey::Escrow(item_id));
    }

    // ── Read-Only Queries ────────────────────────────────────────────────────

    /// Return the current stock count for an item. Returns 0 if unknown.
    pub fn get_stock(env: Env, item_id: Symbol) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Stock(item_id))
            .unwrap_or(0)
    }

    /// Return the store owner's address.
    pub fn get_owner(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("Contract not initialized")
    }

    /// Return active escrow info for an item (useful for UI display).
    pub fn get_escrow(env: Env, item_id: Symbol) -> Option<EscrowInfo> {
        env.storage()
            .instance()
            .get(&DataKey::Escrow(item_id))
    }
}

#[cfg(test)]
mod test;
