# 配置指南

本文档详细说明了 PDF Module MCP 服务器的配置选项。

## 环境变量配置

项目使用环境变量进行配置,支持 `.env` 文件。

### 服务器配置

#### HOST
服务器监听地址。

- **类型**: string
- **默认值**: 127.0.0.1
- **示例**: HOST=0.0.0.0

#### PORT
服务器监听端口。

- **类型**: number
- **默认值**: 8000
- **示例**: PORT=8080

### 存储配置

#### STORAGE_TYPE
存储后端类型。

- **类型**: string
- **可选值**: local, s3, gcs, azure
- **默认值**: local
- **示例**: STORAGE_TYPE=s3

#### STORAGE_LOCAL_DIR
本地存储目录路径。

- **类型**: string
- **默认值**: ./data
- **示例**: STORAGE_LOCAL_DIR=/var/lib/pdf-module/data

#### STORAGE_S3_BUCKET
S3 存储桶名称。

- **类型**: string
- **必需**: 当 STORAGE_TYPE=s3 时必需
- **示例**: STORAGE_S3_BUCKET=my-pdf-bucket

#### STORAGE_S3_REGION
S3 区域。

- **类型**: string
- **默认值**: us-east-1
- **示例**: STORAGE_S3_REGION=ap-northeast-1

#### STORAGE_S3_ACCESS_KEY
S3 访问密钥。

- **类型**: string
- **必需**: 当使用需要认证的 S3 服务时
- **示例**: STORAGE_S3_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE

#### STORAGE_S3_SECRET_KEY
S3 秘密密钥。

- **类型**: string
- **必需**: 当使用需要认证的 S3 服务时
- **示例**: STORAGE_S3_SECRET_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY

#### STORAGE_S3_PREFIX
S3 存储前缀。

- **类型**: string
- **可选**: 是
- **示例**: STORAGE_S3_PREFIX=pdf-files/

#### STORAGE_S3_ENDPOINT
S3 兼容服务的自定义端点。

- **类型**: string
- **可选**: 是
- **示例**: STORAGE_S3_ENDPOINT=https://s3.example.com

#### STORAGE_GCS_BUCKET
Google Cloud Storage 存储桶名称。

- **类型**: string
- **必需**: 当 STORAGE_TYPE=gcs 时必需
- **示例**: STORAGE_GCS_BUCKET=my-pdf-bucket

#### STORAGE_GCS_CREDENTIALS_PATH
GCS 服务账户凭证文件路径。

- **类型**: string
- **必需**: 当 STORAGE_TYPE=gcs 时必需
- **示例**: STORAGE_GCS_CREDENTIALS_PATH=/path/to/service-account.json

#### STORAGE_AZURE_ACCOUNT
Azure 存储账户名称。

- **类型**: string
- **必需**: 当 STORAGE_TYPE=azure 时必需
- **示例**: STORAGE_AZURE_ACCOUNT=mystorageaccount

#### STORAGE_AZURE_KEY
Azure 存储账户密钥。

- **类型**: string
- **必需**: 当 STORAGE_TYPE=azure 时必需
- **示例**: STORAGE_AZURE_KEY=your-storage-account-key

#### STORAGE_AZURE_CONTAINER
Azure Blob 存储容器名称。

- **类型**: string
- **必需**: 当 STORAGE_TYPE=azure 时必需
- **示例**: STORAGE_AZURE_CONTAINER=pdf-files

### 缓存配置

#### CACHE_ENABLED
是否启用缓存。

- **类型**: boolean
- **默认值**: true
- **可选值**: true, false
- **示例**: CACHE_ENABLED=true

#### CACHE_MAX_SIZE_MB
缓存最大大小(MB)。

- **类型**: number
- **默认值**: 100
- **示例**: CACHE_MAX_SIZE_MB=500

#### CACHE_TTL_SECONDS
缓存过期时间(秒)。

- **类型**: number
- **默认值**: 3600 (1小时)
- **示例**: CACHE_TTL_SECONDS=7200

### 审计配置

#### AUDIT_ENABLED
是否启用审计日志。

- **类型**: boolean
- **默认值**: true
- **可选值**: true, false
- **示例**: AUDIT_ENABLED=true

#### AUDIT_RETENTION_DAYS
审计日志保留天数。

- **类型**: number
- **默认值**: 30
- **示例**: AUDIT_RETENTION_DAYS=90

#### AUDIT_LOG_DIR
审计日志存储目录。

- **类型**: string
- **默认值**: ./logs/audit
- **示例**: AUDIT_LOG_DIR=/var/log/pdf-module/audit

### 日志配置

#### LOG_LEVEL
日志级别。

- **类型**: string
- **默认值**: info
- **可选值**: trace, debug, info, warn, error
- **示例**: LOG_LEVEL=debug

#### LOG_FORMAT
日志格式。

- **类型**: string
- **默认值**: text
- **可选值**: text, json
- **示例**: LOG_FORMAT=json

### 安全配置

#### ENABLE_CORS
是否启用 CORS。

- **类型**: boolean
- **默认值**: true
- **可选值**: true, false
- **示例**: ENABLE_CORS=true

#### ALLOWED_ORIGINS
允许的 CORS 源列表。

- **类型**: string
- **默认值**: *
- **示例**: ALLOWED_ORIGINS=https://example.com,https://app.example.com

#### MAX_FILE_SIZE_MB
最大文件大小(MB)。

- **类型**: number
- **默认值**: 100
- **示例**: MAX_FILE_SIZE_MB=200

