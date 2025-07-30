use crate::error::ArbitrageProgramError;
use bytemuck::{Pod, Zeroable};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};
use spl_pod::optional_keys::OptionalNonZeroPubkey;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
/// 表示部分代币账户状态的结构体
///
/// 该结构体用于存储代币账户的核心信息，包括铸币地址、所有者地址和代币数量
pub struct PartialTokenAccountState {
    pub mint: Pubkey,  //占用32字节
    pub owner: Pubkey, //占用32字节
    pub amount: u64,   //占用8字节
}

/// 我们的套利算法的自定义返回类型：
/// (acount,mint,owner,amount)
/// 定义一个用于套利交易的代币账户信息类型别名
///
/// 该类型别名封装了套利交易中需要的代币账户相关信息，包括账户信息、公钥、
/// 代币程序ID和账户余额等核心数据。
///
/// # 类型参数
/// * `'a` - 账户信息的生命周期参数
/// * `'b` - 账户信息内部引用的生命周期参数
///
/// # 包含的字段
/// * `&'a AccountInfo<'b>` - 账户信息引用，包含账户的完整信息
/// * `Pubkey` - 账户的公钥地址
/// * `Pubkey` - 代币程序的公钥ID
/// * `u64` - 账户当前的代币余额
pub type ArbitrageTokenAccountInfo<'a, 'b> = (&'a AccountInfo<'b>, Pubkey, Pubkey, u64);
impl PartialTokenAccountState {
    /// 尝试从账户信息中反序列化代币账户数据
    ///
    /// 该函数验证账户数据的长度和所有者，并尝试将其解析为代币账户信息。
    ///
    /// # 参数
    /// * `account_info` - 要反序列化的账户信息引用
    /// * `owner` - 预期的所有者公钥引用
    ///
    /// # 返回值
    /// 成功时返回包含账户信息、铸币地址、所有者和金额的元组，失败时返回程序错误
    pub fn try_deserialize<'a, 'b>(
        account_info: &'a AccountInfo<'b>,
        owner: &'a Pubkey,
    ) -> Result<ArbitrageTokenAccountInfo<'a, 'b>, ProgramError> {
        // 验证账户数据长度是否足够（至少72字节）
        if account_info.data_len() < 72 {
            msg!(
                "Data too small. Should be 72 bytes. Found len: {}",
                account_info.data_len()
            );

            return Err(ArbitrageProgramError::InvalidAccountsList.into());
        }

        // 尝试将账户数据的前72字节转换为代币账户结构
        match bytemuck::try_from_bytes::<Self>(&account_info.data.borrow()[..72]) {
            Ok(partial_token) => {
                // 验证账户所有者是否匹配
                if !partial_token.owner.eq(owner) {
                    msg!("Owner mismatch");
                    msg!("Expected: {}", owner);
                    msg!("Got:      {}", partial_token.owner);
                    msg!("Token Account: {}", account_info.key);
                    return Err(ArbitrageProgramError::InvalidAccountsList.into());
                }
                Ok((
                    account_info,
                    partial_token.mint,
                    partial_token.owner,
                    partial_token.amount,
                ))
            }
            Err(_) => Err(ArbitrageProgramError::InvalidAccountsList.into()),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
/// 部分铸币状态结构体，用于存储代币铸造的相关信息
///
/// 该结构体包含了铸造权限和供应量信息，通常用于代币合约的状态管理
pub struct PartialMintState {
    /// 铸造权限地址，使用OptionalNonZeroPubkey类型确保地址的有效性
    /// 该字段占用32字节的存储空间
    pub mint_authority: OptionalNonZeroPubkey, // 占用 32 字节
    /// 代币的总供应量
    /// 该字段占用8字节的存储空间
    pub supply: u64, // 占用 8 字节
}

/// 仲裁铸币信息类型别名
///

/// 定义一个用于套利挖矿信息的类型别名
///
/// 该类型别名用于表示套利挖矿相关的账户信息和权限级别
///
/// # 泛型参数
/// * `'a` - 账户信息引用的生命周期
/// * `'b` - 账户数据引用的生命周期
///
/// # 类型组成
/// * `&'a AccountInfo<'b>` - 指向账户信息的引用，包含账户的公钥、余额等数据
/// * `u8` - 权限级别或角色标识，用于区分不同的操作权限
pub type ArbitrageMintInfo<'a, 'b> = (&'a AccountInfo<'b>, u8);

impl PartialMintState {
    /// 尝试从账户信息中反序列化ArbitrageMintInfo数据
    ///
    /// 该函数检查账户数据是否包含足够的字节来构成有效的ArbitrageMintInfo，
    /// 并尝试进行反序列化操作。成功时返回包含账户信息和小数位数的元组。
    ///
    /// # 参数
    /// * `account_info` - 要反序列化的账户信息引用
    ///
    /// # 返回值
    /// * `Ok(ArbitrageMintInfo<'a, 'b>)` - 成功反序列化时返回包含账户信息和小数位数的元组
    /// * `Err(ProgramError)` - 当数据长度不足、反序列化失败或无法获取小数位数时返回错误
    pub fn try_deserialize<'a, 'b>(
        account_info: &'a AccountInfo<'b>,
    ) -> Result<ArbitrageMintInfo<'a, 'b>, ProgramError> {
        // 检查账户数据长度是否足够（至少41字节）
        if account_info.data_len() < 41 {
            msg!(
                "Data too small. Should be 41 bytes. Found len: {}",
                account_info.data_len()
            );

            return Err(ArbitrageProgramError::InvalidAccountsList.into());
        }

        // 获取前41字节的数据用于反序列化
        let data = &account_info.data.borrow()[..41];

        // 尝试反序列化前40字节为主要结构体数据
        match bytemuck::try_from_bytes::<Self>(&data[..40]) {
            Ok(_) => {
                // 从第41字节获取小数位数
                let decimals = match data.get(40) {
                    Some(d) => *d,
                    None => {
                        msg!("Could not get decimals");
                        msg!("Mint: {}", account_info.key);
                        return Err(ArbitrageProgramError::InvalidAccountsList.into());
                    }
                };
                Ok((account_info, decimals))
            }
            Err(_) => {
                msg!("Failed to deserialize mint account");
                msg!("Mint: {}", account_info.key);
                return Err(ArbitrageProgramError::InvalidAccountsList.into());
            }
        }
    }
}
