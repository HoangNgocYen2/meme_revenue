# meme_revenue

## Project Title
meme_revenue

## Project Description
meme_revenue is a Soroban smart contract that brings fair, transparent royalty
distribution to the meme economy. An original creator registers a meme on-chain;
remixers register their derivatives and declare their royalty share in basis
points. Every tip is automatically split between the original creator and any
remixer according to the recorded percentages, with the resulting balances
claimable on demand. There is no off-chain settlement, no platform middleman,
and no opaque revenue split — just deterministic, on-chain logic that pays
the people who actually made the meme (and the people who improved it).

## Project Vision
To establish a creator-friendly economic layer for viral content on Stellar,
where every successful meme generates ongoing, programmable revenue for the
people who made it and the people who improved it. meme_revenue aims to be a
foundational primitive for the next generation of decentralized creator
economies, fair-launch communities, and on-chain cultural artefacts — a world
in which attribution is permanent, royalties are automatic, and remixing is
rewarded rather than punished.

## Key Features
- On-chain original-meme registration with creator attestation via
  `require_auth`, producing a tamper-evident timestamp and creator address.
- Royalty-bearing remixes with basis-point (0–10000) split configuration,
  so remixers can declare exactly how much of every tip they keep.
- Tip-driven revenue distribution that credits the original creator and the
  remixer in a single atomic transaction, with saturating arithmetic to
  prevent overflow.
- Self-custodial royalty claims: each creator withdraws their own balance
  with their own authorisation — no custodial intermediary.
- Transparent, queryable views for the current royalty split
  (`get_royalty_split`) and the cumulative tip volume (`get_tip_total`) of
  any meme.
- Deterministic, replay-safe logic: no real XLM transfer, so the contract
  can be safely tested and iterated on Stellar Testnet before any mainnet
  deployment.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** content dApp — see `contracts/meme_revenue/src/lib.rs` for the full meme_revenue business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `<to be deployed on Stellar Testnet>`
- **Explorer template:** `https://stellar.expert/explorer/testnet`
- **Screenshot of deployed contract on Stellar Expert:**
  `_(Screenshot of the contract page on Stellar Expert will appear here after deploy.)_`


## Future Scope
- Multi-level remix chains: a derivative of a derivative that propagates the
  royalty split up the full attribution tree, with a configurable number of
  hops and a fallback for deeply nested chains.
- Integration with Stellar native assets and SAC tokens, so tips and royalty
  payouts settle in XLM or custom issued tokens rather than a pure ledger
  entry.
- Off-chain content storage on IPFS or Arweave, with the on-chain
  `content_hash` acting as a tamper-evident pointer to the actual meme file.
- Optional minting of each meme (and its remixes) as a Soroban token so that
  ownership, trading, and royalty flows can be expressed as NFT transfers.
- A browser-based dApp for browsing, tipping, and claiming memes, with
  notifications, social-graph features, and a creator dashboard.
- Governance module allowing creators to update the royalty split or
  transfer the canonical creator address subject to multi-sig approval.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `meme_revenue` (content)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