#### ALLOWED_EXTENSIONS
允许的文件扩展名列表。

- **类型**: string
- **默认值**: pdf
- **示例**: ALLOWED_EXTENSIONS=pdf,docx

## 配置文件示例

### .env 示例文件

```bash
# 服务器配置
HOST=0.0.0.0
PORT=8000

# 存储配置
STORAGE_TYPE=local
STORAGE_LOCAL_DIR=/var/lib/pdf-module/data

# S3 配置示例
# STORAGE_TYPE=s3
# STORAGE_S3_BUCKET=my-pdf-bucket
# STORAGE_S3_REGION=us-east-1
# STORAGE_S3_ACCESS_KEY=your-access-key
# STORAGE_S3_SECRET_KEY=your-secret-key
# STORAGE_S3_PREFIX=pdf-files/

# GCS 配置示例
# STORAGE_TYPE=gcs
# STORAGE_GCS_BUCKET=my-pdf-bucket
# STORAGE_GCS_CREDENTIALS_PATH=/path/to/service-account.json

# Azure 配置示例
# STORAGE_TYPE=azure
# STORAGE_AZURE_ACCOUNT=mystorageaccount
# STORAGE_AZURE_KEY=your-storage-key
# STORAGE_AZURE_CONTAINER=pdf-files

# 缓存配置
CACHE_ENABLED=true
CACHE_MAX_SIZE_MB=100
CACHE_TTL_SECONDS=3600

# 审计配置
AUDIT_ENABLED=true
AUDIT_RETENTION_DAYS=30
AUDIT_LOG_DIR=/var/log/pdf-module/audit

# 日志配置
LOG_LEVEL=info
LOG_FORMAT=text

# 安全配置
ENABLE_CORS=true
ALLOWED_ORIGINS=*
MAX_FILE_SIZE_MB=100
ALLOWED_EXTENSIONS=pdf
```

## 配置验证

配置会在服务启动时自动验证。如果配置无效,服务将无法启动。

### 配置验证规则

1. **存储配置验证**:
   - 必须提供有效的存储类型
   - 云存储必须提供必需的凭证
   - 本地存储目录必须可写

2. **缓存配置验证**:
   - 最大缓存大小必须为正数
   - TTL 必须为正数

3. **安全配置验证**:
   - 文件大小限制必须为正数
   - 文件扩展名不能为空

4. **审计配置验证**:
   - 保留天数必须为正数
   - 审计日志目录必须可写

## 运行时配置

某些配置可以在运行时通过 API 修改:

### 清除缓存

```bash
curl -X POST http://localhost:8000/api/v1/x2text/cache/clear
```

### 更新配置

```bash
curl -X POST http://localhost:8000/api/v1/config/update \
  -H "Content-Type: application/json" \
  -d '{"CACHE_MAX_SIZE_MB": 200}'
```

## 配置最佳实践

### 生产环境配置建议

1. **存储配置**:
   - 使用云存储(如 S3)以提高可靠性
   - 配置适当的存储前缀以组织文件
   - 启用存储访问日志

2. **缓存配置**:
   - 根据内存大小调整缓存大小
   - 根据文件更新频率调整 TTL
   - 监控缓存命中率

3. **审计配置**:
   - 启用审计日志以追踪操作
   - 根据合规要求设置保留天数
   - 定期备份审计日志

4. **安全配置**:
   - 限制 CORS 允许的源
   - 设置合理的文件大小限制
   - 只允许必要的文件类型
   - 使用 HTTPS 部署

5. **日志配置**:
   - 生产环境使用 JSON 格式日志
   - 设置适当的日志级别(warn 或 error)
   - 配置日志轮转

### 开发环境配置建议

1. 使用本地存储便于调试
2. 启用详细日志(debug 级别)
3. 禁用缓存以加快开发迭代
4. 允许所有 CORS 源

## 故障排除

### 配置问题

1. **服务无法启动**:
   - 检查环境变量是否正确设置
   - 验证配置文件语法
   - 查看日志输出获取详细错误信息

2. **存储访问失败**:
   - 检查存储凭证是否正确
   - 验证存储权限
   - 确认网络连接正常

3. **缓存不生效**:
   - 确认缓存已启用
   - 检查缓存大小设置
   - 验证 TTL 配置

## 配置迁移

从旧版本迁移配置时,请注意以下变更:

1. **配置文件格式变更**:
   - 旧版本使用 `.env` 文件
   - 新版本继续支持 `.env` 文件
   - 某些配置项名称可能发生变化

2. **新增配置项**:
   - 新增了审计配置选项
   - 新增了安全配置选项
   - 新增了日志格式选项

3. **废弃配置项**:
   - 某些旧配置项已被废弃
   - 使用新配置项替代

## 配置监控

建议监控以下配置相关指标:

1. **缓存命中率**: 低于 80% 可能需要调整缓存策略
2. **存储使用量**: 接近容量限制时需要扩容
3. **审计日志大小**: 过大需要调整保留策略
4. **错误日志频率**: 异常增加可能表示配置问题

## 配置安全建议

1. **敏感信息保护**:
   - 不要将凭证提交到版本控制
   - 使用密钥管理服务存储敏感信息
   - 定期轮换访问密钥

2. **访问控制**:
   - 限制配置文件的访问权限
   - 使用最小权限原则
   - 定期审计配置访问

3. **配置备份**:
   - 定期备份配置文件
   - 版本控制非敏感配置
   - 文档化配置更改
