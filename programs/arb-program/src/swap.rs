use crate::error::ArbitrageProgramError;
use solana_program::program_error::ProgramError;
use std::ops::{Add, Div, Mul};

/// 计算交换操作中接收方应获得的代币数量
///
/// 该函数基于恒定乘积公式计算在给定支付金额的情况下，接收方应获得的代币数量。
/// 使用公式: r = (R * p) / (P + p)
/// 其中 R 是接收代币池余额，P 是支付代币池余额，p 是支付金额，r 是接收金额
/// R = pool_receive_balance（接收代币池中的可用代币数量）
// P = pool_pay_balance（支付代币池中的可用代币数量）
// p = pay_amount（用户想要支付的代币数量）
// r = 结果（用户将要收到的代币数量）
/// # 参数
/// * `pool_receive_balance` - 接收代币的池子余额
/// * `receive_decimals` - 接收代币的小数位数
/// * `pool_pay_balance` - 支付代币的池子余额
/// * `pay_decimals` - 支付代币的小数位数
/// * `pay_amount` - 支付的代币数量
///
/// # 返回值
/// * `Ok(u64)` - 计算出的应接收代币数量

// 处理流程
// 小数位标准化：使用 convert_to_float 函数将整数形式的代币数量转换为浮点数，以正确处理小数精度
// 恒定乘积计算：应用公式计算用户应该收到的代币数量
// 流动性验证：确保计算结果不超过池子中的可用储备
// 结果转换：将浮点数结果转换回整数形式的代币数量
// 示例
// 假设我们有一个交换池，包含以下参数：

// 1000 USDC (pool_receive_balance)，有6位小数 (receive_decimals)
// 5000 DAI (pool_pay_balance)，有18位小数 (pay_decimals)
// 用户想要支付100 DAI (pay_amount)

// 计算过程：
// R = 1000 (标准化后)
// P = 5000 (标准化后)
// p = 100 (标准化后)
// r = (1000 * 100) / (5000 + 100) = 100000 / 5100 ≈ 19.61 USDC
// 假设初始池子状态：
// USDC池：1000个 (pool_receive_balance = 1000_000000，6位小数)
// DAI池：5000个 (pool_pay_balance = 5000_000000000000000000，18位小数)

// 当前兑换率计算：
// 价格由公式 R * P = k 决定
// 即：1000 * 5000 = 5,000,000 (这是k值)

// 如果用户想用DAI换USDC：
// 用户支付100 DAI后，DAI池变为：5000 + 100 = 5100
// 新的USDC池数量：5,000,000 / 5100 ≈ 980.39
// 用户得到的USDC：1000 - 980.39 = 19.61

// 这意味着大约 100 DAI = 19.61 USDC，即 1 USDC ≈ 5.1 DAI
/// * `Err(ProgramError)` - 计算错误或流动性不足时返回错误
pub fn determine_swap_receive(
    pool_receive_balance: u64,
    receive_decimals: u8,
    pool_pay_balance: u64,
    pay_decimals: u8,
    pay_amount: u64,
) -> Result<u64, ProgramError> {
    // 将整数金额转换为浮点数进行计算
    let big_r = convert_to_float(pool_receive_balance, receive_decimals);
    let big_p = convert_to_float(pool_pay_balance, pay_decimals);
    let p = convert_to_float(pay_amount, pay_decimals);

    // 应用恒定乘积公式计算接收金额
    let bigr_times_p = big_r.mul(p);
    let bigp_plus_p = big_p.add(p);
    let r = bigr_times_p.div(bigp_plus_p);

    // 检查计算结果是否超过池子余额，防止流动性不足
    if r > big_r {
        return Err(ArbitrageProgramError::InvalidSwapNotEnoughLiquidity.into());
    }
    Ok(convert_from_float(r, receive_decimals))
}

/// 将一个无符号64位整数转换为浮点数，并根据指定的小数位数进行缩放
///
/// 该函数通过将输入值除以10的指定次幂来实现小数点的定位，
/// 从而将整数表示的数值转换为具有指定小数位数的浮点数。
///
/// # 参数
/// * `value` - 需要转换的无符号64位整数值
/// * `decimals` - 指定小数点后保留的位数
///
/// # 返回值
/// 返回转换后的32位浮点数，其值等于 value / (10^decimals)
///
/// # 示例
/// ```
/// let result = convert_to_float(12345, 2);
/// // result 等于 123.45
/// ```
fn convert_to_float(value: u64, decimals: u8) -> f32 {
    // 将整数值转换为浮点数并除以10的decimals次幂，实现小数点定位
    (value as f32).div(f32::powf(10.0, decimals as f32))
}

/// 将浮点数转换为整数表示
///
/// 该函数通过将浮点数乘以10的指定次幂，然后转换为u64类型来实现浮点数到整数的转换。
/// 常用于需要将小数位数固定的浮点数转换为整数进行存储或计算的场景。
///
/// # 参数
/// * `value` - 需要转换的浮点数值
/// * `decimals` - 小数位数，决定乘以10的多少次幂
///
/// # 返回值
/// 返回转换后的u64整数值
///
/// # 示例
/// ```
/// let result = convert_from_float(123.45, 2); // 返回 12345
/// ```
fn convert_from_float(value: f32, decimals: u8) -> u64 {
    value.mul(f32::powf(10.0, decimals as f32)) as u64
}
