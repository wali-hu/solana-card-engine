use anchor_lang::prelude::*;

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
        space = UserCardAccount::SPACE
    )]
    pub user_card: Account<'info, UserCardAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

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
