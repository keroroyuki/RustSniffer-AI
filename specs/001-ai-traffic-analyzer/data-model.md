# Data Model: Rust 智能抓包与流量分析工具

**Branch**: `001-ai-traffic-analyzer` | **Date**: 2026-07-16

## Core Entities

### 1. 网络数据包 (Packet)

**描述**: 捕获的原始网络数据，是系统处理的最小单元

**字段**:
- `id`: u64 - 唯一标识符（数据库自增）
- `timestamp`: i64 - 捕获时间戳（Unix 毫秒）
- `raw_bytes`: Vec<u8> - 原始数据包字节
- `interface_id`: String - 网卡标识
- `pcap_file`: String - PCAPNG 文件路径
- `pcap_offset`: u64 - 在文件中的偏移量

**关系**: 
- 属于一个 `Session`（通过五元组关联）
- 包含一个 `ProtocolMetadata`（解析后的协议信息）

**验证规则**:
- `timestamp` MUST > 0
- `raw_bytes` MUST 非空
- `pcap_file` MUST 为有效路径

---

### 2. 五元组 (FiveTuple)

**描述**: 标识唯一网络流的五元组组合

**字段**:
- `src_ip`: IpAddr - 源 IP 地址（IPv4/IPv6）
- `dst_ip`: IpAddr - 目的 IP 地址
- `src_port`: u16 - 源端口（TCP/UDP）
- `dst_port`: u16 - 目的端口
- `protocol`: Protocol - 传输层协议（TCP/UDP/ICMP）

**唯一性规则**:
- 五元组 + 时间窗口（如 5 分钟）标识一个会话
- 同一五元组在不同时间窗口可能属于不同会话

**关系**:
- 标识一个 `Session`
- 被多个 `Packet` 共享

**验证规则**:
- IP 地址 MUST 为有效格式
- 端口号 MUST 在 0-65535 范围内
- `protocol` MUST 为有效协议类型

---

### 3. 协议元数据 (ProtocolMetadata)

**描述**: 协议解析后的结构化信息

**字段**:
- `packet_id`: u64 - 关联的数据包 ID
- `ethernet_type`: u16 - Ethernet 类型（0x0800=IPv4, 0x86DD=IPv6）
- `ip_version`: u8 - IP 版本（4 或 6）
- `ip_ttl`: u8 - IP TTL 值
- `transport_protocol`: String - 传输层协议名称
- `app_protocol`: Option<String> - DPI 识别的应用层协议
- `app_metadata`: Option<serde_json::Value> - 应用层字段（JSON）
  - HTTP: `{ "method": "GET", "host": "example.com", "uri": "/path" }`
  - DNS: `{ "query": "example.com", "type": "A" }`
  - TLS: `{ "version": "1.3", "ja3": "...", "ja3s": "..." }`

**关系**:
- 属于一个 `Packet`
- 关联一个 `Session`

**验证规则**:
- `packet_id` MUST 存在于 packets 表
- `app_protocol` MUST 为 DPI 引擎支持的值或 null
- `app_metadata` MUST 为有效 JSON

---

### 4. 流量会话 (Session)

**描述**: 由五元组标识的双向通信流

**字段**:
- `id`: u64 - 会话唯一标识符
- `five_tuple`: FiveTuple - 五元组
- `start_time`: i64 - 会话开始时间
- `end_time`: i64 - 会话结束时间
- `packet_count`: u32 - 数据包数量
- `total_bytes`: u64 - 总字节数
- `reassembly_status`: ReassemblyStatus - TCP 重组状态
- `reassembly_failed_count`: u32 - 重组失败次数

**状态转换**:
```
Created → Active → Completed
                  → Failed (重组失败)
```

**ReassemblyStatus 枚举**:
- `NotApplicable` - 非 TCP 协议
- `InProgress` - 重组中
- `Completed` - 重组成功
- `Failed(missing_count: u32)` - 重组失败，缺失 N 个包

**关系**:
- 包含多个 `Packet`
- 关联一个 `ProtocolMetadata`（首个包的元数据）

**验证规则**:
- `start_time` MUST <= `end_time`
- `packet_count` MUST > 0
- `total_bytes` MUST > 0

---

### 5. AI 查询 (AIQuery)

**描述**: 用户输入的自然语言查询及执行结果

**字段**:
- `id`: u64 - 查询唯一标识符
- `timestamp`: i64 - 查询时间
- `natural_language`: String - 用户输入的自然语言
- `generated_query`: String - 生成的 BPF/SQL 查询
- `query_type`: QueryType - 查询类型（BPF/SQL）
- `result_count`: u32 - 结果数量
- `execution_time_ms`: u32 - 执行时间（毫秒）
- `success`: bool - 是否成功
- `error_message`: Option<String> - 错误信息

**QueryType 枚举**:
- `BPF` - BPF 过滤表达式
- `SQL` - SQL 查询
- `Hybrid` - BPF + SQL 组合

**关系**:
- 无直接关联，独立记录

**验证规则**:
- `natural_language` MUST 非空
- `generated_query` MUST 非空
- `execution_time_ms` MUST >= 0

