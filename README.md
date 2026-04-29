# 👻 GhostMarket

![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?style=flat&logo=solana)
![Anchor](https://img.shields.io/badge/Anchor-0.30.1-blue?style=flat)
![Next.js](https://img.shields.io/badge/Next.js-16-black?style=flat)
![MagicBlock](https://img.shields.io/badge/MagicBlock-PER-green?style=flat)
![License](https://img.shields.io/badge/license-MIT-brightgreen?style=flat)

> Private farmer-to-buyer sealed-bid marketplace on Solana.
> Fair prices for farmers. Private bids for buyers. No middlemen.

---

## 🏆 Challenge Submission Details

| Field | Value |
|---|---|
| **Project Name** | GhostMarket |
| **Repository** | https://github.com/sadbunny3000/ghostmarket |
| **Live Demo** | https://ghostmarket-three.vercel.app |
| **Program ID** | iY3mhchKCD4zpxFRjg7DXcpY4t8kjJ8LFmsrsKnsE6F |
| **Network** | Solana Devnet |
| **Hackathon** | Solana Colosseum Frontier |
| **Developer** | Natangwe Martin |

---

## 🌾 Project Overview

GhostMarket is a sealed-bid agricultural marketplace built on Solana. Farmers in developing markets are forced to sell through brokers who can see all bids — enabling buyer collusion and price suppression. GhostMarket fixes this with cryptographic bid privacy and trustless on-chain settlement. No middleman. No manipulation.

Bids are sealed using a commit-reveal scheme, architectured to upgrade directly to MagicBlock Private Ephemeral Rollups (PER) for full production privacy.

### ✨ Key Highlights

- **Private Sealed Bids:** SHA-256 commitment hashes hide real bid amounts on-chain during the auction
- **Trustless Settlement:** Smart contract automatically selects the highest verified bid
- **Bid Refunds:** Losing bidders reclaim their escrowed SOL after finalization
- **MagicBlock PER Ready:** delegate/undelegate hooks in place for production upgrade
- **Multi-Wallet Support:** Phantom, Solflare, Backpack, Coinbase Wallet, Torus
- **Live Frontend:** 3-tab UI — List Product, Place Bid, View Results

---

## ⚡ Quick Start

```bash
# Clone
git clone https://github.com/sadbunny3000/ghostmarket.git
cd ghostmarket

# Build smart contract
anchor build
anchor test --skip-deploy

# Run frontend
cd frontend
npm install
npm run dev
# Open http://localhost:3000
```

---

## 🔐 How It Works
### Privacy Mechanism

**Live MVP — Commit-Reveal Scheme**
- Buyer computes: `commitment = SHA256(amount + nonce)`
- Only the hash is stored on-chain during bidding
- After deadline, buyer reveals real amount and nonce
- Contract verifies hash and records highest bid

**Production Upgrade — MagicBlock Private Ephemeral Rollups**
- Listing account delegated to private rollup
- All bids execute inside the rollup — never visible on Solana base layer
- Only the final settlement result returns to mainnet

---

## 🏗️ Architecture
---

## 🛠️ Tech Stack

| Layer | Technology | Version |
|---|---|---|
| Blockchain | Solana | Devnet |
| Smart Contract | Rust + Anchor | 0.30.1 |
| Privacy | Commit-Reveal → MagicBlock PER | — |
| Frontend | Next.js + TailwindCSS | 16 |
| Wallets | Phantom, Solflare, Backpack, Coinbase | Latest |
| Deployment | Vercel | — |

---

## 📁 Project Structure
---

## 📋 Smart Contract Instructions

| Instruction | Caller | Description |
|---|---|---|
| `create_listing` | Farmer | Creates listing with min price and deadline |
| `place_bid` | Buyer | Submits sealed hash and locks SOL in escrow |
| `reveal_bid` | Buyer | Reveals real bid after deadline |
| `finalize_auction` | Farmer | Closes auction and declares winner |
| `refund_losing_bid` | Loser | Returns escrowed SOL to losing bidders |

---

## 🔵 MagicBlock Integration

```rust
// create_listing — delegate to Private Ephemeral Rollup
// delegate_account(&listing.key(), &rollup_config, &payer)?;

// finalize_auction — settle rollup and private payment
// undelegate_account(&listing.key(), &rollup_config)?;
// magicblock_payments::transfer_private(...)?;
```

---

## 📄 License

MIT — Built by Natangwe Martin for Solana Colosseum Frontier Hackathon
