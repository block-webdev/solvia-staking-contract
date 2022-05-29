use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::*;

#[account]
#[derive(Default)]
pub struct GlobalPool {
    pub total_stake_amount: u64,   // 8
    pub total_stake_users: u32   // 4
}

#[account]
#[derive(Default)]
pub struct UserPool {
    // 65
    pub user: Pubkey,            // 32
    pub stake_amount: u64,       // 8
    pub stake_mode: u8,          // 1
    pub stake_time: i64,         // 8
    pub reward_time: i64,        // 8
    pub reward_amount: u64,      // 8
}

