import { Keypair } from '@solana/web3.js'
import fs from 'fs'


/**
 * 从指定文件路径加载密钥对
 * @param path - 包含密钥数据的文件路径
 * @returns 解析文件内容后创建的Keypair对象
 */
export function loadKeypairFromFile(path: string): Keypair {
    // 从文件读取密钥数据，解析JSON格式内容，并转换为Buffer后创建Keypair对象
    return Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync(path, 'utf-8')))
    )
}

/**
 * 延迟执行指定秒数的异步函数
 * @param s 延迟的秒数
 * @returns 返回一个Promise，在指定秒数后resolve
 */
export function sleepSeconds(s: number) {
    // 将秒数转换为毫秒数，创建一个在指定时间后resolve的Promise
    return new Promise((resolve) => setTimeout(resolve, s * 1000))
}