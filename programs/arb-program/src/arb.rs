use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, instruction::Instruction, msg,
    program::invoke, pubkey::Pubkey,
};

use crate::{
    error::ArbitrageProgramError,
    partial_state::{ArbitrageMintInfo, ArbitrageTokenAccountInfo},
    swap::determine_swap_receive,
    util::{ArbtrageEvaluateOption, ToAccountMeta},
};

/// 尝试执行套利交易的参数结构体
///
/// 该结构体包含了执行套利交易所需的所有账户信息和程序引用。
/// 主要用于在两个不同的交易池之间进行价格套利操作。
pub struct TryArbitrageArgs<'a, 'b> {
    /// 用户的代币账户信息列表，用于存储用户持有的各种代币余额
    pub token_accounts_user: Vec<ArbitrageTokenAccountInfo<'a, 'b>>,
    /// 第一个交易池的代币账户信息列表，包含该交易池中的代币储备信息
    pub token_accounts_swap_1: Vec<ArbitrageTokenAccountInfo<'a, 'b>>,
    /// 第二个交易池的代币账户信息列表，包含该交易池中的代币储备信息
    pub token_accounts_swap_2: Vec<ArbitrageTokenAccountInfo<'a, 'b>>,
    /// 铸币信息列表，包含套利涉及的所有代币的铸币账户信息
    pub mints: Vec<ArbitrageMintInfo<'a, 'b>>,
    /// 交易支付方账户，用于支付交易费用和作为交易签名者
    pub payer: &'a AccountInfo<'b>,
    /// SPL代币程序账户，用于执行代币相关的操作
    pub token_program: &'a AccountInfo<'b>,
    /// 系统程序账户，用于创建新账户等系统级操作
    pub system_program: &'a AccountInfo<'b>,
    /// 关联代币程序账户，用于创建和管理关联代币账户
    pub associated_token_program: &'a AccountInfo<'b>,
    /// 第一个交易池的程序账户，用于调用第一个交易池的交换逻辑
    pub swap_1_program: &'a AccountInfo<'b>,
    /// 第二个交易池的程序账户，用于调用第二个交易池的交换逻辑
    pub swap_2_program: &'a AccountInfo<'b>,
    /// 第一个交易池账户，包含第一个交易池的状态和配置信息
    pub swap_1_pool: &'a AccountInfo<'b>,
    /// 第二个交易池账户，包含第二个交易池的状态和配置信息
    pub swap_2_pool: &'a AccountInfo<'b>,
    /// 温度参数，用于控制套利交易的敏感度或风险级别
    pub temperature: u8,
}

