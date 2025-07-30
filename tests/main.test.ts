import {
    getAccount as getTokenAccount,
    getAssociatedTokenAddressSync,
    TokenAccountNotFoundError,
    createAccount as createTokenAccount,
} from "@solana/spl-token";
import { sleepSeconds } from "./util";
import { CONNECTION, PAYER, ARBITRAGE_PROGRAM } from "./util/const";
import { createArbitrageInstruction, getPoolAddress } from './util/instruction'
import { PublicKey, SendTransactionError } from '@solana/web3.js'
import assetsConfig from './util/assets.json'
import {
    buildTransactionV0,
    buildTransactionV0WithLookupTable,
    createAddressLookupTable,
    extendAddressLookupTable,
    printAddressLookupTable,
} from './util/transaction'
import { before, describe, it } from 'mocha'
const SWAP_PROGRAM_1 = new PublicKey(
    '5koF84vG5xwah17PNRyge3HmqdJZ4rqdqPvZnMKqi8Bq'
)
const SWAP_PROGRAM_2 = new PublicKey(
    'DRP4K7yv8EBftb3roP81idoPtRDJwpak1Apw8d4Df14T'
)

/**
 * 定义温度常量，用于控制算法的随机性程度
 * 值为60
 */
const temperature = 60

/**
 * 定义并发数常量，表示同时执行的任务数量
 * 值为8
 */
const concurrency = 8

/**
 * 定义迭代次数常量，控制算法的执行轮数
 * 值为2
 */
const iterations = 2

