# PDF Module 客户端配置指南

## 概述

本指南详细说明如何在不同客户端(Windows/Mac/Linux)通过Agent调用部署在服务器Docker中的PDF Module服务。

## 架构说明

```
┌─────────────────┐         ┌──────────────────┐
│  客户端 (Win)   │         │  服务器 (Linux)  │
│                 │         │                  │
│  ┌───────────┐  │         │  ┌────────────┐ │
│  │  Cursor   │  │  SSE    │  │  MCP SSE   │ │
│  │  Claude   │──┼─────────┼─►│  Server    │ │
│  │  Python   │  │  :8001  │  │  :8001     │ │
│  └───────────┘  │         │  └────────────┘ │
│                 │         │                  │
│  ┌───────────┐  │         │  ┌────────────┐ │
│  │  Browser  │  │  HTTP   │  │  REST API  │ │
│  │  Web UI   │──┼─────────┼─►│  :8000     │ │
│  └───────────┘  │  :3000  │  └────────────┘ │
└─────────────────┘         └──────────────────┘
```

## 方式一: 通过SSE远程连接 (推荐)

### 1. 服务器端配置

确保服务器已启动MCP SSE服务:

```bash
# 在服务器上执行
docker compose -f docker-compose.dev.yml up -d

# 或者单独启动MCP服务
docker run -d --name pdf-mcp-server \
  -p 8001:8001 \
  -v pdf_data:/app/data \
  smile9493/pdf-mcp:latest \
  pdf-mcp serve --transport sse --port 8001
```

### 2. 客户端配置 (Windows)

#### Cursor 配置

创建配置文件 `%USERPROFILE%\.cursor\mcp.json`:

```json
{
  "mcpServers": {
    "pdf-module-remote": {
      "url": "http://YOUR_SERVER_IP:8001/sse",
      "transport": "sse"
    }
  }
}
```

**注意**: 将 `YOUR_SERVER_IP` 替换为你的服务器IP地址。

#### Claude Desktop 配置

编辑配置文件 `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "pdf-module-remote": {
      "url": "http://YOUR_SERVER_IP:8001/sse",
      "transport": "sse"
    }
  }
}
```

### 3. 防火墙配置

确保服务器防火墙开放端口:

```bash
# 开放MCP SSE端口
sudo ufw allow 8001/tcp

# 开放REST API端口
sudo ufw allow 8000/tcp

# 开放Web UI端口
sudo ufw allow 3000/tcp
```

## 方式二: 通过REST API调用

### 1. Python客户端示例

```python
import requests

# 配置服务器地址
API_BASE = "http://YOUR_SERVER_IP:8000/api/v1/x2text"

# 提取PDF文本
def extract_text(pdf_path):
    with open(pdf_path, 'rb') as f:
        response = requests.post(
            f"{API_BASE}/extract",
            files={'file': f},
            params={'adapter': 'auto'}
        )
    return response.json()

# 搜索关键词
def search_keywords(pdf_path, keywords):
    with open(pdf_path, 'rb') as f:
        response = requests.post(
            f"{API_BASE}/search",
            files={'file': f},
            data={'keywords': ','.join(keywords)}
        )
    return response.json()

# 使用示例
result = extract_text("document.pdf")
print(result['text'])
```

### 2. JavaScript/TypeScript客户端示例

```typescript
const API_BASE = 'http://YOUR_SERVER_IP:8000/api/v1/x2text';

// 提取PDF文本
async function extractText(file: File) {
  const formData = new FormData();
  formData.append('file', file);
  
  const response = await fetch(`${API_BASE}/extract?adapter=auto`, {
    method: 'POST',
    body: formData
  });
  
  return response.json();
}

// 搜索关键词
async function searchKeywords(file: File, keywords: string[]) {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('keywords', keywords.join(','));
  
  const response = await fetch(`${API_BASE}/search`, {
    method: 'POST',
    body: formData
  });
  
  return response.json();
}
```

## 方式三: 通过Web UI访问

### 1. 访问Web界面

在浏览器中打开: `http://YOUR_SERVER_IP:3000`

### 2. Web界面功能

- **仪表盘**: 查看系统状态和性能指标
- **文本提取**: 上传PDF提取文本
- **关键词搜索**: 在PDF中搜索关键词
- **批量处理**: 批量处理多个PDF文件
- **引擎管理**: 查看和管理提取引擎
- **性能统计**: 查看缓存命中率和性能数据

## 方式四: 通过Python SDK调用

### 1. 安装SDK

```bash
pip install -e ./pdf-mcp-sdk
```

### 2. 使用SDK

