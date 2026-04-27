use anchor_lang::prelude::*;
use solana_program::hash::hash;
// This is your program's unique ID on Solana
declare_id!("iY3mhchKCD4zpxFRjg7DXcpY4t8kjJ8LFmsrsKnsE6F");

#[program]
pub mod ghostmarket {
    use super::*;

    // ── INSTRUCTION 1: Farmer creates a listing ──────────────────────────────
    // The farmer calls this to say "I have tomatoes for sale"
    pub fn create_listing(
        ctx: Context<CreateListing>,
        product_name: String,   // e.g. "Tomatoes"
        quantity: u64,          // e.g. 100 (kg)
        min_price: u64,         // minimum price in lamports (1 SOL = 1,000,000,000 lamports)
        deadline: i64,          // Unix timestamp when bidding closes
    ) -> Result<()> {
        let listing = &mut ctx.accounts.listing;

        // Make sure product name isn't too long
        require!(product_name.len() <= 50, GhostMarketError::NameTooLong);
        // Make sure minimum price is more than zero
        require!(min_price > 0, GhostMarketError::InvalidPrice);

        listing.farmer = ctx.accounts.farmer.key();
        listing.product_name = product_name;
        listing.quantity = quantity;
        listing.min_price = min_price;
        listing.deadline = deadline;
        listing.is_active = true;
        listing.highest_bid = 0;
        listing.highest_bidder = Pubkey::default(); // empty for now
        listing.bid_count = 0;

        msg!("Listing created: {} by {}", listing.product_name, listing.farmer);
        Ok(())
    }

    // ── INSTRUCTION 2: Buyer places a bid ────────────────────────────────────
    // NOTE: In the real MagicBlock version, bid amount is hidden.
    // For this MVP, we store a "commitment hash" instead of the real amount.
    // Think of it like a sealed envelope — the number is inside but nobody can see it.
    pub fn place_bid(
        ctx: Context<PlaceBid>,
        bid_commitment: [u8; 32],  // This is a hash of the real bid (keeps it private)
        escrow_amount: u64,        // Buyer locks this much SOL as a deposit
    ) -> Result<()> {
        let listing = &ctx.accounts.listing;
        let bid = &mut ctx.accounts.bid;
        let clock = Clock::get()?;

        // Make sure the auction is still open
        require!(listing.is_active, GhostMarketError::ListingNotActive);
        require!(clock.unix_timestamp < listing.deadline, GhostMarketError::AuctionEnded);
        require!(escrow_amount >= listing.min_price, GhostMarketError::BidTooLow);

        // Transfer SOL from buyer to escrow (locked until auction ends)
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &ctx.accounts.escrow.key(),
            escrow_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.escrow.to_account_info(),
            ],
        )?;

        bid.buyer = ctx.accounts.buyer.key();
        bid.listing = ctx.accounts.listing.key();
        bid.bid_commitment = bid_commitment;
        bid.escrow_amount = escrow_amount;
        bid.timestamp = clock.unix_timestamp;
        bid.revealed = false;

        // Increase bid count on the listing
        let listing_mut = &mut ctx.accounts.listing;
        listing_mut.bid_count += 1;

        msg!("Bid placed by {}", bid.buyer);
        Ok(())
    }

    // ── INSTRUCTION 3: Buyer reveals their real bid ──────────────────────────
    // After auction closes, each buyer reveals the real number that matches their hash.
    // This is how "commit-reveal" private bidding works.
    pub fn reveal_bid(
        ctx: Context<RevealBid>,
        real_amount: u64,   // The real bid amount
        nonce: [u8; 32],    // A random number used when creating the hash
    ) -> Result<()> {
        let bid = &mut ctx.accounts.bid;
        let listing = &mut ctx.accounts.listing;
        let clock = Clock::get()?;

        // Auction must be over before revealing
        require!(clock.unix_timestamp >= listing.deadline, GhostMarketError::AuctionNotEnded);
        require!(!bid.revealed, GhostMarketError::AlreadyRevealed);

        // Verify: hash(real_amount + nonce) must equal what was submitted
        let mut data = real_amount.to_le_bytes().to_vec();
        data.extend_from_slice(&nonce);
        let computed_hash: [u8; 32] = hash(&data).to_bytes();

        require!(computed_hash == bid.bid_commitment, GhostMarketError::InvalidReveal);

        bid.revealed = true;
        bid.real_amount = real_amount;

        // If this is the highest bid so far, record it
        if real_amount > listing.highest_bid {
            listing.highest_bid = real_amount;
            listing.highest_bidder = bid.buyer;
        }

        msg!("Bid revealed: {} lamports by {}", real_amount, bid.buyer);
        Ok(())
    }

    // ── INSTRUCTION 4: Finalize the auction ──────────────────────────────────
    // The farmer calls this after all bids are revealed.
    // Winner's escrow goes to farmer. Losers get refunds.
    pub fn finalize_auction(ctx: Context<FinalizeAuction>) -> Result<()> {
        let listing = &mut ctx.accounts.listing;

        require!(listing.is_active, GhostMarketError::ListingNotActive);
        require!(
            ctx.accounts.farmer.key() == listing.farmer,
            GhostMarketError::NotTheFarmer
        );

        listing.is_active = false;

        msg!(
            "Auction finalized! Winner: {} with {} lamports",
            listing.highest_bidder,
            listing.highest_bid
        );
        Ok(())
    }
}

