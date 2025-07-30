use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};
use spl_associated_token_account::tools::account;

use crate::arb::{try_arbitrage, TryArbitrageArgs};
use crate::partial_state::{PartialMintState, PartialTokenAccountState};
use crate::util::check_pool_address;

/// 处理套利交易逻辑的主函数。
///
/// 该函数解析传入的账户信息，验证交易池地址，并为用户、两个交易池准备代币账户和铸币信息，
/// 最终调用 `try_arbitrage` 执行实际的套利操作。
///
/// # 参数说明
/// - `accounts`: 包含所有相关账户信息的切片，用于交易和状态读取。
/// - `swap_1_program_id`: 第一个去中心化交易所（DEX）的程序 ID。
/// - `swap_2_program_id`: 第二个去中心化交易所（DEX）的程序 ID。
/// - `concurrency`: 并行处理的代币对数量。
/// - `temperature`: 控制套利行为的参数（具体含义由业务逻辑定义）。
///
/// # 返回值
/// 返回 `ProgramResult`，表示操作是否成功执行。
pub fn process_arbitrage(
    accounts: &[AccountInfo],
    swap_1_program_id: &Pubkey,
    swap_2_program_id: &Pubkey,
    concurrency: u8,
    temperature: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let associated_token_program = next_account_info(accounts_iter)?;
    let swap_1_program = next_account_info(accounts_iter)?;
    let swap_2_program = next_account_info(accounts_iter)?;
    let swap_1_pool = next_account_info(accounts_iter)?;
    let swap_2_pool = next_account_info(accounts_iter)?;

    // 验证两个交易池的地址是否与指定的程序 ID 匹配
    check_pool_address(swap_1_program_id, swap_1_pool.key)?;
    check_pool_address(swap_2_program_id, swap_2_pool.key)?;

    // 解析用户相关的代币账户状态
    let token_accounts_user = {
        let mut accts = vec![];
        for _x in 0..concurrency {
            accts.push(PartialTokenAccountState::try_deserialize(
                next_account_info(accounts_iter)?,
                payer.key,
            )?);
        }
        accts
    };

    // 解析第一个交易池相关的代币账户状态
    let token_accounts_swap_1 = {
        let mut accts = vec![];
        for _x in 0..concurrency {
            accts.push(PartialTokenAccountState::try_deserialize(
                next_account_info(accounts_iter)?,
                swap_1_pool.key,
            )?);
        }
        accts
    };

    // 解析第二个交易池相关的代币账户状态
    let token_accounts_swap_2 = {
        let mut accts = vec![];
        for _x in 0..concurrency {
            accts.push(PartialTokenAccountState::try_deserialize(
                next_account_info(accounts_iter)?,
                swap_2_pool.key,
            )?);
        }
        accts
    };

    // 解析所有涉及的铸币信息
    let mints = {
        let mut accts = vec![];
        for _x in 0..concurrency {
            accts.push(PartialMintState::try_deserialize(next_account_info(
                accounts_iter,
            )?)?);
        }
        accts
    };

    // 调用核心套利逻辑函数
    try_arbitrage(TryArbitrageArgs {
        token_accounts_user,
        token_accounts_swap_1,
        token_accounts_swap_2,
        mints,
        payer,
        token_program,
        system_program,
        associated_token_program,
        swap_1_program,
        swap_2_program,
        swap_1_pool,
        swap_2_pool,
        temperature,
    })
}
