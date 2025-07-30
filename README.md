### solana 套利脚本

### 什么是套利？
##### 套利就像是在一个地方低价买入，在另一个地方高价卖出来赚钱。比如：

- 在A商店苹果卖1元，在B商店卖2元
- 你从A商店买来苹果，立刻在B商店卖出去
- 每个苹果赚1元差价
- 这个机器人做什么？
- 这个机器人专门在区块链上的去中心化交易所(DEX)寻找这种价格差异：

### 1. 工作原理
Swap Program 1 (商店A)    机器人    Swap Program 2 (商店B)
    USDC 100  ◄-----------▲-----------►  USDC 102
    SOL 1     ◄-----------▼-----------►  SOL 1

### 假设在两个不同的交易所中，同样的代币对（比如USDC/SOL）价格不同：

- 交易所1：1 SOL = 100 USDC
- 交易所2：1 SOL = 102 USDC
#### 机器人就会：
- 在交易所1用100 USDC买1 SOL
- 立刻在交易所2把1 SOL卖成102 USDC
- 赚取2 USDC的差价

## 2. 具体执行流程

### 准备阶段
- **代币账户准备**：机器人提前创建并初始化所有可能用到的代币账户。  
- **资金充足**：确保每个账户都有足够余额，可立即执行买卖。  
- **地址查找表（ALT）**：预先生成并缓存常用账户地址，减少链上查询时间，提升交易速度。

### 执行阶段
1. **价格扫描**  
   并发监控多个交易对（如 SOL/USDC、ETH/USDC 等）在 A、B 两个交易所的最新挂单价格。
2. **价差判断**  
   若发现价差 ≥ 设定阈值（扣除手续费后仍有盈利空间），立即触发套利逻辑。
3. **原子交易**  
   在同一笔链上交易中完成：
   - 在低价交易所买入
   - 在高价交易所卖出
4. **结果确认**  
   交易上链后，检查事件日志，确保两边仓位均已更新，利润自动进入机器人主钱包。

---

## 3. 怎么获取收益？

### 收益来源
- **价差**：同一资产在两个交易所的瞬时价格差。  
  例：A 所 SOL = 170 USDC，B 所 SOL = 172 USDC → 每 SOL 赚 2 USDC（未计费用）。

### 收益过程
1. 7×24 小时持续轮询价格。  
2. 当 `价差 > 手续费 + 滑点容忍度` 时，立即执行套利。  
3. 成交后，净利润（已扣除所有费用）直接入账。

### 关键点
- **速度**：机会窗口通常 < 1 秒，需低延迟、高并发。  
- **手续费**：链上 gas、交易所 taker fee、ALT 租金等均需计入成本。  
- **资金规模**：资金越大，单笔绝对收益越高，但滑点也越大。

---

## 4. 代码中的关键参数

| 参数           | 示例值 | 说明 |
|----------------|--------|------|
| `concurrency`  | 8      | 同时并发检查的代币对数量。 |
| `iterations`   | 2      | 每轮价格扫描后，再重复确认 2 次，减少误报。 |
| `temperature`  | 60     | 随机化因子（0-100）。值越高，随机性越大，降低被预测风险。 |

---

## 5. 风险和挑战

- **竞争激烈**  
  链上可见的 mempool 信息导致“竞价赛跑”（Priority Fee 战）。
- **交易费用**  
  高并发时，gas 费可能瞬间飙升，侵蚀利润。
- **滑点风险**  
  实际成交价与预期价偏差；可设置 `max_slippage_bps` 限制。
- **技术风险**  
  - 智能合约漏洞  
  - RPC 节点延迟或断线  
  - 私钥泄露

