# Tasks: Rust 智能抓包与流量分析工具

**Branch**: `001-ai-traffic-analyzer` | **Date**: 2026-07-16
**Spec**: [spec.md](file:///d:/ai-coding/RustSniffer-AI/specs/001-ai-traffic-analyzer/spec.md)
**Plan**: [plan.md](file:///d:/ai-coding/RustSniffer-AI/specs/001-ai-traffic-analyzer/plan.md)

## Summary

- **Total Tasks**: 80
- **MVP Scope**: User Story 1 (P1) + User Story 2 (P2) + User Story 3 (P2)
- **User Stories**: 6 total (US1-US3 in MVP, US4-US6 deferred to v1.1+)
- **Parallel Opportunities**: Marked with [P]

---

## Phase 1: Setup (项目初始化)

**Goal**: 初始化 Rust 项目结构，配置依赖，确保 `cargo build` 通过

- [ ] T001 使用 `cargo init --name rustsniffer` 初始化项目，创建 `src/main.rs` 和 `Cargo.toml`
- [ ] T002 在 `Cargo.toml` 中添加核心依赖：`pcap`, `etherparse`, `tokio` (full), `rusqlite` (bundled), `clap` (derive), `serde` (derive), `serde_json`, `toml`, `thiserror`, `anyhow`, `tracing`, `tracing-subscriber`, `tracing-appender`, `chrono`, `dirs`
- [ ] T003 创建项目目录结构：`src/capture/`, `src/protocol/`, `src/dpi/`, `src/storage/`, `src/config/`, `src/logging/`, `src/cli/`, `src/common/`
- [ ] T004 创建测试目录结构：`tests/unit/`, `tests/integration/`, `tests/fixtures/`
- [ ] T005 [P] 创建所有模块的 `mod.rs` 入口文件，在 `src/lib.rs` 中声明各顶层模块
- [ ] T006 验证 `cargo build` 和 `cargo clippy` 通过无错误

---

## Phase 2: Foundational (基础模块)

**Goal**: 实现公共类型、错误处理、配置加载、日志系统——所有 User Story 的前置依赖

- [ ] T007 在 `src/common/types.rs` 中定义 `FiveTuple` 结构体（src_ip, dst_ip, src_port, dst_port, protocol）及 `Protocol` 枚举（TCP/UDP/ICMP）
- [ ] T008 [P] 在 `src/common/types.rs` 中定义 `Direction` 枚举（Inbound/Outbound）和 `PacketInfo` 结构体（timestamp, raw_bytes, interface_id）
- [ ] T009 在 `src/common/error.rs` 中使用 `thiserror` 定义 `SnifferError` 枚举，包含 CaptureError、ProtocolError、DpiError、StorageError、ConfigError 变体
- [ ] T010 [P] 在 `src/common/utils.rs` 中实现工具函数：获取数据目录路径 `~/.rustsniffer/`、确保目录存在
- [ ] T011 在 `src/config/settings.rs` 中定义 `Settings`、`CaptureConfig`、`StorageConfig`、`LoggingConfig` 结构体，使用 `serde` derive
- [ ] T012 在 `src/config/loader.rs` 中实现分层配置加载：默认值 → 配置文件 `~/.rustsniffer/config.toml` → 环境变量（前缀 `RUSTSNIFFER_`，如 `RUSTSNIFFER_LOG_LEVEL=debug`）→ 命令行参数（最高优先级）
- [ ] T013 在 `src/logging/setup.rs` 中实现日志初始化：使用 `tracing-subscriber` + `tracing-appender`，JSON 格式输出到 `~/.rustsniffer/logs/`，100MB 轮转，保留 5 个文件
- [ ] T014 在 `src/common/mod.rs` 中导出所有公共类型，确保 `src/lib.rs` 正确导出 `common` 模块
- [ ] T015 验证 `cargo test` 通过，基础模块可编译

---

## Phase 3: User Story 1 - 基础网络抓包与实时查看 (P1)

**Story Goal**: 用户启动工具后选择网卡，系统开始抓包并在终端实时显示数据包五元组信息

**Independent Test**: 启动工具 → 选择网卡 → 观察终端实时输出五元组信息 → Ctrl+C 优雅退出显示统计摘要

**Dependencies**: Phase 2 (Foundational)

### CLI 参数定义

- [ ] T016 [US1] 在 `src/cli/args.rs` 中使用 `clap` derive 定义 CLI 参数结构：`Cli` 主结构含 `interface`/`filter`/`promiscuous`/`count`/`duration` 参数，`list_interfaces` 子命令
- [ ] T017 [P] [US1] 在 `src/cli/commands.rs` 中实现 `list_interfaces` 命令处理：调用 `pcap::Device::list()` 枚举网卡并格式化输出

### 网卡枚举与选择

- [ ] T018 [US1] 在 `src/capture/interface.rs` 中实现 `list_interfaces()` 函数：返回可用网卡列表（名称、描述、地址）
- [ ] T019 [P] [US1] 在 `src/capture/interface.rs` 中实现 `select_interface(name: Option<&str>)` 函数：自动选择第一个或按名称匹配

### BPF 过滤器

- [ ] T020 [US1] 在 `src/capture/bpf.rs` 中实现 `validate_bpf(expression: &str) -> Result<()>` 函数：验证 BPF 语法正确性
- [ ] T021 [P] [US1] 在 `src/capture/bpf.rs` 中实现 `apply_bpf(capture: &mut pcap::Capture, expression: &str) -> Result<()>` 函数

### 抓包引擎

- [ ] T022 [US1] 在 `src/capture/sniffer.rs` 中定义 `Sniffer` 结构体，包含 `start(interface, bpf_filter, promiscuous) -> Result<CaptureStream>` 方法
- [ ] T023 [US1] 在 `src/capture/sniffer.rs` 中实现 `CaptureStream`：封装 `pcap::Capture`，使用 `tokio::task::spawn_blocking()` 异步捕获数据包
- [ ] T024 [US1] 在 `src/capture/sniffer.rs` 中实现 `CaptureStats` 统计结构：total_packets, dropped_packets, duration, packets_per_second
- [ ] T025 [P] [US1] 在 `src/capture/sniffer.rs` 中实现信号处理：捕获 SIGINT/SIGTERM，优雅停止并输出统计摘要
- [ ] T025a [US1] 在 `src/capture/sniffer.rs` 中实现 SIGHUP 信号处理：捕获 SIGHUP 后调用 `config::reload()` 重新加载配置，应用新配置到正在运行的抓包会话（如日志级别、BPF 过滤器）
- [ ] T025b [US1] 在 `src/config/loader.rs` 中实现 `reload() -> Result<Settings>` 函数：重新读取配置文件并合并环境变量，返回新的 Settings
- [ ] T025c [US1] 在 `src/capture/sniffer.rs` 中实现流量采样：当 packets_per_second > 100,000 时，启用采样模式（每 N 个包保留 1 个），在终端输出提示 "已启用采样，实际流量为显示值的 N 倍"
- [ ] T025d [US1] 在 `src/capture/sniffer.rs` 中实现采样率动态调整：根据实时流量自动调整采样间隔，确保显示延迟 < 200ms

### 实时输出

- [ ] T026 [US1] 在 `src/cli/commands.rs` 中实现 `run_capture()` 函数：整合 Sniffer + 终端输出，按 tcpdump 格式打印五元组（时间戳、源IP:端口 → 目的IP:端口、协议、包长）
- [ ] T027 [P] [US1] 在 `src/capture/mod.rs` 中导出所有公共 API

### 集成验证

- [ ] T028 [US1] 在 `src/main.rs` 中整合 CLI 解析 + capture 命令，确保 `rustsniffer capture` 可运行
- [ ] T029 [US1] 在 `src/main.rs` 中整合 `list-interfaces` 子命令
- [ ] T030 [US1] 验证端到端流程：`cargo run -- capture` 能实时输出数据包信息，Ctrl+C 优雅退出

---

## Phase 4: User Story 2 - 深度协议解析与 DPI 分类 (P2)

**Story Goal**: 系统解析数据包，识别 HTTP/DNS/TLS/SSH 等应用层协议，提取关键元数据和 JA3 指纹

**Independent Test**: 捕获 HTTP 流量 → 验证识别为 HTTP 并提取 Host/URI/Method → 捕获 DNS 流量 → 验证提取查询域名

**Dependencies**: Phase 3 (US1 - 需要抓包引擎)

### 协议解析框架

- [ ] T031 [US2] 在 `src/protocol/parser.rs` 中定义 `ProtocolParser` trait：`fn parse(&self, raw: &[u8]) -> Result<ParsedPacket>`，`ParsedPacket` 包含五元组 + 协议元数据
- [ ] T032 [P] [US2] 在 `src/protocol/parser.rs` 中定义 `ParsedPacket` 和 `ProtocolMetadata` 结构体（ethernet_type, ip_version, ip_ttl, transport_protocol, app_protocol, app_metadata）
- [ ] T033 [US2] 在 `src/protocol/ethernet.rs` 中实现 Ethernet 帧解析：使用 `etherparse` 提取 Ethernet 类型
- [ ] T034 [P] [US2] 在 `src/protocol/ip.rs` 中实现 IPv4/IPv6 解析：提取 IP 版本、TTL、源/目的 IP
- [ ] T035 [US2] 在 `src/protocol/tcp.rs` 中实现 TCP 解析：提取源/目的端口、序列号、确认号、标志位
- [ ] T036 [P] [US2] 在 `src/protocol/udp.rs` 中实现 UDP 解析：提取源/目的端口
- [ ] T037 [US2] 在 `src/protocol/icmp.rs` 中实现 ICMP 解析：提取类型和代码

### TCP 流重组

- [ ] T038 [US2] 在 `src/protocol/tcp.rs` 中实现 `TcpStream` 结构体：按五元组标识，使用 `BTreeMap` 存储乱序包
- [ ] T039 [US2] 在 `src/protocol/tcp.rs` 中实现 `TcpReassembler`：基于序列号重组 Payload，处理乱序和重传
- [ ] T040 [P] [US2] 在 `src/protocol/tcp.rs` 中实现超时机制（60 秒）和缓冲区限制（10MB），标记重组状态（Completed/Failed）

### DPI 引擎

- [ ] T041 [US2] 在 `src/dpi/engine.rs` 中定义 `DpiEngine` trait：`fn detect(&mut self, packet: &[u8], flow: &FiveTuple) -> DpiResult`
- [ ] T042 [US2] 在 `build.rs` 中配置 `bindgen` 生成 nDPI FFI 绑定到 `src/dpi/ndpi_sys.rs`（自动排除版本控制），在 `src/dpi/ndpi.rs` 中实现安全封装
- [ ] T043 [US2] 在 `src/dpi/ndpi.rs` 中实现 `NdpiDetector` 结构体：封装 nDPI C 库，所有 `unsafe` 代码隔离在此文件
- [ ] T044 [US2] 在 `src/dpi/ndpi.rs` 中实现 `Drop` trait 确保 nDPI 资源正确释放，防止内存泄漏
- [ ] T045 [P] [US2] 在 `src/dpi/classifier.rs` 中实现 `ProtocolClassifier`：调用 DpiEngine 获取协议分类结果，未知协议标记为 "Unknown"

### JA3 指纹提取

- [ ] T046 [US2] 在 `src/dpi/ja3.rs` 中实现 TLS Client Hello 解析：提取 TLS 版本、加密套件列表
- [ ] T047 [P] [US2] 在 `src/dpi/ja3.rs` 中实现 JA3/JA3S 指纹计算：MD5 哈希生成标准 JA3 字符串

### 协议解析整合

- [ ] T048 [US2] 在 `src/protocol/mod.rs` 中实现 `PacketProcessor`：串联协议解析 → DPI 分类 → 元数据提取的完整流水线
- [ ] T049 [US2] 在 `src/cli/commands.rs` 中更新 `run_capture()`：集成协议解析和 DPI 分类，终端输出增加应用层协议和元数据字段
- [ ] T050 [US2] 验证端到端流程：捕获 HTTP 流量 → 识别协议 → 提取 Host/URI/Method

---

## Phase 5: User Story 3 - 元数据存储与快速检索 (P2)

**Story Goal**: 将解析后的元数据存入 SQLite，支持快速查询，原始包按天切片写入 PCAPNG

**Independent Test**: 抓包一段时间 → 查询 "过去 5 分钟 DNS 查询" → 1 秒内返回结果 → 通过元数据索引回溯原始包

**Dependencies**: Phase 4 (US2 - 需要解析后的元数据)

### 元数据存储

- [ ] T051 [US3] 在 `src/storage/metadata.rs` 中实现 `MetadataStore` 结构体：封装 `rusqlite::Connection`，数据库路径 `~/.rustsniffer/metadata.db`
- [ ] T052 [US3] 在 `src/storage/metadata.rs` 中实现 `init_schema()`：创建 packets 和 sessions 表及索引，启用 WAL 模式
- [ ] T053 [US3] 在 `src/storage/metadata.rs` 中实现 `insert_packet()` 和 `batch_insert_packets()`：批量插入元数据（每 1000 条或 100ms 提交）
- [ ] T054 [P] [US3] 在 `src/storage/metadata.rs` 中实现 `query_packets()`：支持按时间范围、协议类型、IP 地址查询，使用参数化查询

### 原始包存储

- [ ] T055 [US3] 在 `src/storage/packet_store.rs` 中实现 `PacketStore` 结构体：写入 PCAPNG 文件到 `~/.rustsniffer/pcap/`
- [ ] T056 [US3] 在 `src/storage/packet_store.rs` 中实现按天切片：文件名格式 `YYYY-MM-DD.pcapng`，每日 00:00 创建新文件
- [ ] T057 [P] [US3] 在 `src/storage/packet_store.rs` 中实现 `write_packet()` 返回 `(file_path, offset)` 用于元数据关联
- [ ] T057a [US3] 在 `src/storage/packet_store.rs` 中实现 `read_packet(file_path: &str, offset: u64) -> Result<Vec<u8>>` 函数：从 PCAPNG 文件读取指定偏移的原始数据包
- [ ] T057b [US3] 在 `src/storage/metadata.rs` 中实现 `get_raw_packet(packet_id: u64) -> Result<Vec<u8>>` 方法：查询元数据获取 (pcap_file, pcap_offset)，调用 PacketStore 读取原始包
- [ ] T057c [US3] 在 `src/cli/commands.rs` 中添加 `export-packet` 子命令：接受 packet_id 参数，导出原始包到指定文件

### 数据清理

- [ ] T058 [US3] 在 `src/storage/cleanup.rs` 中实现 `cleanup_old_data()`：删除 30 天前的元数据记录和 PCAPNG 文件
- [ ] T058a [US3] 在 `src/storage/metadata.rs` 中实现 `check_integrity() -> Result<bool>` 方法：执行 `PRAGMA integrity_check`，检测数据库损坏
- [ ] T058b [US3] 在 `src/storage/metadata.rs` 中实现 `recover_from_corruption() -> Result<()>` 方法：备份损坏文件为 `metadata.db.corrupted.{timestamp}`，创建新数据库，记录日志 "数据库已重建，历史数据位于 backup.db"
- [ ] T058c [US3] 在 `src/storage/metadata.rs` 的 `init_schema()` 中调用 `check_integrity()`：若检测失败，自动调用 `recover_from_corruption()` 并输出用户提示

### 查询接口

- [ ] T059 [US3] 在 `src/storage/query.rs` 中实现 `QueryEngine`：封装高级查询（Top N IP、协议分布、时间聚合）
- [ ] T060 [US3] 在 `src/cli/commands.rs` 中实现 `query` 子命令：接受 SQL 查询，格式化输出（table/json/csv）

### 存储层整合

- [ ] T061 [US3] 在 `src/cli/commands.rs` 中更新 `run_capture()`：集成异步存储流水线（采集 → 解析 → 存储），使用 `tokio::sync::mpsc` 通道连接各层
- [ ] T062 [US3] 验证端到端流程：抓包 → 元数据写入数据库 → `rustsniffer query` 查询返回结果

---

## Phase 6: Polish & Cross-Cutting Concerns

**Goal**: 完善错误处理、日志记录、代码质量，确保 MVP 达到生产可用标准

- [ ] T063 在所有模块中添加 `///` 文档注释（中文）和 `//!` 模块说明
- [ ] T064 确保所有生产代码不使用 `unwrap()`，改用 `?` 或 `match` 处理错误
- [ ] T065 实现错误信息增强：所有 `SnifferError` 包含问题原因和解决建议
- [ ] T066 在关键路径添加 `tracing::info!`/`tracing::warn!`/`tracing::error!` 日志
- [ ] T067 运行 `cargo clippy --all-targets --all-features -- -D warnings` 确保零警告
- [ ] T068 运行 `cargo fmt --check` 确保代码格式一致
- [ ] T069 编写集成测试 `tests/integration/capture_integration.rs`：验证抓包 → 解析 → 存储完整流水线
- [ ] T070 更新 `Cargo.toml` 版本号、描述、许可证等元数据
- [ ] T070a 在 `tests/integration/memory_leak_test.rs` 中实现内存泄漏测试：运行抓包 1 小时（使用 pcap 文件回放），每 5 分钟采样 RSS 内存，验证增长 < 5%
- [ ] T070b 在 `tests/integration/memory_leak_test.rs` 中使用 `tokio::time::interval` 定期触发 GC（如可能），记录内存使用趋势

---

## Deferred User Stories (v1.1+)

以下 User Story 不在 MVP 范围内，延后至后续版本：

- **US4** - 自然语言查询与 AI 分析 (P3)：需接入 LLM API，实现 Text-to-BPF/SQL
- **US5** - 流量语义总结与异常检测 (P3)：需 AI 辅助分析，异常行为识别
- **US6** - 终端用户界面与实时监控 (P4)：需 `ratatui` 实现 TUI 仪表盘

---

## Dependency Graph

```text
Phase 1 (Setup)
    ↓
Phase 2 (Foundational)
    ↓
Phase 3 (US1: 基础抓包) ← P1, MVP 核心
    ↓
Phase 4 (US2: DPI 分类) ← P2, MVP 核心
    ↓
Phase 5 (US3: 元数据存储) ← P2, MVP 核心
    ↓
Phase 6 (Polish)
```

---

## Parallel Execution Examples

### US1 并行机会
- T017 (list_interfaces 命令) 与 T016 (CLI 参数) 可并行
- T019 (select_interface) 与 T018 (list_interfaces) 可并行
- T021 (apply_bpf) 与 T020 (validate_bpf) 可并行
- T025 (信号处理) 与 T022-T024 (Sniffer) 可并行
- T025a (SIGHUP 处理) 与 T025b (config reload) 可并行
- T025c (流量采样) 与 T025d (采样率调整) 可并行

### US2 并行机会
- T032 (ParsedPacket 定义) 与 T031 (ProtocolParser trait) 可并行
- T034 (IP 解析) 与 T033 (Ethernet 解析) 可并行
- T036 (UDP 解析) 与 T035 (TCP 解析) 可并行
- T045 (ProtocolClassifier) 与 T043 (NdpiDetector) 可并行
- T047 (JA3 计算) 与 T046 (TLS 解析) 可并行

### US3 并行机会
- T054 (query_packets) 与 T053 (batch_insert) 可并行
- T057 (write_packet) 与 T055-T056 (PacketStore) 可并行
- T057a (read_packet) 与 T057b (get_raw_packet) 可并行
- T058a (check_integrity) 与 T058b (recover_from_corruption) 可并行

---

## Implementation Strategy

1. **MVP First**: 仅实现 US1-US3，确保基础抓包 + DPI + 存储功能完整可用
2. **Incremental Delivery**: 每个 Phase 完成后独立可测试
3. **Quality Gates**: 每个 Phase 结束时 `cargo build` + `cargo clippy` + `cargo test` 必须通过
4. **Safety First**: unsafe 代码严格隔离在 `dpi/ndpi.rs`，FFI 边界编写专项测试
