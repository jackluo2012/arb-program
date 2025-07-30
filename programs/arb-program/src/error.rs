use solana_program::msg;
/// 定义套利程序中可能发生的错误类型枚举
///
/// 该枚举包含了在执行套利交易过程中可能遇到的各种错误情况，
/// 用于错误处理和程序流程控制。每个错误变体都包含了详细的错误
/// 描述信息，便于调试和问题定位。
#[derive(Clone, Debug, Eq, thiserror::Error, num_derive::FromPrimitive, PartialEq)]
pub enum ArbitrageProgramError {
    /// 无效的账户列表：每个账户列表的长度应相同，
    /// 并且应按以下顺序传入：
    /// 用户代币账户、掉期 1 代币账户、掉期 2 代币账户、铸币账户
    #[error("Invalid list of accounts: Each list of accounts should be the same length and passed in the following order: user token accounts, swap 1 token accounts, swap 2 token accounts, mints")]
    InvalidAccountsList,
    /// A token account not belonging to the user, swap #1's Liquidity Pool, or
    /// swap #2's Liquidity Pool was passed into the program
    #[error("A token account not belonging to the user, swap #1's Liquidity Pool, or swap #2's Liquidity Pool was passed into the program")]
    TokenAccountOwnerNotFound,
    /// The user's proposed pay amount resolves to a value for [r](file://d:\works\learn\rust\solana\arb-program\node_modules\typescript\bin\tsserver) that exceeds
    /// the balance of the pool's token account for the receive asset
    #[error("The amount proposed to pay resolves to a receive amount that is greater than the current liquidity")]
    InvalidSwapNotEnoughLiquidity,
    /// No arbitrage opportunity was detected, so the program will return an
    /// error so that preflight fails
    #[error("No arbitrage opportunity detected")]
    NoArbitrage,
}
impl From<ArbitrageProgramError> for solana_program::program_error::ProgramError {
    /// 将ArbitrageProgramError转换为Solana程序错误
    ///
    /// # 参数
    /// * [e](file://d:\works\learn\rust\solana\arb-program\node_modules\arrify\license) - ArbitrageProgramError枚举值，需要被转换的自定义错误类型
    ///
    /// # 返回值
    /// 返回对应的Solana程序错误，将自定义错误编码为u32格式的Custom错误
    fn from(e: ArbitrageProgramError) -> Self {
        // 将自定义错误类型转换为u32编码，包装为Solana的Custom程序错误
        solana_program::program_error::ProgramError::Custom(e as u32)
    }
}

impl ArbitrageProgramError {
    pub fn log(&self) {
        msg!("{}", &self.to_string());
    }
}
