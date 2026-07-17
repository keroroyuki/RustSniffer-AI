# Research: Rust 智能抓包与流量分析工具

**Branch**: `001-ai-traffic-analyzer` | **Date**: 2026-07-16

## Research Tasks

### Task 1: pcap crate 最佳实践

**Decision**: 使用 `pcap` crate 作为抓包基础，配合 `tokio` 异步运行时

**Rationale**: 
- `pcap` 是 libpcap 的官方 Rust 绑定，跨平台支持好（Linux/Windows）
- 成熟稳定，社区维护活跃
- 支持 BPF 过滤器、混杂模式等核心功能
- 与 `etherparse` 配合良好，可直接传递原始字节

**Alternatives considered**:
- `libbpf-rs` (eBPF): 性能更高，但仅限 Linux 4.15+，原型阶段不采用
- `pnet`: 纯 Rust 实现，但功能不如 pcap 完整，DPI 支持弱

**Implementation notes**:
- 使用 `pcap::Capture::from_device()` 打开网卡
- 设置 `promisc()` 启用混杂模式
- 使用 `filter()` 应用 BPF 表达式
- 通过 `next_packet()` 循环捕获，配合 `tokio::task::spawn_blocking()` 避免阻塞异步运行时

---

### Task 2: etherparse 协议解析策略

**Decision**: 使用 `etherparse` 作为主协议解析库，采用分层解析策略

**Rationale**:
- 纯 Rust 实现，无外部 C 依赖，内存安全
- 支持 Ethernet、IPv4/IPv6、TCP/UDP、ICMP 等基础协议
- 性能优异，解析速度快
- API 设计符合 Rust 惯例，返回 `Result` 类型便于错误处理

**Alternatives considered**:
- `pnet`: 功能较全但 API 设计较老，文档不完善
- 自行实现：开发成本高，易出错

**Implementation notes**:
- 定义 `ProtocolParser` trait，支持扩展自定义协议
- 解析顺序：Ethernet → IP → TCP/UDP → 应用层
- 对于无法识别的协议，保留原始字节供 DPI 分析
- TCP 流重组需自行实现（基于序列号和确认号）

---

### Task 3: nDPI FFI 封装策略

**Decision**: 通过 `bindgen` 生成 nDPI FFI 绑定，在 `dpi/ndpi.rs` 中封装为安全 Rust API

**Rationale**:
- nDPI 是最强的开源 DPI 库，支持 300+ 协议识别
- 基于 Payload 特征识别，不依赖端口号
- C 库性能优异，适合高吞吐场景

**Alternatives considered**:
- 纯 Rust DPI 实现：生态不成熟，协议覆盖率低
- 其他 C 库（OpenDPI）：维护不活跃

**Implementation notes**:
- 使用 `bindgen` 生成 FFI 绑定，放入 `dpi/ndpi_sys.rs`
- 在 `dpi/ndpi.rs` 中封装 `NdpiDetector` 结构体
- 所有 `unsafe` 代码限制在 `dpi/ndpi.rs`，对外暴露安全 API
- 使用 `Drop` trait 确保 nDPI 资源正确释放，防止内存泄漏
- 检测流程：创建检测器 → 逐包输入 → 获取协议分类结果

**Safety measures**:
- 所有指针操作在 `unsafe` 块内，添加详细注释说明安全性保证
- 使用 `cargo miri` 进行内存检查
- 编写 FFI 边界测试，验证资源释放

---

### Task 4: SQLite 元数据存储优化

**Decision**: 使用 `rusqlite` + SQLite，启用 WAL 模式和批量插入

**Rationale**:
- SQLite 轻量级，无需额外服务，适合单节点部署
- `rusqlite` 是成熟的 Rust SQLite 绑定
- WAL 模式提升并发写入性能
- 批量插入减少事务开销

**Alternatives considered**:
- ClickHouse: 性能更强，但部署复杂，原型阶段不采用
- PostgreSQL: 功能更全，但需额外服务

**Implementation notes**:
- 数据库路径：`~/.rustsniffer/metadata.db`
- 启用 WAL 模式：`PRAGMA journal_mode=WAL`
- 批量插入：每 1000 条或每 100ms 提交一次事务
- 索引策略：在 `timestamp`、`src_ip`、`dst_ip`、`protocol` 字段建索引
- 查询优化：使用参数化查询，避免 SQL 注入
- 数据清理：每日凌晨执行，删除 30 天前的记录

**Schema design**:
```sql
CREATE TABLE packets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,  -- Unix timestamp (毫秒)
    src_ip TEXT NOT NULL,
    dst_ip TEXT NOT NULL,
    src_port INTEGER,
    dst_port INTEGER,
    protocol TEXT NOT NULL,
    app_protocol TEXT,           -- DPI 识别的应用层协议
    packet_size INTEGER NOT NULL,
    direction TEXT NOT NULL,     -- "inbound" 或 "outbound"
    metadata TEXT,               -- JSON 格式的应用层字段
    pcap_file TEXT,              -- 原始包文件路径
    pcap_offset INTEGER          -- 在文件中的偏移
);

CREATE INDEX idx_timestamp ON packets(timestamp);
CREATE INDEX idx_src_ip ON packets(src_ip);
CREATE INDEX idx_dst_ip ON packets(dst_ip);
CREATE INDEX idx_protocol ON packets(protocol);
CREATE INDEX idx_app_protocol ON packets(app_protocol);
```

---

### Task 5: PCAPNG 文件存储策略

**Decision**: 使用 `pcap-file` crate 写入 PCAPNG 格式，按天切片存储

**Rationale**:
- PCAPNG 是现代抓包文件格式，支持更多元数据
- 按天切片便于管理和清理
- 支持通过偏移量快速定位原始包