---

### 6. 异常事件 (AnomalyEvent)

**描述**: AI 检测到的异常行为

**字段**:
- `id`: u64 - 事件唯一标识符
- `timestamp`: i64 - 检测时间
- `anomaly_type`: AnomalyType - 异常类型
- `confidence`: f32 - 置信度（0.0-1.0）
- `evidence`: serde_json::Value - 证据数据（JSON）
- `related_session_id`: Option<u64> - 关联的会话 ID
- `status`: AnomalyStatus - 处理状态
- `description`: String - 异常描述

**AnomalyType 枚举**:
- `PortScan` - 端口扫描
- `DDoS` - DDoS 攻击
- `C2Heartbeat` - C2 心跳
- `SuspiciousPayload` - 可疑 Payload
- `Unknown` - 未知异常

**AnomalyStatus 枚举**:
- `Pending` - 待确认
- `Confirmed` - 已确认
- `Dismissed` - 已忽略

**关系**:
- 可选关联一个 `Session`

**验证规则**:
- `confidence` MUST 在 0.0-1.0 范围内
- `evidence` MUST 为有效 JSON
- `description` MUST 非空

---

## Database Schema

### packets 表

```sql
CREATE TABLE packets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    src_ip TEXT NOT NULL,
    dst_ip TEXT NOT NULL,
    src_port INTEGER,
    dst_port INTEGER,
    protocol TEXT NOT NULL,
    app_protocol TEXT,
    packet_size INTEGER NOT NULL,
    direction TEXT NOT NULL,
    metadata TEXT,
    pcap_file TEXT NOT NULL,
    pcap_offset INTEGER NOT NULL,
    session_id INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE INDEX idx_packets_timestamp ON packets(timestamp);
CREATE INDEX idx_packets_src_ip ON packets(src_ip);
CREATE INDEX idx_packets_dst_ip ON packets(dst_ip);
CREATE INDEX idx_packets_protocol ON packets(protocol);
CREATE INDEX idx_packets_app_protocol ON packets(app_protocol);
CREATE INDEX idx_packets_session_id ON packets(session_id);
```

### sessions 表

```sql
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    src_ip TEXT NOT NULL,
    dst_ip TEXT NOT NULL,
    src_port INTEGER,
    dst_port INTEGER,
    protocol TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    packet_count INTEGER NOT NULL,
    total_bytes INTEGER NOT NULL,
    reassembly_status TEXT NOT NULL,
    reassembly_failed_count INTEGER DEFAULT 0
);

CREATE INDEX idx_sessions_start_time ON sessions(start_time);
CREATE INDEX idx_sessions_src_ip ON sessions(src_ip);
CREATE INDEX idx_sessions_dst_ip ON sessions(dst_ip);
```

### ai_queries 表

```sql
CREATE TABLE ai_queries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    natural_language TEXT NOT NULL,
    generated_query TEXT NOT NULL,
    query_type TEXT NOT NULL,
    result_count INTEGER NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    success INTEGER NOT NULL,
    error_message TEXT
);

CREATE INDEX idx_ai_queries_timestamp ON ai_queries(timestamp);
```

### anomaly_events 表

```sql
CREATE TABLE anomaly_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    anomaly_type TEXT NOT NULL,
    confidence REAL NOT NULL,
    evidence TEXT NOT NULL,
    related_session_id INTEGER,
    status TEXT NOT NULL,
    description TEXT NOT NULL,
    FOREIGN KEY (related_session_id) REFERENCES sessions(id)
);

CREATE INDEX idx_anomaly_timestamp ON anomaly_events(timestamp);
CREATE INDEX idx_anomaly_type ON anomaly_events(anomaly_type);
CREATE INDEX idx_anomaly_status ON anomaly_events(status);
```

---

## Data Flow

```
1. 采集层捕获 Packet
   ↓
2. 协议解析层提取 ProtocolMetadata
   ↓
3. DPI 引擎识别 app_protocol
   ↓
4. 流重组模块关联到 Session
   ↓
5. 存储层写入数据库和 PCAPNG 文件
   ↓
6. 查询层提供检索接口
   ↓
7. AI 层（MVP 后）执行分析和异常检测
```

---

## Data Retention

- **元数据**: 保留 30 天，每日凌晨自动清理
- **PCAPNG 文件**: 保留 30 天，按天切片
- **日志文件**: 保留 5 个文件，每个最大 100MB
- **AI 查询记录**: 保留 90 天
- **异常事件**: 永久保留（除非手动清理）

---

## Data Validation Summary

| Entity | Key Validations |
|--------|----------------|
| Packet | timestamp > 0, raw_bytes 非空, pcap_file 有效 |
| FiveTuple | IP 有效, 端口 0-65535, protocol 有效 |
| ProtocolMetadata | packet_id 存在, app_metadata 有效 JSON |
| Session | start_time <= end_time, packet_count > 0 |
| AIQuery | natural_language 非空, execution_time_ms >= 0 |
| AnomalyEvent | confidence 0.0-1.0, evidence 有效 JSON |
