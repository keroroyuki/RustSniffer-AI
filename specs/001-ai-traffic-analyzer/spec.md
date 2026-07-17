# Feature Specification: Rust 智能抓包与流量分析工具

**Feature Branch**: `001-ai-traffic-analyzer`  
**Created**: 2026-07-16  
**Status**: Draft  
**Input**: User description: "Rust 智能抓包与流量分析工具 - 基于PRD文档的完整功能需求"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 基础网络抓包与实时查看 (Priority: P1)

网络工程师需要快速捕获网络流量并实时查看基础信息，以便排查网络故障。用户启动工具后选择网卡，系统开始抓包并在终端实时显示数据包的五元组信息（源/目的 IP、端口、协议），类似 tcpdump 的输出格式。

**Why this priority**: 这是工具的核心基础功能，没有抓包能力就无法进行后续分析。所有其他功能都依赖于此。

**Independent Test**: 可以通过启动工具、选择网卡、观察终端输出来独立测试，能够实时显示捕获的数据包基础信息。

**Acceptance Scenarios**:

1. **Given** 系统存在可用网卡，**When** 用户启动工具并选择网卡，**Then** 系统开始捕获数据包并在终端实时显示五元组信息
2. **Given** 正在抓包，**When** 用户指定 BPF 过滤表达式（如 `port 80`），**Then** 系统仅显示符合过滤条件的数据包
3. **Given** 正在抓包，**When** 用户按 Ctrl+C，**Then** 系统优雅停止抓包并显示统计摘要（总包数、丢包率、捕获时长）

---

### User Story 2 - 深度协议解析与 DPI 分类 (Priority: P2)

安全研究员需要识别网络流量的具体应用层协议，不仅基于端口号，还要基于 Payload 特征进行深度检测。系统解析捕获的数据包，识别 HTTP、DNS、TLS、SSH 等应用层协议，并提取关键元数据（如 HTTP Host、DNS 查询域名、TLS JA3 指纹）。

**Why this priority**: 深度包检测是区别于传统工具的核心能力，为后续 AI 分析提供结构化数据基础。

**Independent Test**: 可以通过捕获包含已知协议的流量（如 HTTP 请求），验证系统能否正确识别协议类型并提取元数据。

**Acceptance Scenarios**:

1. **Given** 捕获到 HTTP 流量，**When** 系统解析数据包，**Then** 识别为 HTTP 协议并提取 Host、URI、Method 等字段
2. **Given** 捕获到 DNS 流量，**When** 系统解析数据包，**Then** 识别为 DNS 协议并提取查询域名、查询类型
3. **Given** 捕获到 TLS 流量，**When** 系统完成握手解析，**Then** 提取 JA3/JA3S 指纹并记录 TLS 版本、加密套件
4. **Given** 捕获到未知协议流量，**When** DPI 引擎无法识别，**Then** 标记为 "Unknown" 并记录原始 Payload 前 64 字节用于后续分析

---

### User Story 3 - 元数据存储与快速检索 (Priority: P2)

用户需要保存解析后的流量元数据以便后续分析和历史查询。系统将结构化元数据（五元组、协议类型、时间戳、应用层字段）存入本地数据库，支持按时间范围、协议类型、IP 地址等条件快速查询。

**Why this priority**: 数据持久化是进行历史分析和 AI 分析的前提，与 DPI 分类同等重要。

**Independent Test**: 可以通过捕获一段时间流量后，使用查询条件检索特定记录来验证存储和检索功能。

**Acceptance Scenarios**:

1. **Given** 系统正在抓包，**When** 数据包被解析，**Then** 元数据在 100ms 内写入数据库
2. **Given** 数据库已有记录，**When** 用户查询 "过去 5 分钟的 DNS 查询"，**Then** 系统在 1 秒内返回符合条件的记录列表
3. **Given** 数据库已有记录，**When** 用户查询 "访问次数最多的前 10 个 IP"，**Then** 系统返回聚合统计结果
4. **Given** 系统运行 24 小时，**When** 检查数据库完整性，**Then** 无数据丢失或损坏

