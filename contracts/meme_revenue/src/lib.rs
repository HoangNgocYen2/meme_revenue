#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, Symbol};

/// On-chain record of an original meme registered by its creator.
#[contracttype]
#[derive(Clone)]
pub struct Meme {
    pub creator: Address,
    pub content_hash: Symbol,
    pub base_price: u32,
    pub created_at: u64,
}

/// On-chain record of a derivative (remix) of an existing meme.
#[contracttype]
#[derive(Clone)]
pub struct Remix {
    pub remixer: Address,
    pub parent_meme_id: Symbol,
    pub content_hash: Symbol,
    pub royalty_bps: u32,
    pub created_at: u64,
}

/// Meme royalty-split contract: original creators and remixers share tip
/// revenue according to the basis points recorded at registration time.
/// All settlement is reflected in the contract's internal ledger; no real
/// asset transfer is performed, which keeps the contract safe to iterate
/// on Stellar Testnet.
#[contract]
pub struct MemeRevenue;

#[contractimpl]
impl MemeRevenue {
    /// Register a brand-new original meme on-chain. The `creator` becomes
    /// the canonical owner and full beneficiary of tips until a remix is
    /// created. `base_price` is a suggested minimum tip expressed in the
    /// smallest unit (informational; no real asset transfer is performed).
    /// The caller (`creator`) must authorise the registration.
    pub fn register_meme(
        env: Env,
        creator: Address,
        meme_id: Symbol,
        content_hash: Symbol,
        base_price: u32,
    ) {
        creator.require_auth();

        let key = Symbol::new(&env, "meme");
        let mut memes: Map<Symbol, Meme> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or(Map::new(&env));
        if memes.contains_key(meme_id.clone()) {
            panic!("meme already registered");
        }

        let meme = Meme {
            creator: creator.clone(),
            content_hash,
            base_price,
            created_at: env.ledger().timestamp(),
        };
        memes.set(meme_id, meme);
        env.storage().instance().set(&key, &memes);
    }

    /// Register a remix (derivative) of an existing meme. `royalty_bps` is
    /// the remixer's share expressed in basis points (0–10000). The parent
    /// creator receives the complementary share on every tip that lands on
    /// this remix. The caller (`remixer`) must authorise the registration
    /// and the parent meme must already be registered.
    pub fn register_remix(
        env: Env,
        remixer: Address,
        parent_meme_id: Symbol,
        remix_id: Symbol,
        content_hash: Symbol,
        royalty_bps: u32,
    ) {
        remixer.require_auth();

        if royalty_bps > 10_000 {
            panic!("royalty_bps must be <= 10000");
        }

        let meme_key = Symbol::new(&env, "meme");
        let memes: Map<Symbol, Meme> = env
            .storage()
            .instance()
            .get(&meme_key)
            .unwrap_or(Map::new(&env));
        if !memes.contains_key(parent_meme_id.clone()) {
            panic!("parent meme not found");
        }

        let remix_key = Symbol::new(&env, "remix");
        let mut remixes: Map<Symbol, Remix> = env
            .storage()
            .instance()
            .get(&remix_key)
            .unwrap_or(Map::new(&env));
        if remixes.contains_key(remix_id.clone()) {
            panic!("remix already registered");
        }

        let remix = Remix {
            remixer: remixer.clone(),
            parent_meme_id: parent_meme_id.clone(),
            content_hash,
            royalty_bps,
            created_at: env.ledger().timestamp(),
        };
        remixes.set(remix_id, remix);
        env.storage().instance().set(&remix_key, &remixes);
    }

    /// Tip a meme (original or remix). The tip is atomically split between
    /// the primary recipient (the creator for originals, the remixer for
    /// derivatives) and the parent creator according to the recorded
    /// `royalty_bps`. Balances accumulate in the royalty ledger and can be
    /// withdrawn via `claim_royalties`. The caller (`tipper`) must authorise
    /// the tip. No real asset transfer is performed; this is intentionally a
    /// ledger-only settlement so the contract can be exercised on Testnet
    /// without funding.
    pub fn tip_meme(env: Env, tipper: Address, meme_id: Symbol, amount: u32) {
        tipper.require_auth();

        if amount == 0 {
            panic!("amount must be > 0");
        }

        let meme_key = Symbol::new(&env, "meme");
        let remix_key = Symbol::new(&env, "remix");
        let memes: Map<Symbol, Meme> = env
            .storage()
            .instance()
            .get(&meme_key)
            .unwrap_or(Map::new(&env));
        let remixes: Map<Symbol, Remix> = env
            .storage()
            .instance()
            .get(&remix_key)
            .unwrap_or(Map::new(&env));

        let (primary, primary_bps, secondary): (Address, u32, Option<Address>) =
            if let Some(meme) = memes.get(meme_id.clone()) {
                (meme.creator, 10_000u32, None)
            } else if let Some(remix) = remixes.get(meme_id.clone()) {
                let parent = memes
                    .get(remix.parent_meme_id.clone())
                    .unwrap_or_else(|| panic!("parent meme missing"));
                (remix.remixer, remix.royalty_bps, Some(parent.creator))
            } else {
                panic!("meme not found");
            };

        let primary_amount = amount.saturating_mul(primary_bps) / 10_000;
        let secondary_amount = amount.saturating_sub(primary_amount);

        Self::credit(&env, &meme_id, &primary, primary_amount);
        if let Some(parent_creator) = secondary {
            Self::credit(&env, &meme_id, &parent_creator, secondary_amount);
        }

        Self::bump_tip_total(&env, &meme_id, amount);
    }

