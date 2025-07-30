import * as borsh from 'borsh'
import { Buffer } from 'buffer'
import {
    AccountMeta,
    PublicKey,
    SystemProgram,
    SYSVAR_RENT_PUBKEY,
    TransactionInstruction,
} from '@solana/web3.js'
import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
} from '@solana/spl-token'


/**
 * 获取流动性池的地址
 * 
 * 该函数通过程序ID和预定义的种子字符串来派生出流动性池的唯一地址。
 * 使用Solana的程序派生地址(PDA)机制来确保地址的确定性和安全性。
 * 
 * @param programId - 流动性池程序的公钥ID，用于生成派生地址
 * @returns 返回根据程序ID和种子字符串计算出的流动性池地址
 */
export function getPoolAddress(programId: PublicKey): PublicKey {
    // 使用'liquidity_pool'作为种子字符串生成程序派生地址
    return PublicKey.findProgramAddressSync(
        [Buffer.from('liquidity_pool')],
        programId
    )[0]
}

/**
 * ArbitrageProgramInstruction 类用于构建套利程序的指令数据
 * 该类将套利交易的相关参数序列化为可发送到区块链程序的二进制数据
 */
class ArbitrageProgramInstruction {
    instruction: number
    swap_1_program_id: Uint8Array
    swap_2_program_id: Uint8Array
    concurrency: number
    temperature: number

    /**
     * 构造函数，初始化套利程序指令参数
     * @param props 包含套利交易配置的属性对象
     * @param props.swapProgram1 第一个交换程序的公钥
     * @param props.swapProgram2 第二个交换程序的公钥
     * @param props.concurrency 并发数，控制同时执行的交易数量
     * @param props.temperature 温度参数，用于控制交易的敏感度
     */
    constructor(props: {
        swapProgram1: PublicKey
        swapProgram2: PublicKey
        concurrency: number
        temperature: number
    }) {
        this.instruction = 0
        this.swap_1_program_id = props.swapProgram1.toBuffer()
        this.swap_2_program_id = props.swapProgram2.toBuffer()
        this.concurrency = props.concurrency
        this.temperature = props.temperature
    }

    /**
     * 将当前指令对象序列化为二进制缓冲区
     * 使用 borsh 序列化方案将对象转换为可传输的二进制格式
     * @returns 序列化后的 Buffer 对象，可用于发送到区块链程序
     */
    toBuffer() {
        // 使用 borsh 序列化方案将当前对象序列化为二进制数据
        return Buffer.from(
            borsh.serialize(ArbitrageProgramInstructionSchema, this)
        )
    }
}

/**
 * 定义套利程序指令的数据结构模式
 * 
 * 该模式用于序列化和反序列化套利程序的指令数据，包含执行套利交易所需的核心参数配置。
 * 指令结构定义了两个交换程序的标识、并发级别和温度控制参数等关键字段。
 */
const ArbitrageProgramInstructionSchema = new Map([
    [
        ArbitrageProgramInstruction,
        {
            kind: 'struct',
            fields: [
                ['instruction', 'u8'],
                ['swap_1_program_id', [32]],
                ['swap_2_program_id', [32]],
                ['concurrency', 'u8'],
                ['temperature', 'u8'],
            ],
        },
    ],
])

/**
 * 创建一个默认的账户元数据对象
 * 
 * @param pubkey - 账户的公钥
 * @returns 返回包含账户元数据的对象，其中isSigner为false，isWritable为true
 */
export function defaultAccountMeta(pubkey: PublicKey): AccountMeta {
    return { pubkey, isSigner: false, isWritable: true }
}

/**
 * 创建用于在两个swap程序之间进行套利的交易指令
 * 
 * @param programId - 套利程序的公钥
 * @param payer - 支付交易费用的账户公钥
 * @param tokenAccountsUser - 用户的代币账户公钥数组
 * @param tokenAccountsSwap1 - 第一个swap程序的代币账户公钥数组
 * @param tokenAccountsSwap2 - 第二个swap程序的代币账户公钥数组
 * @param mints - 涉及的代币mint账户公钥数组
 * @param concurrency - 并发参数，控制套利指令的并发级别
 * @param temperature - 温度参数，控制套利指令的敏感度
 * @param swapProgram1 - 第一个swap程序的公钥
 * @param swapProgram2 - 第二个swap程序的公钥
 * @returns 表示套利操作的TransactionInstruction对象
 */
export function createArbitrageInstruction(
    programId: PublicKey,
    payer: PublicKey,
    tokenAccountsUser: PublicKey[],
    tokenAccountsSwap1: PublicKey[],
    tokenAccountsSwap2: PublicKey[],
    mints: PublicKey[],
    concurrency: number,
    temperature: number,
    swapProgram1: PublicKey,
    swapProgram2: PublicKey
): TransactionInstruction {
    // 获取两个swap程序的流动性池地址
    let swapPool1 = getPoolAddress(swapProgram1)
    let swapPool2 = getPoolAddress(swapProgram2)

    // 创建包含swap程序ID和参数的指令数据缓冲区
    const data = new ArbitrageProgramInstruction({
        swapProgram1,
        swapProgram2,
        concurrency,
        temperature,
    }).toBuffer()

    // 初始化账户元数据数组，包含所需的系统和程序账户
    let keys: AccountMeta[] = [
        // 支付账户
        { pubkey: payer, isSigner: true, isWritable: true },
        // 代币程序
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        // 系统程序
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        // 关联代币程序
        {
            pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
        },
        // Swap #1 程序
        { pubkey: swapProgram1, isSigner: false, isWritable: false },
        // Swap #2 程序
        { pubkey: swapProgram2, isSigner: false, isWritable: false },
        // Swap #1 的流动性池
        defaultAccountMeta(swapPool1),
        // Swap #2 的流动性池
        defaultAccountMeta(swapPool2),
    ]

    // 添加用户的代币账户到指令中
    tokenAccountsUser.forEach((a) => keys.push(defaultAccountMeta(a)))

    // 添加第一个swap程序的代币账户
    tokenAccountsSwap1.forEach((a) => keys.push(defaultAccountMeta(a)))

    // 添加第二个swap程序的代币账户
    tokenAccountsSwap2.forEach((a) => keys.push(defaultAccountMeta(a)))

    // 添加所有涉及代币的mint账户
    mints.forEach((a) => keys.push(defaultAccountMeta(a)))

    return new TransactionInstruction({
        keys,
        programId,
        data,
    })
}