---

### User Story 4 - 自然语言查询与 AI 分析 (Priority: P3)

用户希望通过自然语言与工具交互，降低使用门槛。用户输入 "找出过去 5 分钟内访问次数最多的前 10 个外部 IP"，AI 将其转换为 BPF 过滤表达式或 SQL 查询，自动执行并返回结果。

**Why this priority**: AI 交互是产品的核心亮点，但依赖前三个功能作为基础，因此优先级为 P3。

**Independent Test**: 可以通过输入自然语言查询，验证系统能否正确解析意图、生成查询、执行并返回结果。

**Acceptance Scenarios**:

1. **Given** 数据库有流量记录，**When** 用户输入 "找出访问 github.com 的所有请求"，**Then** AI 生成查询并返回匹配的记录列表
2. **Given** 数据库有流量记录，**When** 用户输入 "统计各协议的流量占比"，**Then** AI 生成聚合查询并返回饼图数据
3. **Given** AI 生成查询，**When** 查询执行前，**Then** 系统显示生成的 BPF/SQL 供用户确认（可选）
4. **Given** AI 无法理解用户输入，**When** 查询意图不明确，**Then** 系统返回友好的错误提示并给出示例查询

---

### User Story 5 - 流量语义总结与异常检测 (Priority: P3)

安全研究员需要对特定会话或时间段的流量进行语义总结，并检测异常行为。用户对某个 IP 的流量执行总结，系统分析其协议分布、访问模式，给出 "该 IP 主要在进行 DNS 查询和 HTTPS 访问，未发现明显的横向移动特征" 的结论。

**Why this priority**: 这是 AI 辅助分析的高级功能，依赖元数据存储和基础分析能力。

**Independent Test**: 可以通过对已知流量模式（如端口扫描、正常浏览）执行总结，验证 AI 能否给出准确描述。

**Acceptance Scenarios**:

1. **Given** 数据库有某 IP 的流量记录，**When** 用户请求 "总结该 IP 的流量特征"，**Then** 系统返回协议分布、Top 访问域名、时间分布等摘要
2. **Given** 存在端口扫描流量，**When** AI 分析该流量，**Then** 识别为 "端口扫描行为" 并标记为可疑
3. **Given** AI 给出结论，**When** 结论基于数据分析，**Then** 必须附带原始数据证据（如 "根据以下 3 个包的 Payload 判断..."）
4. **Given** AI 检测到异常，**When** 异常置信度低于阈值，**Then** 标记为 "待确认" 而非直接告警

---

### User Story 6 - 终端用户界面与实时监控 (Priority: P4)

用户希望通过友好的终端界面实时监控流量统计，而不是仅依赖命令行输出。系统提供类似 htop 的 TUI 界面，显示实时流量统计、协议分布、Top IP 列表，并支持在界面内输入 AI 查询。

**Why this priority**: TUI 提升用户体验，但不影响核心功能，可在基础功能完成后实现。

**Independent Test**: 可以通过启动 TUI 模式，观察界面是否实时更新流量统计信息。

**Acceptance Scenarios**:

1. **Given** 系统正在抓包，**When** 用户启动 TUI 模式，**Then** 界面显示实时流量统计（包/秒、字节/秒、活跃连接数）
2. **Given** TUI 界面运行中，**When** 新数据包到达，**Then** 界面在 200ms 内更新统计信息
3. **Given** TUI 界面运行中，**When** 用户输入 AI 查询，**Then** 查询结果在独立面板显示，不阻塞主界面
4. **Given** TUI 界面运行中，**When** 用户按 q 键，**Then** 系统优雅退出并保存当前状态

---

### Edge Cases

