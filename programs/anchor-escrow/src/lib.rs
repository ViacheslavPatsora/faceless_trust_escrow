use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, spl_token::instruction::AuthorityType, CloseAccount, Mint, SetAuthority, Token,
    TokenAccount, Transfer,
};

declare_id!("6GmDMEb2ETfCUmFDcmhHDpyeEy4AeMeA2unRowUrt3eo");

#[program]
pub mod anchor_escrow {
    use super::*;

    const AUTHORITY_SEED: &[u8] = b"authority";

    pub fn initialize(
        ctx: Context<Initialize>,
        random_seed: u64,
        initializer_amount: [u64; 5],
    ) -> Result<()> {
        ctx.accounts.escrow_state.initializer_key = *ctx.accounts.initializer.key;
        ctx.accounts.escrow_state.initializer_deposit_token_account = *ctx
            .accounts
            .initializer_deposit_token_account
            .to_account_info()
            .key;
        ctx.accounts.escrow_state.taker = *ctx.accounts.taker.key;
        ctx.accounts.escrow_state.initializer_amount = initializer_amount;
        ctx.accounts.escrow_state.random_seed = random_seed;
        ctx.accounts.escrow_state.admin1 = ctx.accounts.admin_state.admin1;
        ctx.accounts.escrow_state.admin2 = ctx.accounts.admin_state.admin2;
        ctx.accounts.escrow_state.resolver = ctx.accounts.admin_state.resolver;
        ctx.accounts.escrow_state.dispute_status = false;

        let (vault_authority, _vault_authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED], ctx.program_id);

        token::set_authority(
            ctx.accounts.into_set_authority_context(),
            AuthorityType::AccountOwner,
            Some(vault_authority),
        )?;

        token::transfer(
            ctx.accounts.into_transfer_to_pda_context(),
            ctx.accounts.escrow_state.initializer_amount[0]
                + ctx.accounts.escrow_state.initializer_amount[1]
                + ctx.accounts.escrow_state.initializer_amount[2]
                + ctx.accounts.escrow_state.initializer_amount[3]
                + ctx.accounts.escrow_state.initializer_amount[4],
        )?;