/// 尝试在两个去中心化交易所池之间执行套利交易。
///
/// 该函数会遍历所有资产对，计算在两个交易池之间的潜在套利机会，并在发现有利可图的交易时执行。
///
/// # 参数
///
/// * [args](file://d:\works\learn\rust\solana\arb-program\node_modules\yargs\yargs) - 套利参数结构体，包含以下字段：
///   - `swap_1_pool`: 第一个交易池账户信息
///   - `swap_2_pool`: 第二个交易池账户信息
///   - `mints`: 所有相关代币的 Mint 账户列表
///   - `token_accounts_user`: 用户拥有的代币账户列表
///   - `token_accounts_swap_1`: 第一个交易池中的代币账户列表
///   - `token_accounts_swap_2`: 第二个交易池中的代币账户列表
///   - `swap_1_program`: 第一个交易程序的账户信息
///   - `swap_2_program`: 第二个交易程序的账户信息
///   - `payer`: 交易支付者账户
///   - `token_program`: SPL Token 程序账户
///   - `system_program`: 系统程序账户
///   - `associated_token_program`: 关联代币程序账户
///   - `temperature`: 套利温度阈值，用于判断是否执行交易
///
/// # 返回值
///
/// * `ProgramResult` - 如果成功执行套利则返回 Ok，否则返回错误码。
///   - 成功执行后将返回相应的交易调用结果
///   - 如果没有找到套利机会，则返回 `ArbitrageProgramError::NoArbitrage`
pub fn try_arbitrage(args: TryArbitrageArgs<'_, '_>) -> ProgramResult {
    msg!("Swap #1 Pool: {}", args.swap_1_pool.key);
    msg!("Swap #2 Pool: {}", args.swap_2_pool.key);

    let mints_len = args.mints.len();

    // 遍历每一对不同的资产（i 和 j），尝试寻找套利路径
    for i in 0..mints_len {
        // 加载当前资产相关的用户账户和两个交易池中的账户及 Mint 信息
        let user_i = args.token_accounts_user.get(i).ok_or_arb_err()?;
        let swap_1_i = args.token_accounts_swap_1.get(i).ok_or_arb_err()?;
        let swap_2_i = args.token_accounts_swap_2.get(i).ok_or_arb_err()?;
        let mint_i = args.mints.get(i).ok_or_arb_err()?;

        for j in (i + 1)..mints_len {
            // 加载目标资产相关的用户账户和两个交易池中的账户及 Mint 信息
            let user_j = args.token_accounts_user.get(j).ok_or_arb_err()?;
            let swap_1_j = args.token_accounts_swap_1.get(j).ok_or_arb_err()?;
            let swap_2_j = args.token_accounts_swap_2.get(j).ok_or_arb_err()?;
            let mint_j = args.mints.get(j).ok_or_arb_err()?;

            // 计算在两个交易池中进行兑换时预期能获得的目标资产数量
            let r_swap_1 =
                determine_swap_receive(swap_1_j.3, mint_j.1, swap_1_i.3, mint_i.1, user_i.3)?;
            let r_swap_2 =
                determine_swap_receive(swap_2_j.3, mint_j.1, swap_2_i.3, mint_i.1, user_i.3)?;

            // 如果兑换金额为零或超过池子余额，则跳过此对资产
            if r_swap_1 == 0 || r_swap_1 > swap_1_j.3 || r_swap_2 == 0 || r_swap_2 > swap_2_j.3 {
                continue;
            }

            // 检查是否存在套利机会
            if let Some(trade) = check_for_arbitrage(r_swap_1, r_swap_2, args.temperature) {
                // 若存在套利机会，则执行交易
                msg!("PLACING TRADE!");
                return match trade {
                    // 在 Swap #1 上买入并在 Swap #2 上卖出
                    Buy::Swap1 => {
                        msg!("Buy on Swap #1 and sell on Swap #2");
                        invoke_arbitrage(
                            (
                                *args.swap_1_program.key,
                                &[
                                    args.swap_1_pool.to_owned(),
                                    mint_j.0.to_owned(),
                                    swap_1_j.0.to_owned(),
                                    user_j.0.to_owned(),
                                    mint_i.0.to_owned(),
                                    swap_1_i.0.to_owned(),
                                    user_i.0.to_owned(),
                                    args.payer.to_owned(),
                                    args.token_program.to_owned(),
                                    args.system_program.to_owned(),
                                    args.associated_token_program.to_owned(),
                                ],
                                user_i.3,
                            ),
                            (
                                *args.swap_2_program.key,
                                &[
                                    args.swap_2_pool.to_owned(),
                                    mint_i.0.to_owned(),
                                    swap_2_i.0.to_owned(),
                                    user_i.0.to_owned(),
                                    mint_j.0.to_owned(),
                                    swap_2_j.0.to_owned(),
                                    user_j.0.to_owned(),
                                    args.payer.to_owned(),
                                    args.token_program.to_owned(),
                                    args.system_program.to_owned(),
                                    args.associated_token_program.to_owned(),
                                ],
                                r_swap_1,
                            ),
                        )
                    }
                    // 在 Swap #2 上买入并在 Swap #1 上卖出
                    Buy::Swap2 => {
                        msg!("Buy on Swap #2 and sell on Swap #1");
                        invoke_arbitrage(
                            (
                                *args.swap_2_program.key,
                                &[
                                    args.swap_2_pool.to_owned(),
                                    mint_j.0.to_owned(),
                                    swap_2_j.0.to_owned(),
                                    user_j.0.to_owned(),
                                    mint_i.0.to_owned(),
                                    swap_2_i.0.to_owned(),
                                    user_i.0.to_owned(),
                                    args.payer.to_owned(),
                                    args.token_program.to_owned(),
                                    args.system_program.to_owned(),
                                    args.associated_token_program.to_owned(),
                                ],
                                r_swap_2,
                            ),
                            (
                                *args.swap_1_program.key,
                                &[
                                    args.swap_1_pool.to_owned(),
                                    mint_i.0.to_owned(),
                                    swap_1_i.0.to_owned(),
                                    user_i.0.to_owned(),
                                    mint_j.0.to_owned(),
                                    swap_1_j.0.to_owned(),
                                    user_j.0.to_owned(),
                                    args.payer.to_owned(),
                                    args.token_program.to_owned(),
                                    args.system_program.to_owned(),
                                    args.associated_token_program.to_owned(),
                                ],
                                user_j.3,
                            ),
                        )
                    }
                };
            }
        }
    }

    // 如果遍历完所有资产对仍未发现套利机会，则返回无套利错误
    Err(ArbitrageProgramError::NoArbitrage.into())
}