```python
from pdf_mcp_sdk import PDFMCPClient

# 连接到远程服务器
client = PDFMCPClient("http://YOUR_SERVER_IP:8001")

# 提取文本
result = client.extract_text("document.pdf")
print(result.text)

# 搜索关键词
matches = client.search_keywords("document.pdf", ["关键词1", "关键词2"])
for match in matches:
    print(f"页码: {match.page}, 内容: {match.context}")
```

## 高级配置

### 1. 环境变量配置

在客户端创建 `.env` 文件:

```env
# 服务器配置
PDF_MODULE_HOST=YOUR_SERVER_IP
PDF_MODULE_MCP_PORT=8001
PDF_MODULE_REST_PORT=8000
PDF_MODULE_WEB_PORT=3000

# 连接配置
PDF_MODULE_TIMEOUT=30
PDF_MODULE_MAX_RETRIES=3
```

### 2. 配置文件示例

创建 `pdf_client_config.json`:

```json
{
  "server": {
    "host": "YOUR_SERVER_IP",
    "mcp_port": 8001,
    "rest_port": 8000,
    "web_port": 3000
  },
  "connection": {
    "timeout": 30,
    "max_retries": 3,
    "retry_delay": 1
  },
  "cache": {
    "enabled": true,
    "ttl": 3600
  }
}
```

### 3. 使用配置文件

```python
from pdf_mcp_sdk import PDFMCPClient
import json

# 加载配置
with open('pdf_client_config.json') as f:
    config = json.load(f)

# 创建客户端
client = PDFMCPClient(
    f"http://{config['server']['host']}:{config['server']['mcp_port']}",
    timeout=config['connection']['timeout']
)
```

## 安全配置

### 1. HTTPS配置 (推荐)

在服务器上配置Nginx反向代理:

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location /mcp {
        proxy_pass http://localhost:8001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
    }

    location /api {
        proxy_pass http://localhost:8000;
    }
}
```

### 2. API密钥认证

在服务器环境变量中添加:

```bash
export API_KEY=your-secret-key
```

客户端请求时添加认证头:

```python
headers = {"Authorization": "Bearer your-secret-key"}
response = requests.post(url, headers=headers, ...)
```

### 3. IP白名单

在服务器防火墙中配置IP白名单:

```bash
# 只允许特定IP访问
sudo ufw allow from YOUR_CLIENT_IP to any port 8001
sudo ufw allow from YOUR_CLIENT_IP to any port 8000
```

## 故障排查

### 1. 连接问题

```bash
# 测试服务器连通性
ping YOUR_SERVER_IP

# 测试端口开放
telnet YOUR_SERVER_IP 8001
telnet YOUR_SERVER_IP 8000

# 测试API健康检查
curl http://YOUR_SERVER_IP:8000/api/v1/x2text/health
```

### 2. 防火墙问题

```bash
# 检查防火墙状态
sudo ufw status

# 查看端口监听
sudo netstat -tlnp | grep -E '8000|8001|3000'
```

### 3. Docker问题

```bash
# 查看容器状态
docker ps

# 查看容器日志
docker logs pdf-module-api
docker logs pdf-module-web
```

## 性能优化建议

### 1. 网络优化

- 使用内网连接(如果客户端和服务器在同一内网)
- 启用HTTP/2
- 使用CDN加速静态资源

### 2. 客户端优化

- 启用本地缓存
- 使用连接池
- 批量处理请求

### 3. 服务器优化

- 增加缓存大小
- 启用Gzip压缩
- 使用负载均衡

## 完整示例: Windows + Cursor

### 步骤1: 服务器部署

```bash
# 在服务器上执行
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp
docker compose -f docker-compose.dev.yml up -d
```

### 步骤2: 客户端配置

在Windows上创建文件 `C:\Users\YourName\.cursor\mcp.json`:

```json
{
  "mcpServers": {
    "pdf-module": {
      "url": "http://192.168.1.100:8001/sse",
      "transport": "sse"
    }
  }
}
```

### 步骤3: 重启Cursor

重启Cursor IDE,PDF工具将自动可用。

### 步骤4: 使用示例

在Cursor中,你可以直接让AI调用PDF工具:

```
请帮我提取这个PDF文件的文本内容: /path/to/document.pdf
```

```
在这个PDF中搜索关键词"人工智能": /path/to/document.pdf
```

## 总结

通过以上配置,你可以在任何客户端(Windows/Mac/Linux)通过以下方式调用服务器上的PDF Module:

1. **MCP SSE** - 适合Cursor、Claude Desktop等AI Agent
2. **REST API** - 适合自定义应用集成
3. **Web UI** - 适合手动操作和可视化
4. **Python SDK** - 适合Python应用集成

选择最适合你需求的方式即可!