describe("Arbitrage Bot ", () => {
    const connection = CONNECTION;
    const payer = PAYER;
    const arbProgram = ARBITRAGE_PROGRAM;
    const assets = assetsConfig.assets.map((o) => {
        return {
            name: o.name,
            quantity: o.quantity,
            decimals: o.decimals,
            address: new PublicKey(o.address),
        }
    })

    let lookupTable: PublicKey
    let tokenAccountsUser: PublicKey[] = []
    let tokenAccountsSwap1: PublicKey[] = []
    let tokenAccountsSwap2: PublicKey[] = []
    let mints: PublicKey[] = []
    before(
        'Collect all token accounts & mints and mint some assets to the payer if necessary',
        async () => {
            for (const a of assets) {
                /**
  * 获取关联的代币地址
  * @param a.address - 代币的mint地址
  * @param payer.publicKey - 支付者的公钥
  * @returns 返回与用户公钥关联的代币地址
  */
                const tokenAddressUser = getAssociatedTokenAddressSync(
                    a.address,
                    payer.publicKey
                )
                try {

                    /**
                     * 获取代币账户信息
                     * @param connection - 区块链网络连接对象，用于与链上节点进行通信
                     * @param tokenAddressUser - 用户代币地址，用于查询对应的代币账户信息
                     * @returns Promise对象，解析后返回代币账户信息
                     */
                    const tokenAccount = await getTokenAccount(
                        connection,
                        tokenAddressUser
                    )
                    if (tokenAccount.amount === BigInt(0)) {
                        // 如何代币信息不存在，就铸造10个代币
                        await mintExistingTokens(
                            connection,
                            payer,
                            a.address,
                            10,
                            a.decimals
                        )
                    }
                } catch (e) {
                    // 检测到是帐号不存在
                    if (e === TokenAccountNotFoundError) {
                        // 创建token 帐号
                        await createTokenAccount(
                            connection,
                            payer,
                            a.address,
                            payer.publicKey
                        )
                        // 铸造10个代币
                        await mintExistingTokens(
                            connection,
                            payer,
                            a.address,
                            10,
                            a.decimals
                        )
                    }
                }
                // Add each account to its respective list
                tokenAccountsUser.push(tokenAddressUser)
                /**
                 * 为两个不同的swap程序获取关联的代币账户地址，并收集代币铸造地址
                 * 
                 * 该代码块执行以下操作：
                 * 1. 为SWAP_PROGRAM_1获取池地址对应的关联代币账户地址，并添加到tokenAccountsSwap1数组
                 * 2. 为SWAP_PROGRAM_2获取池地址对应的关联代币账户地址，并添加到tokenAccountsSwap2数组
                 * 3. 收集当前处理的代币铸造地址到mints数组
                 */

                tokenAccountsSwap1.push(
                    getAssociatedTokenAddressSync(
                        a.address,
                        getPoolAddress(SWAP_PROGRAM_1),
                        true
                    )
                )
                tokenAccountsSwap2.push(
                    getAssociatedTokenAddressSync(
                        a.address,
                        getPoolAddress(SWAP_PROGRAM_2),
                        true
                    )
                )
                mints.push(a.address)
            }
        }
    )

    /**
     * 测试用例：创建地址查找表
     * 该函数创建一个地址查找表，并向其中添加多组地址信息，用于后续的交易优化
     */
    it('Create a Lookup Table', async () => {
        // 创建新的地址查找表
        lookupTable = await createAddressLookupTable(connection, payer)
        await sleepSeconds(2)

        /**
         * 内联函数：扩展地址查找表
         * 将指定的地址数组添加到已有的查找表中
         * @param addresses - 需要添加到查找表的公钥地址数组
         */
        async function inlineExtend(addresses: PublicKey[]) {
            await extendAddressLookupTable(
                connection,
                payer,
                lookupTable,
                addresses
            )
            await sleepSeconds(2)
        }

        // 依次将用户代币账户、交换池1代币账户、交换池2代币账户和铸币地址添加到查找表中
        inlineExtend(tokenAccountsUser)
        inlineExtend(tokenAccountsSwap1)
        inlineExtend(tokenAccountsSwap2)
        inlineExtend(mints)

        // 打印最终的地址查找表信息
        printAddressLookupTable(connection, lookupTable)
    })

    /**
     * 发送套利交易指令，构建并发送包含套利逻辑的交易。
     * 
     * @param tokenAccountsUserSubList - 用户代币账户公钥列表
     * @param tokenAccountsSwap1SubList - 第一个交换程序的代币账户公钥列表
     * @param tokenAccountsSwap2SubList - 第二个交换程序的代币账户公钥列表
     * @param mintsSubList - 代币铸造账户公钥列表
     * @param concurrencyVal - 并发处理的账户数量
     * @returns 无返回值，但会通过控制台输出交易状态和结果
     */
    async function sendArbitrageInstruction(
        tokenAccountsUserSubList: PublicKey[],
        tokenAccountsSwap1SubList: PublicKey[],
        tokenAccountsSwap2SubList: PublicKey[],
        mintsSubList: PublicKey[],
        concurrencyVal: number
    ) {
        // 创建套利指令，传入所有必要的账户和参数
        const ix = createArbitrageInstruction(
            arbProgram.publicKey,
            payer.publicKey,
            tokenAccountsUserSubList,
            tokenAccountsSwap1SubList,
            tokenAccountsSwap2SubList,
            mintsSubList,
            concurrencyVal,
            temperature,
            SWAP_PROGRAM_1,
            SWAP_PROGRAM_2
        )

        // 使用查找表构建版本化交易（V0），以减少交易大小
        const tx = await buildTransactionV0WithLookupTable(
            connection,
            [ix],
            payer.publicKey,
            [payer],
            lookupTable
        )
        // const txNoLT = await buildTransactionV0(
        //     connection,
        //     [ix],
        //     payer.publicKey,
        //     [payer]
        // )

        // 输出交易信息，包括并发账户数和使用查找表后的交易大小
        console.log(`Sending transaction with ${concurrencyVal} accounts...`)
        console.log(`Tx size with Lookup Table      : ${tx.serialize().length}`)
        // console.log(
        //     `Tx size WITHOUT Lookup Table   : ${txNoLT.serialize().length}`
        // )

        try {
            // 发送交易，并跳过预执行检查以提高性能
            await connection.sendTransaction(tx, { skipPreflight: true })
            console.log('====================================')
            console.log('   Arbitrage trade placed!')
            console.log('====================================')
        } catch (error) {
            // 捕获交易发送错误，并根据错误类型进行处理
            if (error instanceof SendTransactionError) {
                // 如果是自定义程序错误且错误码为0x3，则表示未找到套利机会
                if (error.message.includes('custom program error: 0x3')) {
                    console.log('====================================')
                    console.log('   No arbitrage opportunity found')
                    console.log('====================================')
                } else {
                    throw error
                }
            } else {
                throw error
            }
        }

        // 等待两秒，避免频繁发送交易
        await sleepSeconds(2)
    }

    /**
     * 测试用例：尝试执行套利操作
     * 
     * 该测试用例会根据设定的迭代次数和并发数，遍历资产组合并发送套利指令。
     * 在每次迭代中，它会将资产按并发数量分批处理，并调用 `sendArbitrageInstruction` 发送套利请求。
     * 
     * 注意：此函数为异步函数，使用了 sleepSeconds 来控制执行节奏。
     */
    it('Try Arbitrage', async () => {
        await sleepSeconds(4)

        // 外层循环：按照设定的迭代次数重复执行套利逻辑
        for (let x = 0; x < iterations; x++) {
            console.log(`Iteration: ${x + 1}`)
            let len = mints.length

            // 初始化变量用于分批处理资产组合
            let step = 0
            let brake = concurrency
            let tokenAccountsUserSubList = []
            let tokenAccountsSwap1SubList = []
            let tokenAccountsSwap2SubList = []
            let mintsSubList = []

            // 中层循环：构建资产组合（i, j）并分批发送套利指令
            for (let i = 0; i < len; i++) {
                for (let j = i; j < len; j++) {

                    // 当前批次已满，发送套利指令并重置子列表
                    if (step == brake) {
                        const end = brake + concurrency
                        await sendArbitrageInstruction(
                            tokenAccountsUserSubList,
                            tokenAccountsSwap1SubList,
                            tokenAccountsSwap2SubList,
                            mintsSubList,
                            end - brake
                        )
                        await sleepSeconds(2)
                        brake = end
                        tokenAccountsUserSubList = []
                        tokenAccountsSwap1SubList = []
                        tokenAccountsSwap2SubList = []
                        mintsSubList = []
                    }

                    // 将当前资产加入待处理队列
                    tokenAccountsUserSubList.push(tokenAccountsUser[j])
                    tokenAccountsSwap1SubList.push(tokenAccountsSwap1[j])
                    tokenAccountsSwap2SubList.push(tokenAccountsSwap2[j])
                    mintsSubList.push(mints[j])
                    step++
                }
            }

            // 发送剩余未处理的资产组合
            if (mintsSubList.length !== 0) {
                await sendArbitrageInstruction(
                    tokenAccountsUserSubList,
                    tokenAccountsSwap1SubList,
                    tokenAccountsSwap2SubList,
                    mintsSubList,
                    mintsSubList.length
                )
            }
        }
    })
});