/// 买入操作枚举类型
///
/// 该枚举定义了两种不同的买入策略或方式，用于区分不同的交易路径或机制。
///
/// 变体说明：
/// - Swap1: 第一种买入策略
/// - Swap2: 第二种买入策略
enum Buy {
    Swap1,
    Swap2,
}
/// 检查是否存在套利机会
///
/// 该函数通过比较两个交换池的价格差异来判断是否存在套利机会。
/// 当价格差异超过由温度参数决定的阈值时，返回相应的购买建议。
///
/// # 参数
/// * `r_swap_1` - 第一个交换池的汇率值
/// * `r_swap_2` - 第二个交换池的汇率值
/// * `temperature` - 温度参数，用于调整套利检测的敏感度，值越小越敏感
///
/// # 返回值
/// * `Some(Buy::Swap1)` - 当第一个交换池存在套利机会时返回
/// * `Some(Buy::Swap2)` - 当第二个交换池存在套利机会时返回
/// * `None` - 当不存在套利机会时返回
fn check_for_arbitrage(r_swap_1: u64, r_swap_2: u64, temperature: u8) -> Option<Buy> {
    // 计算套利检测阈值，温度越低阈值越高
    let threshold = 100.0 - temperature as f64;
    // 计算两个交换池之间的价格差异百分比
    let percent_diff = (r_swap_1 as f64 / r_swap_2 as f64 - 1.0).abs() * 100.0;
    // 判断价格差异是否超过阈值
    if percent_diff > threshold {
        // 根据价格差异的方向决定购买哪个交换池
        if percent_diff > 0.0 {
            return Some(Buy::Swap1);
        } else {
            return Some(Buy::Swap2);
        }
    }
    None
}

/// 执行套利交易函数，先后执行买入和卖出两个交易指令
///
/// # 参数
/// * `buy` - 买入交易信息元组，包含程序ID、账户信息切片和买入金额
/// * `sell` - 卖出交易信息元组，包含程序ID、账户信息切片和卖出金额
///
/// # 返回值
/// * `ProgramResult` - 程序执行结果，成功返回Ok(())，失败返回相应错误
fn invoke_arbitrage(
    buy: (Pubkey, &[AccountInfo], u64),
    sell: (Pubkey, &[AccountInfo], u64),
) -> ProgramResult {
    // 构建买入和卖出指令的数据
    let (buy_swap_ix_data, sell_swap_ix_data) = build_ix_datas(buy.2, sell.2);

    // 创建买入交易指令
    let ix_buy = Instruction::new_with_borsh(
        buy.0,
        &buy_swap_ix_data,
        buy.1.iter().map(ToAccountMeta::to_account_meta).collect(),
    );

    // 创建卖出交易指令
    let ix_sell = Instruction::new_with_borsh(
        sell.0,
        &sell_swap_ix_data,
        sell.1.iter().map(ToAccountMeta::to_account_meta).collect(),
    );

    // 执行买入交易
    msg!("Executing buy ...");
    invoke(&ix_buy, buy.1)?;

    // 执行卖出交易
    msg!("Executing sell ...");
    invoke(&ix_sell, sell.1)?;

    Ok(())
}

/// 构建交易指令数据
///
/// 该函数用于生成买入和卖出交易的指令数据，主要用于Solana程序交易
///
/// # 参数
/// * `buy_amount` - 买入数量，类型为u64
/// * `sell_amount` - 卖出数量，类型为u64
///
/// # 返回值
/// 返回一个元组，包含两个16字节的数组：
/// * 第一个元素是买入交易指令数据
/// * 第二个元素是卖出交易指令数据
fn build_ix_datas(buy_amount: u64, sell_amount: u64) -> ([u8; 16], [u8; 16]) {
    let mut buy_swap_ix_data = [0u8; 16];
    let mut sell_swap_ix_data = [0u8; 16];

    // 生成swap指令的哈希值，用于标识交易类型
    let swap_ix_hash = solana_program::hash::hash(b"global:swap");

    // 将交易金额转换为大端字节序
    let buy_amount_as_bytes: [u8; 8] = buy_amount.to_be_bytes();
    let sell_amount_as_bytes: [u8; 8] = sell_amount.to_be_bytes();

    // 构造买入交易指令数据：前8字节为指令哈希，后8字节为交易金额
    buy_swap_ix_data[..8].copy_from_slice(&swap_ix_hash.to_bytes()[..8]);
    buy_swap_ix_data[8..].copy_from_slice(&buy_amount_as_bytes);

    // 构造卖出交易指令数据：前8字节为指令哈希，后8字节为交易金额
    sell_swap_ix_data[..8].copy_from_slice(&swap_ix_hash.to_bytes()[..8]);
    sell_swap_ix_data[8..].copy_from_slice(&sell_amount_as_bytes);

    (buy_swap_ix_data, sell_swap_ix_data)
}