        Ok(())
    }

    pub fn withdraw_for_resolve(ctx: Context<WithdrawForResolve>) -> Result<()> {
        let (_vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED], ctx.program_id);
        let authority_seeds = &[&AUTHORITY_SEED[..], &[vault_authority_bump]];

        token::transfer(
            ctx.accounts
                .into_transfer_to_resolver_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_state.initializer_amount[0]
                + ctx.accounts.escrow_state.initializer_amount[1]
                + ctx.accounts.escrow_state.initializer_amount[2]
                + ctx.accounts.escrow_state.initializer_amount[3]
                + ctx.accounts.escrow_state.initializer_amount[4],
        )?;

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
        )?;

        Ok(())
    }

    pub fn approve(ctx: Context<Approve>, milestone_idx: u64) -> Result<()> {
        let (_vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED], ctx.program_id);
        let authority_seeds = &[&AUTHORITY_SEED[..], &[vault_authority_bump]];

        token::transfer(
            ctx.accounts
                .into_transfer_to_taker_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_state.initializer_amount[milestone_idx as usize]
                * (100 - ctx.accounts.admin_state.admin_fee)
                / 100,
        )?;

        token::transfer(
            ctx.accounts
                .into_transfer_to_admin1_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_state.initializer_amount[milestone_idx as usize]
                * ctx.accounts.admin_state.admin_fee
                * 15
                / 10000,
        )?;

        token::transfer(
            ctx.accounts
                .into_transfer_to_admin2_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_state.initializer_amount[milestone_idx as usize]
                * ctx.accounts.admin_state.admin_fee
                * 85
                / 10000,
        )?;

        ctx.accounts.escrow_state.initializer_amount[milestone_idx as usize] = 0;

        Ok(())
    }

    pub fn resolve(ctx: Context<Resolve>, milestone_idx: u64) -> Result<()> {
        let (_vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED], ctx.program_id);
        let authority_seeds = &[&AUTHORITY_SEED[..], &[vault_authority_bump]];

        token::transfer(
            ctx.accounts
                .into_transfer_to_taker_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_state.initializer_amount[milestone_idx as usize]
                * (100
                    - ctx.accounts.admin_state.admin_fee
                    - ctx.accounts.admin_state.resolver_fee)
                / 100,
        )?;

        token::transfer(
            ctx.accounts
                .into_transfer_to_admin1_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_state.initializer_amount[milestone_idx as usize]
                * ctx.accounts.admin_state.admin_fee
                * 15
                / 10000,
        )?;

        token::transfer(
            ctx.accounts
                .into_transfer_to_admin2_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_state.initializer_amount[milestone_idx as usize]
                * ctx.accounts.admin_state.admin_fee
                * 85
                / 10000,
        )?;

        token::transfer(
            ctx.accounts
                .into_transfer_to_resolver_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_state.initializer_amount[milestone_idx as usize]
                * ctx.accounts.admin_state.resolver_fee
                / 100,
        )?;

        ctx.accounts.escrow_state.initializer_amount[milestone_idx as usize] = 0;

        Ok(())
    }

    pub fn init_admin(ctx: Context<InitAdmin>) -> Result<()> {
        ctx.accounts.admin_state.admin1 = *ctx.accounts.admin1.key;
        ctx.accounts.admin_state.admin2 = *ctx.accounts.admin2.key;
        ctx.accounts.admin_state.resolver = *ctx.accounts.resolver.key;

        Ok(())
    }

    pub fn change_admin(ctx: Context<ChangeAdmin>) -> Result<()> {
        ctx.accounts.admin_state.admin1 = *ctx.accounts.new_admin1.key;
        ctx.accounts.admin_state.admin2 = *ctx.accounts.new_admin2.key;
        ctx.accounts.admin_state.resolver = *ctx.accounts.new_resolver.key;

        Ok(())
    }

    pub fn set_fee(ctx: Context<SetFee>, admin_fee: u64, resolver_fee: u64) -> Result<()> {
        ctx.accounts.admin_state.admin_fee = admin_fee;
        ctx.accounts.admin_state.resolver_fee = resolver_fee;

        Ok(())
    }

    pub fn dispute(ctx: Context<Dispute>, _escrow_seed: u64) -> Result<()> {
        ctx.accounts.escrow_state.dispute_status = true;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitAdmin<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub admin1: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub admin2: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub resolver: AccountInfo<'info>,
    #[account(
         init,
         seeds = [b"state".as_ref(), b"admin".as_ref()],
         bump,
         payer = admin1,
         space = AdminState::space()
     )]
    pub admin_state: Box<Account<'info, AdminState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ChangeAdmin<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub admin1: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub new_admin1: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub new_admin2: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub new_resolver: AccountInfo<'info>,
    #[account(
        mut,
        constraint = admin_state.admin1 == *admin1.key
    )]
    pub admin_state: Box<Account<'info, AdminState>>,
}

#[derive(Accounts)]
#[instruction(admin_fee: u64, resolver_fee: u64)]
pub struct SetFee<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub admin1: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = admin_state.admin1 == *admin1.key
    )]
    pub admin_state: Box<Account<'info, AdminState>>,
}

#[derive(Accounts)]
#[instruction(escrow_seed: u64, initializer_amount: [u64;5])]
pub struct Initialize<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub initializer: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub taker: AccountInfo<'info>,
    #[account(mut)]
    pub admin_state: Box<Account<'info, AdminState>>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        seeds = [b"vault".as_ref(), &escrow_seed.to_le_bytes()],
        bump,
        payer = initializer,
        token::mint = mint,
        token::authority = initializer,
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = initializer_deposit_token_account.mint == *mint.to_account_info().key,
        constraint = initializer_deposit_token_account.amount >=(initializer_amount[0]+initializer_amount[1]+initializer_amount[2]+initializer_amount[3]+initializer_amount[4])
    )]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        seeds = [b"state".as_ref(), &escrow_seed.to_le_bytes()],
        bump,
        payer = initializer,
        space = EscrowState::space()
    )]
    pub escrow_state: Box<Account<'info, EscrowState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(escrow_seed: u64)]
