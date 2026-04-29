use anchor_lang::prelude::*;
use solana_program::hash::hash;

declare_id!("iY3mhchKCD4zpxFRjg7DXcpY4t8kjJ8LFmsrsKnsE6F");

#[program]
pub mod ghostmarket {
    use super::*;

    pub fn create_listing(
        ctx: Context<CreateListing>,
        product_name: String,
        quantity: u64,
        min_price: u64,
        deadline: i64,
    ) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        require!(product_name.len() <= 50, GhostMarketError::NameTooLong);
        require!(min_price > 0, GhostMarketError::InvalidPrice);
        listing.farmer = ctx.accounts.farmer.key();
        listing.product_name = product_name.clone();
        listing.quantity = quantity;
        listing.min_price = min_price;
        listing.deadline = deadline;
        listing.is_active = true;
        listing.highest_bid = 0;
        listing.highest_bidder = Pubkey::default();
        listing.bid_count = 0;
        msg!("Listing created: {}", product_name);
        Ok(())
    }

    pub fn place_bid(
        ctx: Context<PlaceBid>,
        bid_commitment: [u8; 32],
        escrow_amount: u64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        {
            let listing = &ctx.accounts.listing;
            require!(listing.is_active, GhostMarketError::ListingNotActive);
            require!(clock.unix_timestamp < listing.deadline, GhostMarketError::AuctionEnded);
            require!(escrow_amount >= listing.min_price, GhostMarketError::BidTooLow);
        }
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
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        let bid = &mut ctx.accounts.bid;
        bid.buyer = ctx.accounts.buyer.key();
        bid.listing = ctx.accounts.listing.key();
        bid.bid_commitment = bid_commitment;
        bid.escrow_amount = escrow_amount;
        bid.timestamp = clock.unix_timestamp;
        bid.revealed = false;
        bid.real_amount = 0;
        bid.refunded = false;
        let listing = &mut ctx.accounts.listing;
        listing.bid_count += 1;
        msg!("Bid placed by {}", bid.buyer);
        Ok(())
    }

    pub fn reveal_bid(
        ctx: Context<RevealBid>,
        real_amount: u64,
        nonce: [u8; 32],
    ) -> Result<()> {
        let clock = Clock::get()?;
        {
            let listing = &ctx.accounts.listing;
            require!(clock.unix_timestamp >= listing.deadline, GhostMarketError::AuctionNotEnded);
        }
        {
            let bid = &ctx.accounts.bid;
            require!(!bid.revealed, GhostMarketError::AlreadyRevealed);
            let mut data = real_amount.to_le_bytes().to_vec();
            data.extend_from_slice(&nonce);
            let computed: [u8; 32] = hash(&data).to_bytes();
            require!(computed == bid.bid_commitment, GhostMarketError::InvalidReveal);
        }
        let bid = &mut ctx.accounts.bid;
        bid.revealed = true;
        bid.real_amount = real_amount;
        let listing = &mut ctx.accounts.listing;
        if real_amount > listing.highest_bid {
            listing.highest_bid = real_amount;
            listing.highest_bidder = bid.buyer;
        }
        msg!("Bid revealed: {} lamports", real_amount);
        Ok(())
    }

    pub fn finalize_auction(ctx: Context<FinalizeAuction>) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        require!(listing.is_active, GhostMarketError::ListingNotActive);
        require!(ctx.accounts.farmer.key() == listing.farmer, GhostMarketError::NotTheFarmer);
        listing.is_active = false;
        msg!("Auction finalized! Winner: {} with {} lamports", listing.highest_bidder, listing.highest_bid);
        Ok(())
    }

    pub fn refund_losing_bid(ctx: Context<RefundLosingBid>) -> Result<()> {
        {
            let listing = &ctx.accounts.listing;
            let bid = &ctx.accounts.bid;
            require!(!listing.is_active, GhostMarketError::AuctionNotEnded);
            require!(bid.revealed, GhostMarketError::BidNotRevealed);
            require!(!bid.refunded, GhostMarketError::AlreadyRefunded);
            require!(bid.buyer != listing.highest_bidder, GhostMarketError::WinnerCannotRefund);
        }
        let refund_amount = ctx.accounts.bid.escrow_amount;
        ctx.accounts.bid.refunded = true;
        **ctx.accounts.escrow.try_borrow_mut_lamports()? -= refund_amount;
        **ctx.accounts.buyer.try_borrow_mut_lamports()? += refund_amount;
        msg!("Refund sent: {} lamports", refund_amount);
        Ok(())
    }
}

#[account]
pub struct Listing {
    pub farmer: Pubkey,
    pub product_name: String,
    pub quantity: u64,
    pub min_price: u64,
    pub deadline: i64,
    pub is_active: bool,
    pub highest_bid: u64,
    pub highest_bidder: Pubkey,
    pub bid_count: u64,
}

#[account]
pub struct Bid {
    pub buyer: Pubkey,
    pub listing: Pubkey,
    pub bid_commitment: [u8; 32],
    pub escrow_amount: u64,
    pub timestamp: i64,
    pub revealed: bool,
    pub real_amount: u64,
    pub refunded: bool,
}

#[derive(Accounts)]
#[instruction(product_name: String)]
pub struct CreateListing<'info> {
    #[account(
        init, payer = farmer,
        space = 8 + 32 + (4 + 50) + 8 + 8 + 8 + 1 + 8 + 32 + 8,
        seeds = [b"listing", farmer.key().as_ref(), product_name.as_bytes()], bump
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
        init, payer = buyer,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 1 + 8 + 1,
        seeds = [b"bid", listing.key().as_ref(), buyer.key().as_ref()], bump
    )]
    pub bid: Account<'info, Bid>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: escrow PDA
    #[account(mut, seeds = [b"escrow", listing.key().as_ref(), buyer.key().as_ref()], bump)]
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

#[derive(Accounts)]
pub struct RefundLosingBid<'info> {
    pub listing: Account<'info, Listing>,
    #[account(mut, has_one = buyer)]
    pub bid: Account<'info, Bid>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: escrow PDA
    #[account(mut, seeds = [b"escrow", listing.key().as_ref(), buyer.key().as_ref()], bump)]
    pub escrow: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum GhostMarketError {
    #[msg("Product name too long")]
    NameTooLong,
    #[msg("Price must be greater than zero")]
    InvalidPrice,
    #[msg("Listing is not active")]
    ListingNotActive,
    #[msg("Auction has ended")]
    AuctionEnded,
    #[msg("Auction has not ended yet")]
    AuctionNotEnded,
    #[msg("Bid is below minimum price")]
    BidTooLow,
    #[msg("Bid already revealed")]
    AlreadyRevealed,
    #[msg("Bid not yet revealed")]
    BidNotRevealed,
    #[msg("Invalid reveal")]
    InvalidReveal,
    #[msg("Only the farmer can finalize")]
    NotTheFarmer,
    #[msg("Already refunded")]
    AlreadyRefunded,
    #[msg("Winner cannot refund")]
    WinnerCannotRefund,
}
