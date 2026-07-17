<!--
Sync Impact Report
==================
Version change: N/A → 1.0.0 (Initial release)
Modified principles: N/A (Initial creation)
Added sections:
  - Core Principles (5 principles)
  - Technical Stack Constraints
  - Development Workflow & Quality Gates
  - Governance
Removed sections: None
Templates requiring updates:
  - .specify/templates/plan-template.md: ✅ No changes needed
  - .specify/templates/spec-template.md: ✅ No changes needed
  - .specify/templates/tasks-template.md: ✅ No changes needed
Follow-up TODOs: None
-->

# RustSniffer-AI Constitution

## Core Principles

### I. 高度可用性 (NON-NEGOTIABLE)
所有代码 MUST 确保编译成功且无功能性缺陷。开发过程中 MUST 严格避免以下问题：
- 编译错误：代码 MUST 通过 `cargo build` 和 `cargo clippy` 无任何错误
- 内存泄露：MUST 使用 Rust 所有权机制管理内存，所有 `unsafe` 代码 MUST 经过严格审查
- 运行时崩溃：MUST 处理所有可能的错误情况，避免 `unwrap()` 在生产代码中使用
- 数据丢失：MUST 确保数据包捕获和存储的完整性

### II. 安全性优先
安全性是本项目的首要考量因素，MUST 遵循以下规则：
- 内存安全：充分利用 Rust 的所有权和借用检查机制，严格限制 `unsafe` 代码块范围
- FFI 安全：调用 nDPI 等 C 库时，MUST 在安全的 Rust API 层封装所有 `unsafe` 操作
- 敏感数据处理：MUST 在发送数据到外部 LLM API 前进行正则脱敏（MAC 地址、内网 IP、敏感 Payload）
- 输入验证：MUST 对所有外部输入（网络数据包、用户输入、配置文件）进行边界检查和有效性验证

### III. 代码可读性与注释规范
所有核心逻辑和复杂实现 MUST 添加清晰、准确的关键注释：
- 函数级注释：公开 API MUST 包含文档注释（`///`），说明功能、参数、返回值和可能的错误
- 复杂逻辑注释：算法实现、协议解析、DPI 分类等核心逻辑 MUST 添加行内注释解释设计意图
- 模块级注释：每个模块文件 MUST 在顶部包含模块功能说明（`//!`）
- 注释语言：代码注释 MUST 使用中文，保持与项目文档一致性

### IV. 易用性与用户体验
MUST 将易用性作为设计核心，遵循以下原则：
- API 设计：公开接口 MUST 简洁直观，遵循 Rust 命名约定和生态惯例
- 错误信息：所有错误信息 MUST 清晰描述问题原因和解决建议，避免技术性术语堆砌
- CLI 交互：命令行参数 MUST 提供清晰的帮助信息和合理的默认值
- TUI 界面：终端界面 MUST 响应迅速，避免阻塞操作，AI 结果 MUST 异步展示

### V. 可维护性与可扩展性
代码架构 MUST 支持长期维护和功能扩展：
- 模块化设计：MUST 采用分层架构（采集层、处理层、存储层、AI 层、交互层），各层职责清晰
- Trait 抽象：协议解析器、DPI 引擎、存储后端 MUST 定义为 Trait 接口，便于替换和扩展
- 依赖管理：MUST 优先选择纯 Rust 实现的 crate，减少外部 C 依赖；必须使用 C 库时，MUST 提供安全的 Rust 封装
- 测试覆盖：核心功能 MUST 编写单元测试，关键路径 MUST 编写集成测试

## Technical Stack Constraints

本项目 MUST 遵循以下技术栈约束：
- **语言版本**：Rust stable 工具链（最新稳定版）
- **异步运行时**：`tokio` 作为异步运行时，MUST 使用 `async/await` 语法
- **网络抓包**：`pcap` crate（原型阶段），后续可引入 `libbpf-rs`（eBPF）
- **协议解析**：`etherparse` 作为主要解析库，避免使用 `pnet`（除非必要）
- **DPI 引擎**：`nDPI` 通过 FFI 绑定，MUST 封装为安全的 Rust API
- **数据存储**：`rusqlite`（SQLite）用于元数据存储，支持批量插入和 WAL 模式
- **AI 接入**：`reqwest` 调用云端 API，`ollama-rs` 支持本地模型，MUST 默认使用本地模型保护隐私
- **CLI/TUI**：`clap` 处理命令行参数，`ratatui` 构建终端界面
- **序列化**：`serde` + `serde_json` 用于 JSON 序列化，`bincode` 用于二进制序列化

## Development Workflow & Quality Gates

所有代码变更 MUST 通过以下质量门禁：
- **编译检查**：`cargo build --all-targets` MUST 无错误
- **代码检查**：`cargo clippy --all-targets --all-features -- -D warnings` MUST 无警告
- **格式化**：`cargo fmt --check` MUST 通过，代码格式 MUST 符合 rustfmt 规范
- **测试执行**：`cargo test` MUST 全部通过，核心功能测试覆盖率 MUST ≥ 80%
- **内存检查**：涉及 `unsafe` 代码的变更 MUST 通过 `cargo miri` 或 `valgrind` 检查
- **代码审查**：所有 PR MUST 经过至少一人审查，审查重点包括安全性、可读性、性能
- **文档更新**：公开 API 变更 MUST 同步更新文档注释，重大变更 MUST 更新 README

## Governance

本宪法是所有开发实践的最高准则，MUST 遵循以下治理规则：
- **优先级**：本宪法优先于所有其他开发实践和约定，任何代码变更 MUST 符合宪法原则
- **修订流程**：宪法修订 MUST 经过团队讨论、文档记录、影响评估，并更新版本号
- **版本管理**：版本号遵循语义化版本（MAJOR.MINOR.PATCH），MAJOR 版本变更表示不兼容的原则修改
- **合规审查**：所有代码审查 MUST 验证是否符合宪法原则，违反原则的代码 MUST 拒绝合并
- **例外处理**：如需偏离宪法原则，MUST 在 PR 中明确说明理由并获得团队一致同意

**Version**: 1.0.0 | **Ratified**: 2026-07-16 | **Last Amended**: 2026-07-16