pub struct Dispute<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub disputor: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = escrow_state.random_seed == escrow_seed,
        constraint = escrow_state.initializer_key == *disputor.to_account_info().key || escrow_state.taker == *disputor.to_account_info().key,
    )]
    pub escrow_state: Box<Account<'info, EscrowState>>,
}

// used for resolver to withdraw money in the vault
#[derive(Accounts)]
pub struct WithdrawForResolve<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub resolver: Signer<'info>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    #[account(mut, constraint = resolver_token_account.owner == *resolver.key)]
    pub resolver_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = escrow_state.admin1 == resolver_token_account.owner,
        close = resolver
    )]
    pub escrow_state: Box<Account<'info, EscrowState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(milestone_idx:u64)]
pub struct Approve<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub initializer: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub taker: AccountInfo<'info>,
    #[account(
        mut,
        constraint = taker_token_account.owner == *taker.key
    )]
    pub taker_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub admin1_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub admin2_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = escrow_state.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        constraint = escrow_state.taker == *taker.key,
        constraint = escrow_state.initializer_key == *initializer.key,
        constraint = escrow_state.initializer_amount[milestone_idx as usize] > 0,
        constraint = escrow_state.dispute_status == false,
    )]
    pub escrow_state: Box<Account<'info, EscrowState>>,
    #[account(
        mut,
        constraint = admin_state.admin1 == admin1_token_account.owner,
        constraint = admin_state.admin2 == admin2_token_account.owner,
    )]
    pub admin_state: Box<Account<'info, AdminState>>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(milestone_idx:u64)]
pub struct Resolve<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub resolver: Signer<'info>,
    #[account(mut)]
    pub taker_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub admin1_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub admin2_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, 
        constraint = resolver_token_account.owner == *resolver.key
    )]
    pub resolver_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = escrow_state.taker == taker_token_account.owner || escrow_state.initializer_key == taker_token_account.owner,
        constraint = escrow_state.resolver == *resolver.key,
        constraint = escrow_state.initializer_amount[milestone_idx as usize] > 0,
        constraint = escrow_state.dispute_status == true,
    )]
    pub escrow_state: Box<Account<'info, EscrowState>>,
    #[account(
        mut,
        constraint = admin_state.admin1 == admin1_token_account.owner,
        constraint = admin_state.admin2 == admin2_token_account.owner,
    )]
    pub admin_state: Box<Account<'info, AdminState>>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct AdminState {
    admin_fee: u64,
    resolver_fee: u64,
    pub admin1: Pubkey,
    pub admin2: Pubkey,
    pub resolver: Pubkey,
}

impl AdminState {
    pub fn space() -> usize {
        8 + 112
    }
}

#[account]
pub struct EscrowState {
    pub random_seed: u64,
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub taker: Pubkey,
    pub initializer_amount: [u64; 5],
    pub admin1: Pubkey,
    pub admin2: Pubkey,
    pub resolver: Pubkey,
    pub dispute_status: bool,
}

impl EscrowState {
    pub fn space() -> usize {
        8 + 241
    }
}

impl<'info> Initialize<'info> {
    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.initializer_deposit_token_account.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.initializer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.vault.to_account_info(),
            current_authority: self.initializer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

impl<'info> WithdrawForResolve<'info> {
    fn into_transfer_to_resolver_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.resolver_token_account.to_account_info(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.resolver.to_account_info(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

impl<'info> Approve<'info> {
    fn into_transfer_to_taker_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.taker_token_account.to_account_info(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_admin1_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.admin1_token_account.to_account_info(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_admin2_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.admin2_token_account.to_account_info(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

impl<'info> Resolve<'info> {
    fn into_transfer_to_taker_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.taker_token_account.to_account_info(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_admin1_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.admin1_token_account.to_account_info(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_admin2_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.admin2_token_account.to_account_info(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_resolver_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.resolver_token_account.to_account_info(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