### 运行 pnpm run test
```output

  Arbitrage Bot
    ✔ Create a Lookup Table (3590ms)
Lookup Table: 7P8bNnYDLm579WHogpwTacbxoo8KgQa9uqNG9MFMg6AT
   Index: 0   Address: 4Xa4ev8BxofHrrS1gtrZBgUFaFQ5qTDB78TwmyZPjBU4
   Index: 1   Address: 6ZXb9w1vecxcAyPMFitLnxdoNtptQMno5vfGfiDeQzd2
   Index: 2   Address: AATwEKVoaqD2NAcD9su5JyyogJqtTkD7nyaqXbggoWpY
   Index: 3   Address: 5yncTchyHsTRSH7Mjc7D3Phr27Jp8bAHBDQ2FVKo5BrY
   Index: 4   Address: 5UUk5sqshG9btAkwfBSZomeh9o2yLsW72exEKjwH7W88
   Index: 5   Address: 7CaEyZp9Ew2BKU1KfJ3rF5wgCfsjn2hCa2A3g9W7zmhL
   Index: 6   Address: 5joHCiKAea3XD4H9FsGrKxeU16zNDcsSanTxmppt9qez
   Index: 7   Address: FH71PmVAYFcY5hdQnV3wK8V551LyGjgV6Xie3eYTShhM
   Index: 8   Address: CM53QXHSuUjwqFYhx3FYSECCC7FsTfG8tmfpQdFdxmCq
   Index: 9   Address: 3vDJxMArgMoLSKUbZJYNkuX8sWfMdspvgpipCLTVNmhv
   Index: 10  Address: 4q9LRV5t4s4V8FsNTDrLKganPdMacPejgYAy6jpKUS24
   Index: 11  Address: CqpQrU5Lqk4sFABGdbRX5UZLYhgVQrNv2ZffcW795kps
   Index: 12  Address: CtXVMjkYQPXnasoo6EPdWDvgsUh5rmGZH3LFkKfeT3JW
   Index: 13  Address: 2SK9bykHka2JKsmMbEWkFn9iLYZgSFfJkhYcrXKS81D8
   Index: 14  Address: 3MeBsh6hSaE2P8m6FVwXxibSkRAggtAXbe5NA27PSoAV
   Index: 15  Address: 8pMK6ggve1Q91j5Z92REmJ61HkhC4qK6q4aAo6CeNqT9
   Index: 16  Address: JCP2NTDrgAUEp62GbuVXyZu45EzP4ZcHuDxpWkvdCesb
   Index: 17  Address: HMZ4x4BibTPAydBQdRXgjmBeE1JvxqHATSYqG1CpxGmQ
   Index: 18  Address: HCgYJqRer3XCt5WM62gg6xEuKiJd2q9L5hLjdceXzktW
   Index: 19  Address: BVW3Rohb17U8EtwH6dd5ManfUD1MbQHzb2xdSpErDFR7
   Index: 20  Address: 82UNaTtUkKLfPXXhUq41iPcm7KhLck4dnMrZjbGbqFUU
   Index: 21  Address: Guw2QLh1sV3jvmLoFrV9cu2zGJnWDdCH6Ae64HFKFuTf
   Index: 22  Address: 2TK5K3gmLrXRqE9wavvbvpiTpeTixjQJKEoQBL91Y1T9
   Index: 23  Address: u2c59B3Rewkb7wFRVZMVYccHtGPcfEWa5snXchfsd69
   Index: 24  Address: 3mZDHUFsfoiD1zfCXXUXzqNGLDTroPbcFvmwbhX8CJFo
   Index: 25  Address: 68q14z37YYcQAgsFN6Finj4kJEPT8Y4BQNfEwJTg4xJe
   Index: 26  Address: 26Nz139wMBpHKW6UPTXXnKhFwGMQneSUg9aAjny3kg6g
   Index: 27  Address: 24rZcAgvuZvr4MyGPaN4Qi9noYYV4PX6yVE9W7srv3eF
   Index: 28  Address: DoWjg7tsWXW6Fqc1Z7uKmJFPu8VEBPGKtd5WvQ9vfZwC
   Index: 29  Address: HWAZL8d5W5qtMvWCZorgQSwhERBFGEHQL7EzwN45jQne
   Index: 30  Address: 52rD1fEspMvjkvDper3CSiKo8wNcs2VBwBZyVY5VajwM
   Index: 31  Address: GgW4bYkV3mtgUSDEmLEjRQCEvag49caD22bKUdPsEe9r
   Index: 32  Address: 2ZiVLC8kSwkqLeHfGJiNTcfJfM8wWACL98Z6HHZ94N6B
Iteration: 1
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 568
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 568
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 568
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 568
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 564
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 560
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 556
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 552
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 2 accounts...
Tx size with Lookup Table      : 516
====================================
   Arbitrage trade placed!
====================================
Iteration: 2
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 568
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 568
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 568
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 568
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 564
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 560
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 556
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 8 accounts...
Tx size with Lookup Table      : 552
====================================
   Arbitrage trade placed!
====================================
Sending transaction with 2 accounts...
Tx size with Lookup Table      : 516
====================================
   Arbitrage trade placed!
====================================
    ✔ Try Arbitrage (152219ms)


  2 passing (3m)

```