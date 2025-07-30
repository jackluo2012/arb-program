import {
    createAssociatedTokenAccountInstruction,
    createInitializeMintInstruction,
    createMintToInstruction,
    getAssociatedTokenAddressSync,
    MINT_SIZE,
    TOKEN_PROGRAM_ID,
} from '@solana/spl-token'
import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js'
import { buildTransactionV0 } from './transaction'


/**
 * 将数量转换为大整数格式的数量
 * 
 * @param quantity - 需要转换的数量值
 * @param decimals - 小数位数，用于确定数量的精度
 * @returns 返回转换后的大整数数量，计算方式为 quantity * 10^decimals
 */
export function toBigIntQuantity(quantity: number, decimals: number): bigint {
    // 将数量乘以10的decimals次方，转换为bigint类型
    return BigInt(quantity) * BigInt(10) ** BigInt(decimals)
}

/**
 * 将bigint类型的数量值转换为带小数位的字符串表示
 * 
 * @param quantity - 以最小单位表示的数量值（如wei、satoshi等）
 * @param decimals - 该数量值的小数位数，用于转换为标准单位
 * @returns 转换后的字符串，保留6位小数
 */
export function fromBigIntQuantity(quantity: bigint, decimals: number): string {
    // 将bigint转换为数字，除以10的decimals次方得到标准单位值，然后保留6位小数
    return (Number(quantity) / 10 ** decimals).toFixed(6)
}
/**
 * 铸造现有代币到指定账户
 * 
 * @param connection - Solana网络连接对象
 * @param payer - 支付交易费用的账户密钥对
 * @param mint - 代币铸造地址
 * @param quantity - 要铸造的代币数量
 * @param decimals - 代币的小数位数
 */
export async function mintExistingTokens(
    connection: Connection,
    payer: Keypair,
    mint: PublicKey,
    quantity: number,
    decimals: number
) {
    // 获取关联代币账户地址
    const tokenAccount = getAssociatedTokenAddressSync(mint, payer.publicKey)

    // 创建铸币指令
    const mintToWalletIx = createMintToInstruction(
        mint,
        tokenAccount,
        payer.publicKey,
        toBigIntQuantity(quantity, decimals)
    )

    // 构建并发送交易
    const tx = await buildTransactionV0(
        connection,
        [mintToWalletIx],
        payer.publicKey,
        [payer]
    )
    await connection.sendTransaction(tx)
}
