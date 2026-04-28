# PDF Module 客户端配置指南

## 概述

本指南说明如何配置 Agent (Cursor/Claude Desktop) 连接 PDF Module MCP 服务。

## 架构

```
┌─────────────────┐         ┌──────────────────┐
│  客户端         │         │  服务器          │
│                 │         │                  │
│  ┌───────────┐  │         │  ┌────────────┐ │
│  │  Cursor   │  │  SSE    │  │  MCP SSE   │ │
│  │  Claude   │──┼─────────┼─►│  Server    │ │
│  └───────────┘  │  :8001  │  │  :8001     │ │
└─────────────────┘         │  └────────────┘ │
                            └──────────────────┘
```

## 方式一: 远程 SSE 连接 (推荐)

### 1. 服务器端启动

```bash
docker compose up -d
```

### 2. Cursor 配置

创建 `~/.cursor/mcp.json` (Mac/Linux) 或 `%USERPROFILE%\.cursor\mcp.json` (Windows):

```json
{
  "mcpServers": {
    "pdf-module": {
      "url": "http://YOUR_SERVER_IP:8001/sse",
      "transport": "sse"
    }
  }
}
```

### 3. Claude Desktop 配置

编辑配置文件:
- Mac: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "pdf-module": {
      "url": "http://YOUR_SERVER_IP:8001/sse",
      "transport": "sse"
    }
  }
}
```

### 4. 防火墙配置

```bash
sudo ufw allow 8001/tcp
```

## 方式二: 本地 stdio 连接

适合服务器和客户端在同一机器:

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "docker",
      "args": ["exec", "-i", "pdf-mcp-server", "pdf-mcp"]
    }
  }
}
```

## 可用工具

配置完成后，Agent 可调用:

| 工具 | 说明 |
|------|------|
| `extract_text` | 提取 PDF 纯文本 |
| `extract_structured` | 提取结构化数据 (per-page + bbox) |
| `get_page_count` | 获取页数 |

## 故障排查

```bash
# 测试端口连通
telnet YOUR_SERVER_IP 8001

# 查看容器日志
docker logs pdf-mcp-server
```
