use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token;

declare_id!("9PznwD37XbYGLsDPfrxumNBBUY1HeBPb4uneRkX3r8vM");

const ADMIN_PUBKEY: &str = "Fskji1sm9H8QwZBGmuRTTie6B111RhCfLtbALMaNRkt";

#[program]
pub mod user_card_program {
    use super::*;

    pub fn initialize_user_card(
        ctx: Context<InitializeUserCard>,
        card_type: CardType,
        amount_paid: u64,
    ) -> Result<()> {
        // 1. PRICE Logic
        let expected_price: u64 = match card_type {
            CardType::Bronze => 125_000_000,     // 0.125 SOL
            CardType::Silver => 250_000_000,     // 0.25 SOL
            CardType::Gold => 500_000_000,       // 0.5 SOL
            CardType::Platinum => 1_000_000_000, // 1 SOL
        };

        // 2. Validation: Check if amount_paid is sufficient
        if amount_paid < expected_price {
            return Err(ErrorCode::InsufficientPayment.into());
        }

        // 3. Calculation (Internal Source of Truth)
        // Jitne SOL diye, utne hi tokens milenge (1:1 Ratio)
        let tokens_to_mint: u64 = amount_paid;

        // --- SOL TRANSFER ---
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.user_card.to_account_info(),
            },
        );

        // Transfer the amount_paid from authority to user_card account
        msg!(
            "Transferring {} lamports from authority to user_card account",
            amount_paid
        );
        system_program::transfer(cpi_context, amount_paid)?;

        // --- TOKEN MINTING ---
        let bump = ctx.bumps.user_card;
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"user_card",                        // seed
            ctx.accounts.authority.key.as_ref(), // user ki public key
            &[bump],                             // Bump seed
        ]];

        // Mint the tokens to user's token account
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.card_mint.to_account_info(), // jo token mint karna hai (Gold/Silver etc)
                to: ctx.accounts.user_token_account.to_account_info(), // User ka wallet jahan token jayega
                authority: ctx.accounts.user_card.to_account_info(),   // Authority (Hamara PDA)
            },
            signer_seeds,
        );

        // Mint tokens
        msg!("Minting {} tokens to user's token account", tokens_to_mint);
        token::mint_to(cpi_ctx, tokens_to_mint)?;

        // --- DATA SAVING ---
        let acct = &mut ctx.accounts.user_card;
        acct.owner = ctx.accounts.authority.key();
        acct.card_type = card_type;
        acct.amount_paid = amount_paid;
        acct.tokens_minted = tokens_to_mint;
        //acct.status = AccountStatus::Active;
        Ok(())
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>, amount: u64) -> Result<()> {
        let user_card = &ctx.accounts.user_card;
        let admin = &ctx.accounts.admin;

        // Validation: Kya account mein itne paise hain?
        let current_balance = user_card.to_account_info().lamports();
        if current_balance < amount {
             return Err(ErrorCode::InsufficientFunds.into());
        }

        let user_balance = user_card.to_account_info().lamports();
        let new_user_balance = user_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::InsufficientFunds)?;

        let admin_balance = admin.to_account_info().lamports();
        let new_admin_balance = admin_balance
            .checked_add(amount)
            .ok_or(ErrorCode::ArithmeticError)?;

        // ⚠️ TRANSFER LOGIC
        **user_card.to_account_info().try_borrow_mut_lamports()? = new_user_balance;
        **admin.to_account_info().try_borrow_mut_lamports()? = new_admin_balance;

        msg!("Withdraw success! Sent {} lamports to admin.", amount);
        Ok(())
    }

}

#[derive(Accounts)]
pub struct InitializeUserCard<'info> {
    #[account(
        init,
        payer = authority,
        space = UserCardAccount::SPACE,
        seeds = [b"user_card", authority.key().as_ref()],
        bump,
    )]
    pub user_card: Account<'info, UserCardAccount>,

    #[account(mut)]
    pub authority: Signer<'info>, // User

    /// CHECK: Mint account jisse token mint hoga (SPL Mint)
    #[account(mut)]
    pub card_mint: Account<'info, token::Mint>,

    /// CHECK: User's token account jahan token jayega (SPL Token Account)
    #[account(mut)]
    pub user_token_account: Account<'info, token::TokenAccount>,

    /// Required SPL token program
    pub token_program: Program<'info, token::Token>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut)]
    pub user_card: Account<'info, UserCardAccount>,

    #[account(
        mut,
        address = ADMIN_PUBKEY.parse::<Pubkey>().unwrap() @ ErrorCode::Unauthorized
    )]
    pub admin: Signer<'info>,
}


#[account]
pub struct UserCardAccount {
    pub owner: Pubkey,         // 32 bytes
    pub card_type: CardType,   // enum
    pub amount_paid: u64,      // 8 bytes
    pub tokens_minted: u64,    // 8 bytes
    //pub status: AccountStatus, // enum
}

impl UserCardAccount {
    // 8 (discriminator) + 32 (Pubkey) + 1 (CardType enum) + 8 (amount_paid)
    // + 8 (tokens_minted) + 1 (status enum) = 58
    pub const SPACE: usize = 58;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum CardType {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
// pub enum AccountStatus {
//     Active,
//     Inactive,
// }

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Insufficient payment for selected card type")]
    InsufficientPayment,

    #[msg("Insufficient funds in user card account")]
    InsufficientFunds,

    #[msg("Arithmetic Overflow/Underflow")]
    ArithmeticError,
}
