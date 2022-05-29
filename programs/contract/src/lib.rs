use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer, Token};
use spl_token::instruction::AuthorityType;
use std::mem::size_of;
use spl_token::state;

pub mod account;
pub mod error;
pub mod constants;

use account::*;
use constants::*;
use error::*;



declare_id!("5uK2fyF65vnaY7BdKYoANym6uAtAwJseEDa6BdHAWNSv");

#[program]
pub mod contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {

        let global_authority = &mut ctx.accounts.global_authority;

        Ok(())
    }

    pub fn init_user_pool(ctx: Context<InitUserPool>, stake_mode: u8) -> Result<()> {
        
        let user_pool = &mut ctx.accounts.user_pool;
        user_pool.user = ctx.accounts.user.key();
        user_pool.stake_mode = stake_mode;
        let timestamp = Clock::get()?.unix_timestamp;
        user_pool.stake_time = timestamp;
        user_pool.reward_time = timestamp;


        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn stake(
        ctx: Context<Stake>, 
        global_bump: u8,
        amount: u64,
    ) -> Result<()> {

        let user_pool = &mut ctx.accounts.user_pool;
        user_pool.stake_amount += amount;

        ctx.accounts.global_authority.total_stake_amount += amount;
        let timestamp = Clock::get()?.unix_timestamp;
        user_pool.stake_time = timestamp;

        let cpi_accounts = Transfer {
            from: ctx.accounts.source_account.clone(),
            to: ctx.accounts.dest_account.clone(),
            authority: ctx.accounts.owner.to_account_info()
        };
        let token_program = ctx.accounts.token_program.clone();
        let transfer_ctx = CpiContext::new(token_program, cpi_accounts);
        token::transfer(
            transfer_ctx,
            amount
        )?;
        Ok(())
    }

    
    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn unstake(
        ctx: Context<Unstake>, 
        global_bump: u8,
        amount: u64,
    ) -> Result<()> {

        let user_pool = &mut ctx.accounts.user_pool;
        ctx.accounts.global_authority.total_stake_amount -= amount;

        let seeds = &[GLOBAL_AUTHORITY_SEED, &[global_bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.source_account.clone(),
            to: ctx.accounts.dest_account.clone(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        let token_program = ctx.accounts.token_program.clone();
        let transfer_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer);
        token::transfer(
            transfer_ctx,
            amount
        )?;

        Ok(())
    }

    pub fn claim_reward(
        ctx: Context<ClaimReward>,
        global_bump: u8,
        reward_amount: u64,
    ) -> Result<()> {
        let seeds = &[GLOBAL_AUTHORITY_SEED, &[global_bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.source_account.to_account_info(),
            to: ctx.accounts.dest_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        let token_program = ctx.accounts.token_program.clone();
        let transfer_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer);
        token::transfer(
            transfer_ctx,
            reward_amount
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {

    #[account(
        init_if_needed,
        seeds = [GLOBAL_AUTHORITY_SEED],
        bump,
        payer = owner,
        space=size_of::<GlobalPool>() + 8,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitUserPool<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump,
        payer = user,
        space=size_of::<UserPool>() + 8,
    )]
    pub user_pool: Account<'info, UserPool>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct Stake<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: Account<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    mint : Account<'info, Mint>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    source_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    dest_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    // pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, constraint = owner.key() == user_pool.user)]
    pub user_pool: Account<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    mint : Account<'info, Mint>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    source_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    dest_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    source_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,owner=spl_token::id())]
    dest_account : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

// Access control modifiers
fn user(pool_loader: &Account<UserPool>, user: &AccountInfo) -> Result<()> {
    require!(pool_loader.user == *user.key, StakingError::InvalidUserPool);
    Ok(())
}