    /// Withdraw the accumulated royalty balance for `creator` on `meme_id`.
    /// Returns the amount claimed (in smallest units) and resets the stored
    /// balance to zero. The caller (`creator`) must authorise the claim.
    /// In a production deployment this would also emit a token transfer;
    /// here the settlement is reflected only in the contract's internal
    /// ledger.
    pub fn claim_royalties(env: Env, creator: Address, meme_id: Symbol) -> u32 {
        creator.require_auth();

        let royalty_key = Symbol::new(&env, "royalty");
        let mut balances: Map<(Symbol, Address), u32> = env
            .storage()
            .instance()
            .get(&royalty_key)
            .unwrap_or(Map::new(&env));

        let key = (meme_id, creator.clone());
        let amount = balances.get(key.clone()).unwrap_or(0);
        if amount == 0 {
            return 0;
        }

        balances.set(key, 0);
        env.storage().instance().set(&royalty_key, &balances);

        amount
    }

    /// Return the primary recipient's royalty share for `meme_id`, in basis
    /// points (0–10000). A return value of `10000` indicates that the
    /// creator takes the full tip (i.e. the id resolves to an original);
    /// any other value is the remixer share recorded on a derivative.
    /// Returns `0` for unknown ids.
    pub fn get_royalty_split(env: Env, meme_id: Symbol) -> u32 {
        let meme_key = Symbol::new(&env, "meme");
        let remix_key = Symbol::new(&env, "remix");
        let memes: Map<Symbol, Meme> = env
            .storage()
            .instance()
            .get(&meme_key)
            .unwrap_or(Map::new(&env));
        if memes.contains_key(meme_id.clone()) {
            return 10_000;
        }
        let remixes: Map<Symbol, Remix> = env
            .storage()
            .instance()
            .get(&remix_key)
            .unwrap_or(Map::new(&env));
        remixes.get(meme_id).map(|r| r.royalty_bps).unwrap_or(0)
    }

    /// Return the cumulative tip volume (in smallest units) recorded for
    /// `meme_id`. Returns `0` for unknown ids.
    pub fn get_tip_total(env: Env, meme_id: Symbol) -> u32 {
        let totals: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "tip_total"))
            .unwrap_or(Map::new(&env));
        totals.get(meme_id).unwrap_or(0)
    }

    // -------- internal helpers --------

    /// Add `amount` to the stored royalty balance for `(meme_id, recipient)`.
    fn credit(env: &Env, meme_id: &Symbol, recipient: &Address, amount: u32) {
        if amount == 0 {
            return;
        }
        let royalty_key = Symbol::new(env, "royalty");
        let key = (meme_id.clone(), recipient.clone());
        let mut balances: Map<(Symbol, Address), u32> = env
            .storage()
            .instance()
            .get(&royalty_key)
            .unwrap_or(Map::new(env));
        let current = balances.get(key.clone()).unwrap_or(0);
        balances.set(key, current.saturating_add(amount));
        env.storage().instance().set(&royalty_key, &balances);
    }

    /// Add `delta` to the cumulative tip total for `meme_id`.
    fn bump_tip_total(env: &Env, meme_id: &Symbol, delta: u32) {
        let totals_key = Symbol::new(env, "tip_total");
        let mut totals: Map<Symbol, u32> = env
            .storage()
            .instance()
            .get(&totals_key)
            .unwrap_or(Map::new(env));
        let current = totals.get(meme_id.clone()).unwrap_or(0);
        totals.set(meme_id.clone(), current.saturating_add(delta));
        env.storage().instance().set(&totals_key, &totals);
    }
}
