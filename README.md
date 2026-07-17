# RustSniffer

Rust 智能抓包与流量分析工具 —— 基于深度包检测（DPI）的高性能网络流量分析原型。

## 功能概览

### 当前版本（MVP）

- **网络抓包**：基于 libpcap/Npcap，支持混杂模式、BPF 过滤、自动网卡枚举
- **协议解析**：Ethernet、IPv4/IPv6、TCP/UDP、ICMP 全栈解析
- **深度包检测（DPI）**：基于 Payload 特征识别应用层协议（HTTP、DNS、TLS、SSH 等），不依赖端口号
- **TLS 指纹**：提取 JA3/JA3S 指纹用于加密流量识别
- **元数据存储**：SQLite 本地数据库，支持按时间、协议、IP 等条件快速查询
- **原始包存储**：PCAPNG 格式按天切片，支持通过元数据索引回溯原始包
- **结构化日志**：JSON 格式，文件轮转（100MB/文件，保留 5 个）

### 规划中（v1.1+）

- 自然语言查询（Text-to-BPF/SQL）
- 流量语义总结与异常行为检测
- TUI 实时监控面板
- 本地/云端 LLM 集成

## 系统要求

### 通用

- **Rust**：stable 1.75+（[安装指南](https://www.rust-lang.org/tools/install)）
- **操作系统**：Linux / Windows

### Windows 额外依赖

需要安装 [Npcap](https://npcap.com/)（替代 WinPcap）：

1. 前往 https://npcap.com/#download 下载安装包
2. 安装时勾选 **"Install Npcap in WinPcap API-compatible Mode"**
3. 安装完成后重启终端

> 程序启动时会自动检测并添加 Npcap DLL 路径到 PATH，无需手动配置。

### Linux 额外依赖

```bash
# Debian/Ubuntu
sudo apt install libpcap-dev

# Fedora
sudo dnf install libpcap-devel

# Arch
sudo pacman -S libpcap
```

## 构建与运行

```bash
# 克隆项目
git clone <repo-url>
cd RustSniffer-AI

# 构建
cargo build --release

# 运行（需要管理员/root 权限来抓包）
# Windows（以管理员身份运行 CMD/PowerShell）：
cargo run --release

# Linux
sudo cargo run --release
```

## 命令行用法

```bash
# 列出可用网卡
rustsniffer list-interfaces

# 直接抓包（自动选择第一个网卡）
rustsniffer

# 指定网卡和过滤器
rustsniffer -i <网卡名> -f "port 80"

# 使用 capture 子命令
rustsniffer capture -i <网卡名> -f "tcp port 443"

# 限制捕获数量
rustsniffer -c 1000

# 限制捕获时长（秒）
rustsniffer -d 60
```

### 参数说明

| 参数 | 短选项 | 说明 | 默认值 |
|------|--------|------|--------|
| `--interface` | `-i` | 网卡名称 | 自动选择第一个 |
| `--filter` | `-f` | BPF 过滤表达式 | 无（捕获全部） |
| `--promiscuous` | `-P` | 启用混杂模式 | `true` |
| `--count` | `-c` | 最大捕获包数（0 = 无限制） | `0` |
| `--duration` | `-d` | 最大捕获时长/秒（0 = 无限制） | `0` |

按 `Ctrl+C` 优雅停止抓包，输出统计摘要（总包数、丢包率、捕获时长）。

## 配置

采用分层配置策略：**命令行参数 > 配置文件 > 默认值**

配置文件位置：`~/.rustsniffer/config.toml`

```toml
[capture]
interface = "eth0"            # 默认网卡（留空自动选择）
filter = ""                   # 默认 BPF 过滤器
promiscuous = true            # 混杂模式
count = 0                     # 最大捕获包数
duration = 0                  # 最大捕获时长（秒）
buffer_size = 2097152         # 抓包缓冲区（2MB）
sampling_threshold = 100000   # 采样阈值（每秒包数）

[storage]
db_path = ""                  # 数据库路径（默认 ~/.rustsniffer/metadata.db）
pcap_dir = ""                 # PCAPNG 目录（默认 ~/.rustsniffer/pcap/）
retention_days = 30           # 数据保留天数
batch_size = 1000             # 批量插入大小
batch_interval_ms = 100       # 批量插入间隔（毫秒）

[logging]
level = "info"                # 日志级别：trace/debug/info/warn/error
log_dir = ""                  # 日志目录（默认 ~/.rustsniffer/logs/）
max_log_size_mb = 100         # 单个日志文件最大大小（MB）
max_log_files = 5             # 保留的日志文件数量
json_format = false           # 是否输出 JSON 格式
```

## 数据存储

```
~/.rustsniffer/
├── config.toml          # 配置文件
├── metadata.db          # SQLite 元数据数据库
├── pcap/                # 原始数据包（PCAPNG，按天切片）
│   ├── 2026-07-16.pcapng
│   └── 2026-07-17.pcapng
└── logs/                # 运行日志
    ├── rustsniffer.log
    └── rustsniffer.log.1
```

- 元数据和 PCAPNG 文件自动保留 30 天，超期自动清理
- 支持通过元数据索引回溯到对应的原始数据包

## 项目结构

```
src/
├── main.rs              # 程序入口
├── lib.rs               # 核心库导出
├── capture/             # 采集层 - 网卡枚举、BPF 过滤、抓包引擎
│   ├── interface.rs     #   网卡发现与选择
│   ├── bpf.rs           #   BPF 过滤器编译与验证
│   └── sniffer.rs       #   抓包核心逻辑
├── protocol/            # 协议解析层
│   ├── ethernet.rs      #   Ethernet 帧解析
│   ├── ip.rs            #   IPv4/IPv6 解析
│   ├── tcp.rs           #   TCP 解析与流重组
│   ├── udp.rs           #   UDP 解析
│   ├── icmp.rs          #   ICMP 解析
│   └── parser.rs        #   协议解析调度器
├── dpi/                 # 深度包检测
│   ├── engine.rs        #   DPI 引擎
│   ├── classifier.rs    #   协议分类器
│   ├── ja3.rs           #   JA3/JA3S 指纹提取
│   └── ndpi.rs          #   nDPI FFI 绑定
├── storage/             # 存储层
│   ├── metadata.rs      #   元数据数据库（SQLite）
│   ├── packet_store.rs  #   原始包 PCAPNG 存储
│   ├── query.rs         #   查询接口
│   └── cleanup.rs       #   过期数据自动清理
├── config/              # 配置管理
│   ├── settings.rs      #   配置结构定义
│   └── loader.rs        #   配置加载（TOML）
├── logging/             # 日志系统
│   └── setup.rs         #   日志初始化与轮转
├── cli/                 # 命令行界面
│   ├── args.rs          #   参数定义（clap）
│   └── commands.rs      #   子命令实现
└── common/              # 公共模块
    ├── types.rs         #   公共类型定义
    ├── error.rs         #   错误类型
    └── utils.rs         #   工具函数
```

## 技术栈

| 组件 | 选型 | 说明 |
|------|------|------|
| 抓包 | `pcap` | libpcap/Npcap 绑定 |
| 协议解析 | `etherparse` | 纯 Rust，无 C 依赖 |
| DPI | `nDPI`（FFI） | 开源 DPI 引擎 |
| 异步 | `tokio` | 异步运行时 |
| 数据库 | `rusqlite` | SQLite 绑定（bundled） |
| CLI | `clap` | 参数解析 |
| 日志 | `tracing` + `tracing-subscriber` | 结构化日志 |
| 序列化 | `serde` / `serde_json` / `toml` | 数据序列化 |

## 开发

```bash
# 检查代码
cargo clippy

# 运行测试
cargo test

# Debug 构建
cargo build
```

## 许可证

MIT