- 当网卡不可用或权限不足时，系统显示明确的错误信息和解决建议（如 "请以管理员权限运行"）
- 当网络流量极大（超过 10 万包/秒）时，系统启用采样模式并在界面提示 "已启用采样，实际流量为显示值的 10 倍"
- 当数据库文件损坏时，系统自动备份损坏文件并创建新数据库，提示用户 "数据库已重建，历史数据位于 backup.db"
- 当 AI 服务不可用（网络问题或本地模型未启动）时，系统降级为仅显示原始查询结果，提示 "AI 分析暂不可用，显示原始数据"
- 当检测到敏感数据（内网 IP、MAC 地址）时，系统在发送到外部 AI 前自动脱敏，并在日志记录 "已脱敏 X 条敏感信息"
- 当 TCP 流重组遇到乱序或重传时，系统正确重组并在元数据标记 "重组完成" 或 "重组失败（缺失 X 个包）"

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 系统 MUST 自动枚举系统可用网卡并允许用户选择
- **FR-002**: 系统 MUST 支持标准 BPF 语法进行底层包过滤
- **FR-003**: 系统 MUST 支持混杂模式和常规模式抓包
- **FR-004**: 系统 MUST 解析 Ethernet、IPv4/IPv6、TCP/UDP、ICMP 等基础协议
- **FR-005**: 系统 MUST 实现 TCP 会话跟踪与流重组（处理乱序、重传）
- **FR-006**: 系统 MUST 基于 Payload 特征识别应用层协议（HTTP、DNS、TLS、SSH 等），而非仅依赖端口号
- **FR-007**: 系统 MUST 提取 TLS 握手的 JA3/JA3S 指纹
- **FR-008**: 系统 MUST 提取五元组、包长、时间戳、方向、应用层关键字段（HTTP Host、DNS Query 等）
- **FR-009**: 系统 MUST 将解析后的结构化元数据存入本地数据库（`~/.rustsniffer/metadata.db`），自动清理 30 天前的数据
- **FR-010**: 系统 MUST 支持按时间范围、协议类型、IP 地址等条件快速查询元数据
- **FR-011**: 系统 MUST 将原始 Pcap 数据按天切片追加写入 `~/.rustsniffer/pcap/` 目录（PCAPNG 格式），保留 30 天
- **FR-012**: 系统 MUST 支持通过元数据索引回溯原始包
- **FR-013**: 系统 MUST 支持自然语言输入并转换为 BPF 过滤表达式或 SQL 查询
- **FR-014**: 系统 MUST 对特定会话或时间段的流量元数据进行语义总结
- **FR-015**: 系统 MUST 基于流的统计特征识别异常行为（DDoS、端口扫描、C2 心跳等）
- **FR-016**: 系统 MUST 在 AI 给出结论时附带原始数据证据
- **FR-017**: 系统 MUST 在发送数据到外部 AI 服务前进行正则脱敏（MAC 地址、内网 IP、敏感 Payload）
- **FR-018**: 系统 MUST 提供 CLI 命令行界面处理参数和交互
- **FR-019**: 系统 MUST 提供 TUI 终端用户界面显示实时流量统计和 AI 对话窗口
- **FR-020**: 系统 MUST 支持本地 LLM（如 Ollama）和云端 LLM API 两种模式
- **FR-021**: 系统 MUST 默认使用本地 LLM 保护隐私
- **FR-022**: 系统 MUST 在千兆网卡下实现丢包率 < 0.1%
- **FR-023**: 系统 MUST 实现协议解析与 DPI 分类延迟 < 1ms/包
- **FR-024**: 系统 MUST 严格封装所有 unsafe 代码块，确保内存安全
- **FR-025**: 系统 MUST 提供协议解析器的 Trait 接口，支持扩展自定义协议

### Key Entities

