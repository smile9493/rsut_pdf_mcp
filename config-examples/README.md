# PDF Module 客户端配置示例

本目录包含各种客户端的配置示例文件,帮助你快速配置客户端连接到远程PDF Module服务。

## 文件说明

### Cursor配置

- **cursor-mcp-local.json** - 本地Docker连接配置
- **cursor-mcp-remote.json** - 远程服务器连接配置

### Claude Desktop配置

- **claude-desktop-config.json** - Claude Desktop配置示例

### Python客户端

- **python_client_example.py** - Python REST API客户端示例

## 快速开始

### 1. Cursor配置 (Windows)

#### 步骤1: 复制配置文件

```bash
# 创建Cursor配置目录
mkdir %USERPROFILE%\.cursor

# 复制远程配置文件
copy cursor-mcp-remote.json %USERPROFILE%\.cursor\mcp.json
```

#### 步骤2: 修改服务器地址

编辑 `%USERPROFILE%\.cursor\mcp.json`,将 `YOUR_SERVER_IP` 替换为你的服务器IP:

```json
{
  "mcpServers": {
    "pdf-module-remote": {
      "url": "http://192.168.1.100:8001/sse",
      "transport": "sse"
    }
  }
}
```

#### 步骤3: 重启Cursor

重启Cursor IDE,PDF工具将自动可用。

### 2. Claude Desktop配置 (Windows)

#### 步骤1: 复制配置文件

```bash
# 创建Claude配置目录
mkdir %APPDATA%\Claude

# 复制配置文件
copy claude-desktop-config.json %APPDATA%\Claude\claude_desktop_config.json
```

#### 步骤2: 修改服务器地址

编辑配置文件,将 `YOUR_SERVER_IP` 替换为你的服务器IP。

#### 步骤3: 重启Claude Desktop

重启Claude Desktop应用。

### 3. Python客户端

#### 步骤1: 安装依赖

```bash
pip install requests
```

#### 步骤2: 修改服务器地址

编辑 `python_client_example.py`,将 `YOUR_SERVER_IP` 替换为你的服务器IP:

```python
client = PDFModuleClient(host="192.168.1.100", port=8000)
```

#### 步骤3: 运行示例

```bash
python python_client_example.py
```

## 配置说明

### MCP SSE连接 (推荐)

适用于Cursor、Claude Desktop等AI Agent客户端。

**优点**:
- 实时双向通信
- 支持所有MCP工具
- 自动集成到AI对话

**配置要点**:
- URL格式: `http://SERVER_IP:8001/sse`
- Transport类型: `sse`
- 端口: 8001 (MCP SSE服务端口)

### REST API连接

适用于自定义应用、脚本集成。

**优点**:
- 简单易用
- 支持所有语言
- 无需特殊客户端

**配置要点**:
- URL格式: `http://SERVER_IP:8000/api/v1/x2text`
- 端口: 8000 (REST API端口)
- 使用HTTP POST请求

### Web UI访问

适用于手动操作、可视化查看。

**访问地址**: `http://SERVER_IP:3000`

**功能**:
- 文本提取
- 关键词搜索
- 批量处理
- 性能监控

## 服务器要求

### 端口开放

确保服务器防火墙开放以下端口:

| 端口 | 服务 | 用途 |
|------|------|------|
| 8001 | MCP SSE | AI Agent连接 |
| 8000 | REST API | HTTP API调用 |
| 3000 | Web UI | 浏览器访问 |

### 防火墙配置

```bash
# Ubuntu/Debian
sudo ufw allow 8001/tcp
sudo ufw allow 8000/tcp
sudo ufw allow 3000/tcp

# CentOS/RHEL
sudo firewall-cmd --add-port=8001/tcp --permanent
sudo firewall-cmd --add-port=8000/tcp --permanent
sudo firewall-cmd --add-port=3000/tcp --permanent
sudo firewall-cmd --reload
```

### Docker服务状态

确保Docker服务正常运行:

```bash
# 查看服务状态
docker compose -f docker-compose.dev.yml ps

# 查看日志
docker compose -f docker-compose.dev.yml logs
```

## 故障排查

### 连接失败

1. **检查服务器IP**: 确保IP地址正确
2. **检查端口**: 确保端口已开放
3. **检查防火墙**: 确保防火墙允许连接
4. **检查服务**: 确保Docker服务正在运行

### 测试连接

```bash
# 测试API健康检查
curl http://YOUR_SERVER_IP:8000/api/v1/x2text/health

# 测试端口连通性
telnet YOUR_SERVER_IP 8001
telnet YOUR_SERVER_IP 8000
```

### 查看日志

```bash
# 查看API日志
docker logs pdf-module-api

# 查看Web日志
docker logs pdf-module-web
```

## 安全建议

### 1. 使用HTTPS

在生产环境中,建议使用HTTPS:

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location /mcp {
        proxy_pass http://localhost:8001;
    }
}
```

### 2. IP白名单

限制只允许特定IP访问:

```bash
sudo ufw allow from YOUR_CLIENT_IP to any port 8001
```

### 3. API密钥

添加API密钥认证:

```bash
export API_KEY=your-secret-key
```

## 更多信息

详细配置指南请查看: [CLIENT_SETUP_GUIDE.md](../docs/CLIENT_SETUP_GUIDE.md)

完整文档请查看: [README.md](../README.md)
