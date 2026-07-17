# Implementation Plan: Rust 智能抓包与流量分析工具

**Branch**: `001-ai-traffic-analyzer` | **Date**: 2026-07-16 | **Spec**: [spec.md](file:///d:/ai-coding/RustSniffer-AI/specs/001-ai-traffic-analyzer/spec.md)
**Input**: Feature specification from `/specs/001-ai-traffic-analyzer/spec.md`

## Summary

构建一款基于 Rust 的高性能网络流量分析工具，支持深度包检测（DPI）和 AI 智能分析。MVP 阶段聚焦基础抓包、协议解析与 DPI 分类、元数据存储三大核心功能，采用分层异步架构，确保内存安全和低延迟。

## Technical Context

**Language/Version**: Rust stable（最新稳定版，建议 1.75+）  
**Primary Dependencies**: 
- `pcap` (libpcap 绑定) - 网络抓包
- `etherparse` - 协议解析（纯 Rust 实现）
- `nDPI` (FFI 绑定) - 深度包检测引擎
- `tokio` - 异步运行时
- `rusqlite` - SQLite 元数据存储
- `clap` - CLI 参数解析
- `ratatui` - TUI 界面（MVP 后）
- `serde` + `serde_json` - 序列化
- `tracing` + `tracing-subscriber` - 结构化日志
- `toml` - 配置文件解析

**Storage**: 
- SQLite (`~/.rustsniffer/metadata.db`) - 结构化元数据
- PCAPNG 文件 (`~/.rustsniffer/pcap/`) - 原始数据包，按天切片

**Testing**: `cargo test`（单元测试 + 集成测试），测试覆盖率 ≥ 80%  
**Target Platform**: Linux（主要），Windows（次要）  
**Project Type**: CLI 工具（带 TUI 能力，MVP 后实现）  
**Performance Goals**: 
- 丢包率 < 0.1%（千兆网卡）
- DPI 分类延迟 < 1ms/包（95 分位）
- 数据库查询 < 1 秒（百万级记录）

**Constraints**: 
- 内存安全：连续运行 24 小时内存增长 < 5%
- 所有 `unsafe` 代码 MUST 严格封装
- 敏感数据 MUST 脱敏后发送

**Scale/Scope**: 单节点部署，处理千兆网络流量，MVP 聚焦 P1-P2 功能

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### ✅ 原则 I: 高度可用性
- 代码 MUST 通过 `cargo build` 和 `cargo clippy` 无错误
- MUST 避免 `unwrap()` 在生产代码中使用
- MUST 确保数据包捕获和存储完整性

### ✅ 原则 II: 安全性优先
- 内存安全：充分利用 Rust 所有权机制
- FFI 安全：nDPI 调用 MUST 封装为安全 Rust API
- 敏感数据：MVP 阶段暂不涉及外部 AI，后续版本实现脱敏

### ✅ 原则 III: 代码可读性与注释规范
- 公开 API MUST 包含 `///` 文档注释（中文）
- 核心逻辑 MUST 添加行内注释解释设计意图
- 模块文件 MUST 在顶部包含 `//!` 模块说明

### ✅ 原则 IV: 易用性与用户体验
- CLI 参数 MUST 提供清晰帮助信息
- 错误信息 MUST 描述问题原因和解决建议
- TUI 界面 MVP 后实现，确保响应迅速

### ✅ 原则 V: 可维护性与可扩展性
- 分层架构：采集层、处理层、存储层、AI 层（MVP 后）、交互层
- Trait 抽象：协议解析器、DPI 引擎 MUST 定义为 Trait 接口
- 测试覆盖：核心功能 MUST 编写单元测试，关键路径 MUST 编写集成测试

**Gate Status**: ✅ 通过，无违规项

## Project Structure

### Documentation (this feature)

```text
specs/001-ai-traffic-analyzer/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── cli.md          # CLI 命令契约
└── tasks.md             # Phase 2 output (by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs                    # 程序入口
├── lib.rs                     # 库根，导出公共 API
│
├── capture/                   # 采集层
│   ├── mod.rs                # 采集模块入口
│   ├── interface.rs          # 网卡枚举与选择
│   ├── sniffer.rs            # 抓包引擎（pcap 封装）
│   └── bpf.rs                # BPF 过滤器处理
│
├── protocol/                  # 处理层 - 协议解析
│   ├── mod.rs                # 协议解析模块
│   ├── parser.rs             # 协议解析器 Trait 定义
│   ├── ethernet.rs           # Ethernet 解析
│   ├── ip.rs                 # IPv4/IPv6 解析
│   ├── tcp.rs                # TCP 解析与流重组
│   ├── udp.rs                # UDP 解析
│   └── icmp.rs               # ICMP 解析
│
├── dpi/                       # 处理层 - 深度包检测
│   ├── mod.rs                # DPI 模块入口
│   ├── engine.rs             # DPI 引擎 Trait 定义
│   ├── ndpi.rs               # nDPI FFI 封装（unsafe 隔离）
│   ├── ja3.rs                # TLS JA3/JA3S 指纹提取
│   └── classifier.rs         # 协议分类器
│
├── storage/                   # 存储层
│   ├── mod.rs                # 存储模块入口
│   ├── metadata.rs           # 元数据存储（SQLite）
│   ├── packet_store.rs       # 原始包存储（PCAPNG）
│   ├── query.rs              # 查询接口
│   └── cleanup.rs            # 数据清理（30 天保留）
│
├── config/                    # 配置管理
│   ├── mod.rs                # 配置模块
│   ├── settings.rs           # 配置结构定义
│   └── loader.rs             # 分层配置加载
│
├── logging/                   # 日志系统
│   ├── mod.rs                # 日志模块
│   └── setup.rs              # 日志初始化与轮转
│
├── cli/                       # 交互层 - CLI
│   ├── mod.rs                # CLI 模块
│   ├── args.rs               # 命令行参数定义（clap）
│   └── commands.rs           # 命令处理逻辑
│
└── common/                    # 公共模块
    ├── mod.rs
    ├── error.rs              # 统一错误类型
    ├── types.rs              # 公共类型定义（五元组等）
    └── utils.rs              # 工具函数

tests/
├── unit/                     # 单元测试
│   ├── capture_tests.rs
│   ├── protocol_tests.rs
│   ├── dpi_tests.rs
│   └── storage_tests.rs
│
├── integration/              # 集成测试
│   ├── capture_integration.rs
│   ├── pipeline_integration.rs
│   └── storage_integration.rs
│
└── fixtures/                 # 测试数据
    ├── sample.pcap
    └── test_config.toml
```

**Structure Decision**: 采用分层架构，各层职责清晰：
- **采集层**：负责网卡枚举、抓包、BPF 过滤
- **处理层**：协议解析 + DPI 分类，通过 Trait 抽象支持扩展
- **存储层**：元数据（SQLite）+ 原始包（PCAPNG）双存储
- **交互层**：CLI 命令解析与执行

unsafe 代码严格隔离在 `dpi/ndpi.rs`，通过安全 Rust API 对外暴露。

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

无违规项，此节留空。