**Alternatives considered**:
- PCAP 格式：较老，不支持某些元数据
- 自定义格式：兼容性问题

**Implementation notes**:
- 存储路径：`~/.rustsniffer/pcap/YYYY-MM-DD.pcapng`
- 使用 `pcap-file` crate 写入
- 每个数据包记录 `(pcap_file, pcap_offset)` 到元数据库
- 支持通过元数据 ID 回溯原始包
- 文件轮转：每日 00:00 创建新文件
- 清理策略：删除 30 天前的文件

---

### Task 6: 配置管理实现

**Decision**: 使用 `toml` + `config` crate，实现分层配置加载

**Rationale**:
- TOML 格式易读易写，适合配置文件
- `config` crate 支持多来源配置合并
- 符合 Rust 生态惯例

**Alternatives considered**:
- YAML: 语法较复杂
- JSON: 不支持注释

**Implementation notes**:
- 配置文件路径：`~/.rustsniffer/config.toml`
- 优先级：命令行参数 > 配置文件 > 默认值
- 配置结构：
  ```rust
  struct Settings {
      capture: CaptureConfig,
      storage: StorageConfig,
      logging: LoggingConfig,
  }
  ```
- 使用 `serde` 反序列化配置
- 支持环境变量覆盖（前缀 `RUSTSNIFFER_`）

---

### Task 7: 日志系统实现

**Decision**: 使用 `tracing` + `tracing-subscriber` + `tracing-appender`

**Rationale**:
- `tracing` 是 Rust 异步生态标准日志框架
- 支持结构化 JSON 输出
- `tracing-appender` 支持文件轮转
- 与 `tokio` 集成良好

**Alternatives considered**:
- `log` + `env_logger`: 功能较简单
- `slog`: 学习曲线较陡

**Implementation notes**:
- 日志路径：`~/.rustsniffer/logs/`
- 轮转策略：100MB/文件，保留 5 个
- 格式：JSON（生产），人类可读（开发）
- 级别：ERROR、WARN、INFO、DEBUG、TRACE
- 默认级别：INFO（生产），DEBUG（开发）
- 关键事件记录：
  - 抓包启动/停止
  - 数据包丢失
  - DPI 识别结果
  - 数据库写入错误
  - 配置加载

---

### Task 8: TCP 流重组策略

**Decision**: 自行实现 TCP 流重组，基于序列号和确认号

**Rationale**:
- `etherparse` 不提供流重组功能
- 需要重组后的 Payload 供 DPI 分析
- 自行实现可精确控制内存使用

**Alternatives considered**:
- 第三方库：Rust 生态中无成熟方案

**Implementation notes**:
- 维护 `TcpStream` 结构体，按五元组标识
- 使用 `BTreeMap` 存储乱序包（按序列号排序）
- 重组逻辑：
  1. 收到包后检查序列号是否连续
  2. 若连续，直接追加到 Payload
  3. 若乱序，存入缓冲区等待缺失包
  4. 超时（如 60 秒）后标记重组失败
- 内存管理：限制单个流最大缓冲区大小（如 10MB）
- 元数据标记：记录重组状态（成功/失败/缺失包数）

---

### Task 9: 异步架构设计

**Decision**: 使用 `tokio` 异步运行时，采用生产者-消费者模式

**Rationale**:
- `tokio` 是 Rust 异步生态事实标准
- 生产者-消费者模式解耦各层，提升吞吐
- 支持高并发网络 I/O

**Alternatives considered**:
- 同步架构：吞吐量低，易阻塞
- 其他异步运行时（async-std）：生态不如 tokio

**Implementation notes**:
- 架构：
  ```
  采集层 (生产者) → 通道 → 处理层 (消费者) → 通道 → 存储层 (消费者)
  ```
- 通道：`tokio::sync::mpsc`，容量 10000
- 采集任务：`spawn_blocking()` 运行 pcap 捕获循环
- 处理任务：异步解析协议和 DPI 分类
- 存储任务：异步批量写入数据库
- 背压处理：通道满时采集层暂停，防止内存暴涨
- 优雅关闭：使用 `CancellationToken` 通知各层退出

---

### Task 10: 错误处理策略

**Decision**: 使用 `thiserror` 定义统一错误类型，`anyhow` 用于应用层错误传播

**Rationale**:
- `thiserror` 适合库代码，生成清晰的错误类型
- `anyhow` 适合应用代码，简化错误传播
- 符合 Rust 生态惯例

**Alternatives considered**:
- 自定义错误枚举：开发成本高
- 仅用 `Box<dyn Error>`：类型信息丢失

**Implementation notes**:
- 库错误：`src/common/error.rs` 定义 `SnifferError` 枚举
  ```rust
  enum SnifferError {
      CaptureError(String),
      ProtocolError(String),
      DpiError(String),
      StorageError(String),
      ConfigError(String),
  }
  ```
- 应用错误：`main.rs` 使用 `anyhow::Result`
- 错误信息：包含上下文，提供解决建议
- 日志记录：所有错误 MUST 记录到日志

---

## Summary

所有研究任务已完成，技术选型明确：
- **抓包**: `pcap` + `tokio::spawn_blocking()`
- **协议解析**: `etherparse` + 自定义 TCP 流重组
- **DPI**: nDPI FFI 绑定，严格封装 unsafe
- **存储**: SQLite (WAL 模式) + PCAPNG 按天切片
- **配置**: TOML 格式，分层加载
- **日志**: `tracing` + JSON 输出 + 文件轮转
- **架构**: tokio 异步，生产者-消费者模式
- **错误处理**: `thiserror` + `anyhow`

所有决策符合项目宪法原则，无违规项。