- **网络数据包 (Packet)**: 捕获的原始网络数据，包含时间戳、原始字节、网卡标识
- **五元组 (Five-tuple)**: 源 IP、目的 IP、源端口、目的端口、协议类型，标识唯一网络流
- **协议元数据 (Protocol Metadata)**: 协议类型、应用层字段（HTTP Host、DNS Query、TLS JA3 等）
- **流量会话 (Session)**: 由五元组标识的双向通信流，包含多个数据包和重组后的 Payload
- **AI 查询 (AI Query)**: 用户输入的自然语言查询，转换为 BPF/SQL 后的执行结果
- **异常事件 (Anomaly Event)**: AI 检测到的异常行为，包含类型、置信度、证据数据

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 用户能够在 30 秒内启动工具并开始抓包（从启动到看到第一个数据包）
- **SC-002**: 系统在千兆网卡下丢包率 < 0.1%（以 tcpdump 为基准对比）
- **SC-003**: 协议解析与 DPI 分类延迟 < 1ms/包（95 分位数）
- **SC-004**: 90% 的自然语言查询在 3 秒内返回结果（包括 AI 处理时间）
- **SC-005**: 用户能够使用自然语言完成 80% 的常见查询任务（无需学习 BPF/SQL 语法）
- **SC-006**: AI 给出的异常检测结论准确率达到 85% 以上（以人工标注为基准）
- **SC-007**: 系统连续运行 24 小时无内存泄漏（内存增长 < 5%）
- **SC-008**: 数据库查询响应时间 < 1 秒（百万级记录）
- **SC-009**: TUI 界面刷新延迟 < 200ms（用户感知流畅）
- **SC-010**: 敏感数据脱敏覆盖率 100%（发送到外部 AI 前）

## Clarifications

### Session 2026-07-16

- Q: MVP 范围应包含哪些优先级？ → A: 仅 P1-P2（抓包+DPI+存储）纳入 MVP，P3-P4（AI 分析+TUI）延后至后续版本
- Q: 数据库文件位置与数据保留策略？ → A: 用户目录 `~/.rustsniffer/`，自动清理 30 天前数据
- Q: 配置管理方式？ → A: 分层配置：命令行参数 > 配置文件（TOML） > 默认值
- Q: 日志记录策略？ → A: 结构化日志（JSON），文件轮转（100MB/文件，保留 5 个）
- Q: 原始数据包存储策略？ → A: `~/.rustsniffer/pcap/`，按天切片，保留 30 天

## Assumptions

- 目标用户具备基础网络知识（了解 IP、端口、协议等概念）
- 系统运行环境为 Linux（主要）和 Windows（次要），需要 libpcap/Npcap 支持
- 本地 LLM 推荐使用 Ollama + Llama-3 或 Qwen 模型，需要 8GB+ 内存
- 云端 LLM API 支持 OpenAI 兼容接口（如 OpenAI、Azure OpenAI、本地 Ollama）
- 原型阶段使用 SQLite 作为元数据存储，生产环境可切换至 ClickHouse
- 网络环境为普通千兆以太网，不考虑 10G+ 高速网络场景
- 用户已安装 Rust 工具链（stable 版本）
- DPI 引擎使用 nDPI 库，需要预先编译或通过 FFI 绑定
- AI 功能为辅助分析，不用于自动拦截或阻断网络流量
- **MVP 范围**：仅包含 P1（基础抓包）和 P2（DPI+存储）功能，P3（AI 分析）和 P4（TUI）延后至 v1.1+ 版本

## Configuration & Logging

### Configuration Strategy
- **分层配置**：命令行参数 > 配置文件 > 默认值
- **配置文件位置**：`~/.rustsniffer/config.toml`
- **配置文件格式**：TOML
- **配置项优先级**：
  1. 命令行参数（最高优先级）
  2. 配置文件中的设置
  3. 系统默认值（最低优先级）

### Logging Strategy
- **日志格式**：结构化 JSON 日志
- **日志位置**：`~/.rustsniffer/logs/`
- **日志轮转**：100MB/文件，保留 5 个历史文件
- **日志级别**：支持 ERROR、WARN、INFO、DEBUG、TRACE
- **默认级别**：INFO（生产环境），DEBUG（开发环境）
