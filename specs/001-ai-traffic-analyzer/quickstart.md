# Quick Start: RustSniffer-AI

**Branch**: `001-ai-traffic-analyzer` | **Date**: 2026-07-16

## Prerequisites

- Rust stable 工具链（1.75+）
- libpcap-dev（Linux）或 Npcap（Windows）
- nDPI 库（用于 DPI 功能）

## Build

```bash
cargo build --release
```

## Quick Usage

### 1. 列出可用网卡
```bash
rustsniffer list-interfaces
```

### 2. 启动基础抓包
```bash
# 默认网卡，实时显示五元组
rustsniffer capture

# 指定网卡和过滤器
rustsniffer capture -i eth0 -f "port 80"
```

### 3. 查询历史数据
```bash
# 查询最近的 DNS 请求
rustsniffer query "SELECT * FROM packets WHERE app_protocol='DNS'" --limit 10

# 统计 Top 10 IP
rustsniffer query "SELECT src_ip, COUNT(*) as cnt FROM packets GROUP BY src_ip ORDER BY cnt DESC LIMIT 10"
```

### 4. 查看统计信息
```bash
rustsniffer stats --period 1h
```

## Configuration

配置文件位于 `~/.rustsniffer/config.toml`，首次运行自动创建默认配置。

## Data Storage

- 元数据：`~/.rustsniffer/metadata.db`（SQLite）
- 原始包：`~/.rustsniffer/pcap/YYYY-MM-DD.pcapng`
- 日志：`~/.rustsniffer/logs/`

## Development

```bash
# 运行测试
cargo test

# 代码检查
cargo clippy --all-targets --all-features -- -D warnings

# 格式化检查
cargo fmt --check
```
