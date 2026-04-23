# API 文档

本文档提供了 PDF Module MCP 服务器的完整 API 参考。

## MCP API

### 基础信息

- **协议**: JSON-RPC 2.0
- **传输**: stdio / SSE
- **版本**: 2024-11-05

### 初始化请求

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {}
}
```

### 响应

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "serverInfo": {
      "name": "pdf-module-mcp",
      "version": "0.1.0"
    },
    "capabilities": {
      "tools": {}
    }
  }
}
```

## MCP 工具

### 1. extract_text

提取 PDF 文件的纯文本内容。

**参数**:
- `file_path` (string, required): PDF 文件的绝对路径
- `adapter` (string, optional): 提取引擎,可选值: lopdf, pdf-extract, pdfium, pymupdf, pdfplumber

**示例请求**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "extract_text",
    "arguments": {
      "file_path": "/path/to/file.pdf",
      "adapter": "lopdf"
    }
  }
}
```

**示例响应**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "PDF 文件的文本内容..."
      }
    ]
  }
}
```

### 2. extract_structured

提取 PDF 文件的结构化数据,包含页面信息和位置信息。

**参数**:
- `file_path` (string, required): PDF 文件的绝对路径
- `adapter` (string, optional): 提取引擎
- `enable_highlight` (boolean, optional): 是否包含高亮元数据

**示例请求**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "extract_structured",
    "arguments": {
      "file_path": "/path/to/file.pdf",
      "adapter": "lopdf",
      "enable_highlight": true
    }
  }
}
```

**示例响应**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"pages\": [\n    {\n      \"page_number\": 1,\n      \"text\": \"...\",\n      \"blocks\": [...]\n    }\n  ]\n}"
      }
    ]
  }
}
```

### 3. get_page_count

获取 PDF 文件的页数。

**参数**:
- `file_path` (string, required): PDF 文件的绝对路径

**示例请求**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "get_page_count",
    "arguments": {
      "file_path": "/path/to/file.pdf"
    }
  }
}
```

**示例响应**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "10"
      }
    ]
  }
}
```

### 4. search_keywords

在 PDF 文件中搜索关键词。

**参数**:
- `file_path` (string, required): PDF 文件的绝对路径
- `keywords` (array of strings, required): 要搜索的关键词列表
- `case_sensitive` (boolean, optional): 是否区分大小写,默认 false

**示例请求**:
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "search_keywords",
    "arguments": {
      "file_path": "/path/to/file.pdf",
      "keywords": ["PDF", "文档"],
      "case_sensitive": false
    }
  }
}
```

**示例响应**:
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"keywords\": {\n    \"PDF\": [\n      {\"page\": 1, \"position\": 0, \"context\": \"...\"}\n    ],\n    \"文档\": [...]\n  }\n}"
      }
    ]
  }
}
```

### 5. extract_keywords

自动提取 PDF 文件中的高频关键词。

**参数**:
- `file_path` (string, required): PDF 文件的绝对路径
- `top_n` (integer, optional): 返回前 N 个关键词,默认 10

**示例请求**:
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "extract_keywords",
    "arguments": {
      "file_path": "/path/to/file.pdf",
      "top_n": 10
    }
  }
}
```

**示例响应**:
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "[{\"keyword\": \"PDF\", \"frequency\": 15}, {\"keyword\": \"文档\", \"frequency\": 10}]"
      }
    ]
  }
}
```

### 6. list_adapters

列出所有可用的 PDF 提取引擎。

**参数**: 无

**示例请求**:
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "list_adapters",
    "arguments": {}
  }
}
```

**示例响应**:
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "[{\"name\": \"lopdf\", \"version\": \"0.34.0\", \"description\": \"Pure Rust PDF library\"}, {\"name\": \"pdf-extract\", \"version\": \"0.7.0\", \"description\": \"PDF text extraction\"}]"
      }
    ]
  }
}
```

### 7. cache_stats

获取缓存统计信息。

**参数**: 无

**示例请求**:
```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "tools/call",
  "params": {
    "name": "cache_stats",
    "arguments": {}
  }
}
```

**示例响应**:
```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"size\": 5, \"max_size\": 100, \"hits\": 10, \"misses\": 2, \"hit_rate\": 0.8333}"
      }
    ]
  }
}
```

## REST API

### 基础信息

- **协议**: HTTP/1.1
- **端口**: 8000 (默认)
- **内容类型**: application/json, multipart/form-data

### 健康检查

**端点**: `GET /api/v1/x2text/health`

**响应**:
```
OK
```

### 提取文本

**端点**: `POST /api/v1/x2text/extract`

**请求**: multipart/form-data
- `file`: PDF 文件
- `adapter` (可选): 提取引擎

**响应**: text/plain
```
PDF 文件的文本内容...
```

### 提取结构化数据

**端点**: `POST /api/v1/x2text/extract-json`

**请求**: multipart/form-data
- `file`: PDF 文件
- `adapter` (可选): 提取引擎

**响应**: application/json
```json
{
  "workflow_id": "uuid",
  "elapsed_time": 150,
  "output": {
    "pages": [...]
  },
  "metadata": {
    "file_name": "file.pdf",
    "file_size": 1024000,
    "processing_time": 150,
    "cache_hit": false,
    "adapter_used": "lopdf"
  }
}
```

### 获取 PDF 信息

**端点**: `POST /api/v1/x2text/info`

**请求**: multipart/form-data
- `file`: PDF 文件

**响应**: application/json
```json
{
  "filename": "file.pdf",
  "page_count": 10,
  "mime_type": "application/pdf"
}
```

### 列出提取引擎

**端点**: `GET /api/v1/x2text/adapters`

**响应**: application/json
```json
{
  "adapters": [
    {
      "name": "lopdf",
      "version": "0.34.0",
      "description": "Pure Rust PDF library"
    }
  ]
}
```

### 获取缓存统计

**端点**: `GET /api/v1/x2text/cache/stats`

**响应**: application/json
```json
{
  "size": 5,
  "max_size": 100,
  "hits": 10,
  "misses": 2,
  "hit_rate": 0.8333
}
```

## 错误响应

### MCP 错误格式

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params"
  }
}
```

### REST 错误格式

```json
{
  "error": "Bad Request",
  "message": "No file uploaded",
  "status_code": 400
}
```

## HTTP 状态码

- `200 OK` - 请求成功
- `400 Bad Request` - 请求参数错误
- `404 Not Found` - 资源不存在
- `500 Internal Server Error` - 服务器内部错误

## 速率限制

当前版本未实施速率限制,建议在生产环境中使用反向代理(如 Nginx)来实施速率限制。

## CORS 支持

默认启用 CORS,允许所有来源。可以通过环境变量配置:
- `ENABLE_CORS`: true/false
- `ALLOWED_ORIGINS`: 允许的源列表,逗号分隔

## 安全性

- 所有文件路径都经过安全验证
- 文件类型严格限制为 PDF
- 文件大小限制默认为 100MB
- 支持 HTTPS 加密传输
