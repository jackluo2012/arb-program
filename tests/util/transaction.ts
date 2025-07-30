import {
    Connection,
    Keypair,
    PublicKey,
    TransactionInstruction,
    VersionedTransaction,
    TransactionMessage,
    AddressLookupTableProgram,
    AddressLookupTableAccount,
} from '@solana/web3.js'
import { sleepSeconds } from '.'


/**
 * 创建地址查找表
 * 
 * @param connection - Solana网络连接对象，用于与区块链交互
 * @param payer - 支付账户的密钥对，用于支付交易费用和作为查找表的权限
 * @returns 返回新创建的地址查找表的公钥
 */
export async function createAddressLookupTable(
    connection: Connection,
    payer: Keypair
): Promise<PublicKey> {
    // 获取最新的slot，使用'max'选项确保地址推导的一致性
    let recentSlot = await connection.getSlot('max')

    // 创建查找表的初始化指令和查找表地址
    let [createLookupTableIx, lookupTable] =
        /**
         * 创建地址查找表程序的查找表
         * 
         * @param authority - 查找表的权限公钥，用于控制查找表的修改权限
         * @param payer - 支付交易费用的账户公钥
         * @param recentSlot - 最近的slot编号，用于确保查找表基于最新的链状态创建
         * @returns 返回创建查找表的指令和查找表地址
         */
        AddressLookupTableProgram.createLookupTable({
            authority: payer.publicKey,  // 谁有权往这个"通讯录"里添加新联系人
            payer: payer.publicKey,      // 谁来付创建这个"通讯录"的钱
            recentSlot,                  // 基于哪个时间点创建(保证数据新鲜)
        })

    // 构建并发送创建查找表的交易
    const tx = await buildTransactionV0(
        connection,
        [createLookupTableIx],
        payer.publicKey,
        [payer]
    )
    await connection.sendTransaction(tx)

    return lookupTable
}


/**
 * 扩展地址查找表，向指定的查找表中添加新的地址
 * @param connection - Solana RPC连接对象，用于与区块链网络交互
 * @param payer - 支付交易费用的账户密钥对
 * @param lookupTable - 要扩展的地址查找表的公钥
 * @param addresses - 要添加到查找表中的地址数组
 * @returns Promise<void> - 无返回值的异步函数
 */
export async function extendAddressLookupTable(
    connection: Connection,
    payer: Keypair,
    lookupTable: PublicKey,
    addresses: PublicKey[]
): Promise<void> {
    // 构造扩展查找表的指令
    let extendLookupTableIx = AddressLookupTableProgram.extendLookupTable({
        addresses,
        authority: payer.publicKey,
        lookupTable,
        payer: payer.publicKey,
    })

    // 构建并发送交易
    const tx = await buildTransactionV0(
        connection,
        [extendLookupTableIx],
        payer.publicKey,
        [payer]
    )
    await connection.sendTransaction(tx)
}

/**
 * 获取地址查找表账户信息
 * 
 * @param connection - Solana网络连接对象，用于与区块链网络进行交互
 * @param lookupTablePubkey - 地址查找表的公钥，用于标识要查询的查找表
 * @returns 返回解析后的地址查找表账户信息
 */
export async function getAddressLookupTable(
    connection: Connection,
    lookupTablePubkey: PublicKey
): Promise<AddressLookupTableAccount> {
    // 通过连接对象获取地址查找表，并返回解析后的账户值
    return connection
        .getAddressLookupTable(lookupTablePubkey)
        .then((res) => res.value)
}


/**
 * 打印地址查找表的详细信息
 * 
 * @param connection - Solana网络连接对象，用于与区块链交互
 * @param lookupTablePubkey - 查找表的公钥地址
 * @returns Promise<void> - 无返回值的异步函数
 */
export async function printAddressLookupTable(
    connection: Connection,
    lookupTablePubkey: PublicKey
): Promise<void> {
    // 等待2秒以避免查找表获取延迟问题
    await sleepSeconds(2)

    // 获取指定公钥的地址查找表账户信息
    const lookupTableAccount = await getAddressLookupTable(
        connection,
        lookupTablePubkey
    )

    // 打印查找表公钥和其中包含的所有地址信息
    console.log(`Lookup Table: ${lookupTablePubkey}`)
    for (let i = 0; i < lookupTableAccount.state.addresses.length; i++) {
        const address = lookupTableAccount.state.addresses[i]
        console.log(
            `   Index: ${i
                .toString()
                .padEnd(2)}  Address: ${address.toBase58()}`
        )
    }
}


/**
 * 构建并返回一个版本化交易对象
 * 
 * @param connection - Solana网络连接对象，用于获取最新的区块哈希
 * @param instructions - 交易指令数组，包含要执行的所有操作
 * @param payer - 付款方的公钥，用于支付交易费用
 * @param signers - 签名者密钥对数组，用于对交易进行签名
 * @returns 返回签名后的版本化交易对象
 */
export async function buildTransactionV0(
    connection: Connection,
    instructions: TransactionInstruction[],
    payer: PublicKey,
    signers: Keypair[]
): Promise<VersionedTransaction> {
    // 获取最新的区块哈希用于交易构建
    let blockhash = await connection
        .getLatestBlockhash()
        .then((res) => res.blockhash)

    // 构建V0版本的交易消息
    const messageV0 = new TransactionMessage({
        payerKey: payer,
        recentBlockhash: blockhash,
        instructions,
    }).compileToV0Message()

    // 创建版本化交易并使用所有签名者进行签名
    const tx = new VersionedTransaction(messageV0)
    signers.forEach((s) => tx.sign([s]))
    return tx
}
/**
 * 构建使用地址查找表的V0版本交易
 * 
 * @param connection - Solana连接对象，用于与区块链网络交互
 * @param instructions - 交易指令数组，包含要执行的操作
 * @param payer - 交易支付者的公钥
 * @param signers - 签名者密钥对数组，用于对交易进行签名
 * @param lookupTablePubkey - 地址查找表的公钥
 * @returns 返回构建好的V0版本交易对象
 */
export async function buildTransactionV0WithLookupTable(
    connection: Connection,
    instructions: TransactionInstruction[],
    payer: PublicKey,
    signers: Keypair[],
    lookupTablePubkey: PublicKey
): Promise<VersionedTransaction> {
    // 等待2秒以避免查找表获取延迟问题
    await sleepSeconds(2)

    // 获取地址查找表账户信息
    const lookupTableAccount = await getAddressLookupTable(
        connection,
        lookupTablePubkey
    )

    // 获取最新的区块哈希
    let blockhash = await connection
        .getLatestBlockhash()
        .then((res) => res.blockhash)

    // 使用查找表编译V0消息
    const messageV0 = new TransactionMessage({
        payerKey: payer,
        recentBlockhash: blockhash,
        instructions,
    }).compileToV0Message([lookupTableAccount])

    // 创建版本化交易并使用签名者签名
    const tx = new VersionedTransaction(messageV0)
    signers.forEach((s) => tx.sign([s]))
    return tx
}