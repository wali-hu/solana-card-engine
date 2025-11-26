use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token;

declare_id!("9PznwD37XbYGLsDPfrxumNBBUY1HeBPb4uneRkX3r8vM");

#[program]
pub mod user_card_program {
    use super::*;

    pub fn initialize_user_card(
        ctx: Context<InitializeUserCard>,
        card_type: CardType,
        amount_paid: u64,
        tokens_minted: u64,
    ) -> Result<()> {
        // Transfer the amount_paid from authority to user_card account
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.user_card.to_account_info(),
            },
        );

        // SOL transfer
        msg!(
            "Transferring {} lamports from authority to user_card account",
            amount_paid
        );
        system_program::transfer(cpi_context, amount_paid)?;

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
        msg!("Minting {} tokens to user's token account", tokens_minted);
        token::mint_to(cpi_ctx, tokens_minted)?;

        let acct = &mut ctx.accounts.user_card;
        acct.owner = ctx.accounts.authority.key();
        acct.card_type = card_type;
        acct.amount_paid = amount_paid;
        acct.tokens_minted = tokens_minted;
        acct.status = AccountStatus::Active;
        Ok(())
    }

    // Example: upgrade card (simple)
    pub fn upgrade_card(ctx: Context<ModifyCard>, new_type: CardType) -> Result<()> {
        let acct = &mut ctx.accounts.user_card;
        acct.card_type = new_type;
        Ok(())
    }

    // Example: deactivate
    pub fn deactivate(ctx: Context<ModifyCard>) -> Result<()> {
        let acct = &mut ctx.accounts.user_card;
        acct.status = AccountStatus::Inactive;
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
pub struct ModifyCard<'info> {
    #[account(mut, has_one = owner @ ErrorCode::Unauthorized)]
    pub user_card: Account<'info, UserCardAccount>,

    pub owner: Signer<'info>,
}

#[account]
pub struct UserCardAccount {
    pub owner: Pubkey,         // 32 bytes
    pub card_type: CardType,   // enum
    pub amount_paid: u64,      // 8 bytes
    pub tokens_minted: u64,    // 8 bytes
    pub status: AccountStatus, // enum
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum AccountStatus {
    Active,
    Inactive,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
}
