# CLI Contract: RustSniffer-AI

**Branch**: `001-ai-traffic-analyzer` | **Date**: 2026-07-16

## Overview

RustSniffer-AI 提供命令行接口（CLI）作为主要交互方式。所有功能通过子命令和参数暴露。

## Command Structure

```
rustsniffer <SUBCOMMAND> [OPTIONS]
```

## Subcommands

### 1. `capture` - 启动抓包

**描述**: 启动网络抓包，实时显示数据包信息

**用法**:
```bash
rustsniffer capture [OPTIONS]
```

**参数**:
| 参数 | 短名 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|------|--------|------|
| `--interface` | `-i` | String | 否 | 自动选择第一个 | 指定网卡名称或索引 |
| `--filter` | `-f` | String | 否 | 无 | BPF 过滤表达式 |
| `--promiscuous` | `-p` | Flag | 否 | false | 启用混杂模式 |
| `--output` | `-o` | String | 否 | 终端 | 输出到文件（PCAPNG 格式） |
| `--count` | `-c` | u64 | 否 | 0（无限） | 捕获包数量，0 表示无限 |
| `--duration` | `-d` | u64 | 否 | 0（无限） | 捕获时长（秒），0 表示无限 |

**示例**:
```bash
# 捕获所有流量
rustsniffer capture

# 指定网卡和过滤器
rustsniffer capture -i eth0 -f "port 80"

# 捕获 100 个包并保存到文件
rustsniffer capture -c 100 -o output.pcapng
```

**退出码**:
- `0` - 正常退出
- `1` - 网卡不可用或权限不足
- `2` - BPF 过滤器语法错误
- `3` - 输出文件写入失败

---

### 2. `list-interfaces` - 列出可用网卡

**描述**: 枚举系统可用网卡并显示信息

**用法**:
```bash
rustsniffer list-interfaces [OPTIONS]
```

**参数**:
| 参数 | 短名 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|------|--------|------|
| `--verbose` | `-v` | Flag | 否 | false | 显示详细信息（IP 地址、MAC 等） |

**示例**:
```bash
# 列出所有网卡
rustsniffer list-interfaces

# 显示详细信息
rustsniffer list-interfaces -v
```

**输出格式**:
```
Available interfaces:
  1. eth0 (Ethernet)
  2. wlan0 (Wi-Fi)
  3. lo (Loopback)
```

---

### 3. `query` - 查询元数据

**描述**: 查询已存储的流量元数据

**用法**:
```bash
rustsniffer query [OPTIONS] <SQL_QUERY>
```

**参数**:
| 参数 | 短名 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|------|--------|------|
| `--format` | `-F` | String | 否 | table | 输出格式（table/json/csv） |
| `--limit` | `-l` | u32 | 否 | 100 | 最大返回记录数 |
| `--since` | `-s` | String | 否 | 无 | 起始时间（ISO 8601） |
| `--until` | `-u` | String | 否 | 无 | 结束时间（ISO 8601） |

**示例**:
```bash
# 查询过去 5 分钟的 DNS 查询
rustsniffer query "SELECT * FROM packets WHERE app_protocol='DNS'" --since "5m ago"

# 查询访问次数最多的 IP
rustsniffer query "SELECT src_ip, COUNT(*) as cnt FROM packets GROUP BY src_ip ORDER BY cnt DESC LIMIT 10"

# 输出为 JSON 格式
rustsniffer query "SELECT * FROM packets" --format json --limit 10
```

**退出码**:
- `0` - 查询成功
- `1` - SQL 语法错误
- `2` - 数据库不可用

---

### 4. `config` - 配置管理

**描述**: 查看和修改配置

**用法**:
```bash
rustsniffer config <SUBCOMMAND>
```

**子命令**:
- `show` - 显示当前配置
- `set <KEY> <VALUE>` - 设置配置项
- `get <KEY>` - 获取配置项值
- `reset` - 重置为默认配置

**示例**:
```bash
# 显示所有配置
rustsniffer config show

# 设置日志级别
rustsniffer config set logging.level debug

# 获取数据库路径
rustsniffer config get storage.db_path
```

---

### 5. `stats` - 统计信息

**描述**: 显示抓包统计信息

**用法**:
```bash
rustsniffer stats [OPTIONS]
```

**参数**:
| 参数 | 短名 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|------|--------|------|
| `--period` | `-p` | String | 否 | 1h | 统计时间段（如 5m, 1h, 24h） |
| `--top` | `-t` | u32 | 否 | 10 | Top N 统计项 |

**示例**:
```bash
# 显示过去 1 小时统计
rustsniffer stats

# 显示过去 24 小时 Top 20 IP
rustsniffer stats --period 24h --top 20
```

**输出示例**:
```
=== Traffic Statistics (Last 1 hour) ===

Total packets: 125,430
Total bytes: 89.2 MB
Packets/sec: 34.8

Top 10 Source IPs:
  1. 192.168.1.100    45,230 packets (36.1%)
  2. 192.168.1.101    23,450 packets (18.7%)
  ...

Protocol Distribution:
  TCP: 89,230 (71.1%)
  UDP: 34,200 (27.3%)
  ICMP: 2,000 (1.6%)
```

---

### 6. `version` - 版本信息

**描述**: 显示版本和构建信息

**用法**:
```bash
rustsniffer version
```

**输出**:
```
RustSniffer-AI v0.1.0
Build: 2026-07-16
Rust: 1.75.0
Platform: x86_64-unknown-linux-gnu
```

---

## Global Options

以下选项适用于所有子命令：

| 参数 | 短名 | 类型 | 默认值 | 描述 |
|------|------|------|--------|------|
| `--config` | `-C` | String | ~/.rustsniffer/config.toml | 配置文件路径 |
| `--log-level` | `-L` | String | info | 日志级别（error/warn/info/debug/trace） |
| `--quiet` | `-q` | Flag | false | 静默模式，仅输出错误 |
| `--verbose` | `-v` | Flag | false | 详细模式，输出调试信息 |
| `--help` | `-h` | Flag | - | 显示帮助信息 |
| `--version` | `-V` | Flag | - | 显示版本号 |

---

## Exit Codes

| 退出码 | 含义 |
|--------|------|
| `0` | 成功 |
| `1` | 通用错误 |
| `2` | 参数错误 |
| `3` | 权限不足 |
| `4` | 网卡不可用 |
| `5` | 数据库错误 |
| `6` | 配置文件错误 |

---

## Signal Handling

- **SIGINT (Ctrl+C)**: 优雅停止抓包，显示统计摘要后退出
- **SIGTERM**: 同 SIGINT
- **SIGHUP**: 重新加载配置文件

---

## Environment Variables

| 变量 | 描述 | 示例 |
|------|------|------|
| `RUSTSNIFFER_CONFIG` | 配置文件路径 | `/etc/rustsniffer/config.toml` |
| `RUSTSNIFFER_LOG_LEVEL` | 日志级别 | `debug` |
| `RUSTSNIFFER_DB_PATH` | 数据库路径 | `/var/lib/rustsniffer/metadata.db` |

---

## Error Messages

所有错误信息 MUST 包含：
1. 错误类型
2. 错误原因
3. 解决建议（如适用）

**示例**:
```
Error: 网卡不可用
原因: 无法打开网卡 'eth99'
建议: 请检查网卡名称是否正确，或运行 'rustsniffer list-interfaces' 查看可用网卡
```
