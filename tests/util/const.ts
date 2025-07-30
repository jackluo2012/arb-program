import { Connection } from '@solana/web3.js'
import { loadKeypairFromFile } from '.'
import os from 'os'

// RPC connection
// export const CONNECTION = new Connection('http://localhost:8899', 'confirmed')
export const CONNECTION = new Connection(
    'https://api.devnet.solana.com',
    'confirmed'
)


/**
 * 从Solana配置文件中加载支付者的密钥对
 * 
 * 该函数从用户的Solana配置目录中读取默认的密钥对文件，
 * 用于身份验证和交易签名
 * 
 * @returns 返回加载的密钥对对象，用于Solana区块链交互
 */
export const PAYER = loadKeypairFromFile(os.homedir + '/.config/solana/id.json')

// Arbitrage program
/**
 * 从指定路径加载套利程序的密钥对文件
 * 
 * 该函数用于加载部署在目标目录下的套利程序密钥对配置文件，
 * 通常用于区块链智能合约的本地开发和测试环境配置。
 * 
 * @returns 返回加载的密钥对对象，用于程序部署和交互
 */
export const ARBITRAGE_PROGRAM = loadKeypairFromFile(
    './target/deploy/arb_program-keypair.json'
)
