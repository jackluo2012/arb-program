use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, instruction::AccountMeta,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::error::ArbitrageProgramError;

/// 检查流动性池地址是否有效
///
/// 该函数通过程序ID和预设的种子生成预期的流动性池地址，
/// 并与传入的池地址进行比较，验证其有效性。
///
/// # 参数
/// * `program_id` - 程序的公钥标识
/// * `pool` - 待验证的流动性池地址
///
/// # 返回值
/// * `ProgramResult` - 验证结果，成功返回Ok(()), 失败返回相应的错误码
///
/// # 错误
/// * `ArbitrageProgramError::InvalidSwapNotEnoughLiquidity` - 当池地址无效时返回
pub fn check_pool_address(program_id: &Pubkey, pool: &Pubkey) -> ProgramResult {
    // 验证传入的池地址是否与根据程序ID生成的预期地址匹配
    if !Pubkey::find_program_address(&[b"liquidity_pool"], program_id)
        .0
        .eq(pool)
    {
        return Err(ArbitrageProgramError::InvalidSwapNotEnoughLiquidity.into());
    }
    Ok(())
}

pub trait ArbtrageEvaluateOption<T> {
    fn ok_or_arb_err(self) -> Result<T, ProgramError>;
}
impl<T> ArbtrageEvaluateOption<T> for Option<T> {
    /// 将Option类型转换为Result类型，如果为None则返回特定的程序错误
    ///
    /// # 参数
    /// * `self` - Option类型的值
    ///
    /// # 返回值
    /// * `Ok(T)` - 当原值为Some时，返回包含值的Ok结果
    /// * `Err(ProgramError)` - 当原值为None时，返回ArbitrageProgramError::InvalidAccountsList错误
    fn ok_or_arb_err(self) -> Result<T, ProgramError> {
        // 匹配Option值，Some则返回Ok，None则返回特定错误
        match self {
            Some(value) => Ok(value),
            None => Err(ArbitrageProgramError::InvalidAccountsList.into()),
        }
    }
}

pub trait ToAccountMeta {
    fn to_account_meta(&self) -> AccountMeta;
}

impl ToAccountMeta for AccountInfo<'_> {
    /// 将当前对象转换为AccountMeta类型
    ///
    /// # Returns
    /// 返回一个新的AccountMeta实例，包含以下字段：
    /// - pubkey: 账户的公钥，从self.key解引用获得
    /// - is_signer: 标识该账户是否为签名者，直接复制self.is_signer的值
    /// - is_writable: 标识该账户是否可写，直接复制self.is_writable的值
    fn to_account_meta(&self) -> AccountMeta {
        AccountMeta {
            pubkey: *self.key,
            is_signer: self.is_signer,
            is_writable: self.is_writable,
        }
    }
}
