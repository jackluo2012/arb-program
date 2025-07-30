pub mod arb;
pub mod error;
pub mod partial_state;
pub mod processor;
pub mod swap;
pub mod util;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
/// ArbitrageProgramInstruction 枚举定义了套利程序的指令类型
///
/// 该枚举用于表示套利交易程序中的不同操作指令，目前包含一个TryArbitrage变体，
/// 用于尝试执行套利交易操作。
///
/// # 变体说明
///
/// ## TryArbitrage
/// 尝试执行套利交易的指令，包含执行套利所需的基本参数
///
/// ### 字段说明
/// * `swap_1_program_id` - 第一个swap程序的公钥标识，用于识别第一个交易对的swap程序
/// * `swap_2_program_id` - 第二个swap程序的公钥标识，用于识别第二个交易对的swap程序
/// * `concurrency` - 并发级别，控制同时执行的交易数量
/// * `temperature` - 温度参数，可能用于控制交易的激进程度或风险水平
pub enum ArbitrageProgramInstruction {
    TryArbitrage {
        swap_1_program_id: Pubkey,
        swap_2_program_id: Pubkey,
        concurrency: u8,
        temperature: u8,
    },
}

/**
 * 程序入口点宏调用
 *
 * 该宏将`process`函数注册为程序的入口点。当程序启动时，
 * 运行时环境会调用这个指定的函数作为程序执行的起点。
 *
 * 参数: 无显式参数，但宏内部会处理程序启动所需的标准参数
 * 返回值: 无直接返回值，但会启动程序的主执行流程
 */
entrypoint!(process);

/// 处理程序入口函数，解析并执行套利交易指令
///
/// # 参数
/// * `program_id` - 程序的公钥标识
/// * `accounts` - 包含所有相关账户信息的数组
/// * `data` - 指令数据字节流
///
/// # 返回值
/// * `ProgramResult` - 程序执行结果，成功或错误信息
fn process(_program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // 解析指令数据并根据指令类型执行相应处理
    match ArbitrageProgramInstruction::try_from_slice(data) {
        Ok(ix) => match ix {
            ArbitrageProgramInstruction::TryArbitrage {
                swap_1_program_id,
                swap_2_program_id,
                concurrency,
                temperature,
            } => processor::process_arbitrage(
                accounts,
                &swap_1_program_id,
                &swap_2_program_id,
                concurrency,
                temperature,
            ),
        },
        Err(_) => Err(ProgramError::InvalidInstructionData),
    }
}