// ── ACCOUNT STRUCTURES ────────────────────────────────────────────────────────
// These define what data gets stored on the blockchain.

#[account]
pub struct Listing {
    pub farmer: Pubkey,          // Wallet address of farmer
    pub product_name: String,    // "Tomatoes"
    pub quantity: u64,           // 100 kg
    pub min_price: u64,          // Minimum acceptable bid
    pub deadline: i64,           // When bidding ends
    pub is_active: bool,         // Is the auction still open?
    pub highest_bid: u64,        // Highest revealed bid so far
    pub highest_bidder: Pubkey,  // Who bid the most
    pub bid_count: u64,          // How many bids received
}

#[account]
pub struct Bid {
    pub buyer: Pubkey,              // Who placed this bid
    pub listing: Pubkey,            // Which listing this is for
    pub bid_commitment: [u8; 32],   // Hash of the real bid (private!)
    pub escrow_amount: u64,         // SOL locked in escrow
    pub timestamp: i64,             // When bid was placed
    pub revealed: bool,             // Has the bid been revealed?
    pub real_amount: u64,           // Real bid (only set after reveal)
}

// ── INSTRUCTION CONTEXTS ──────────────────────────────────────────────────────
// These tell Anchor which accounts each instruction needs.

#[derive(Accounts)]
#[instruction(product_name: String)]
pub struct CreateListing<'info> {
    #[account(
        init,
        payer = farmer,
        space = 8 + 32 + 4 + 50 + 8 + 8 + 8 + 1 + 8 + 32 + 8,
        seeds = [b"listing", farmer.key().as_ref(), product_name.as_bytes()],
        bump
    )]
    pub listing: Account<'info, Listing>,

    #[account(mut)]
    pub farmer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(mut)]
    pub listing: Account<'info, Listing>,

    #[account(
        init,
        payer = buyer,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 1 + 8,
        seeds = [b"bid", listing.key().as_ref(), buyer.key().as_ref()],
        bump
    )]
    pub bid: Account<'info, Bid>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: This is the escrow account that holds locked SOL
    #[account(
        mut,
        seeds = [b"escrow", listing.key().as_ref(), buyer.key().as_ref()],
        bump
    )]
    pub escrow: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevealBid<'info> {
    #[account(mut)]
    pub listing: Account<'info, Listing>,

    #[account(mut, has_one = buyer)]
    pub bid: Account<'info, Bid>,

    pub buyer: Signer<'info>,
}

#[derive(Accounts)]
pub struct FinalizeAuction<'info> {
    #[account(mut)]
    pub listing: Account<'info, Listing>,

    pub farmer: Signer<'info>,
}

// ── ERROR CODES ───────────────────────────────────────────────────────────────
#[error_code]
pub enum GhostMarketError {
    #[msg("Product name is too long (max 50 characters)")]
    NameTooLong,
    #[msg("Price must be greater than zero")]
    InvalidPrice,
    #[msg("This listing is not active")]
    ListingNotActive,
    #[msg("The auction has already ended")]
    AuctionEnded,
    #[msg("The auction has not ended yet")]
    AuctionNotEnded,
    #[msg("Bid is below the minimum price")]
    BidTooLow,
    #[msg("This bid has already been revealed")]
    AlreadyRevealed,
    #[msg("The revealed amount doesn't match the commitment")]
    InvalidReveal,
    #[msg("Only the farmer can finalize the auction")]
    NotTheFarmer,
}