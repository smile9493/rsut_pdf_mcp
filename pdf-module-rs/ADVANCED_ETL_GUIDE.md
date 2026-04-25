# PDF 高级 ETL 流水线文档

## 概述

PDF 高级 ETL 流水线是一个高性能的 PDF 结构化提取与自动入库解决方案,采用四层架构设计:

1. **物理解析层 (I/O & Parsing)**: 利用 pdfium/lopdf 提取原生文本
2. **版面感知层 (Layout Analysis)**: 对复杂文档进行区域检测(ROI)
3. **语义映射层 (Semantic Mapping)**: 利用 LLM 进行结构化转换
4. **持久化层 (Persistence)**: 通过 SQLx 事务写入数据库

## 核心特性

### 1. 智能路径选择

系统根据文档特征自动选择最优处理路径:

- **Fast Path**: 原生 PDF 文本提取,适用于文本密度高的文档
- **Vision Path**: 多模态模型识别,适用于扫描件和复杂表格
- **Hybrid Path**: 结合文本和视觉,适用于混合型文档

### 2. 声明式 Schema 定义

使用 `schemars` 实现 Rust 结构体与 JSON Schema 的强一致:

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct ContractExtraction {
    /// 合同甲方全称
    pub party_a: String,
    /// 合同总金额(元)
    pub total_amount: f64,
    /// 签署日期 (YYYY-MM-DD)
    pub sign_date: String,
    /// 核心条款摘要
    pub key_clauses: Vec<String>,
}
```

### 3. 闭环验证机制

自动验证提取结果的准确性:

- 关键字段非空检查
- 金额字段原文匹配验证
- 日期格式规范检查
- 自动重试和人工复核标记

### 4. 隐私脱敏

自动识别并脱敏敏感信息:

- 身份证号
- 手机号码
- 银行卡号

### 5. 版面分析

智能识别文档结构:

- 标题、正文、表格、列表
- 页眉、页脚
- 图片区域

## 快速开始

### 1. 环境准备

```bash
# 克隆项目
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp/pdf-module-rs

# 复制环境变量配置
cp .env.advanced.example .env

# 编辑 .env 文件,设置必要的配置
vim .env
```

### 2. 启动服务

```bash
# 启动完整服务栈
docker-compose -f docker-compose.advanced.yml up -d

# 查看日志
docker-compose -f docker-compose.advanced.yml logs -f pdf-advanced-etl

# 仅启动核心服务(不含监控)
docker-compose -f docker-compose.advanced.yml up -d pdf-advanced-etl postgres redis
```

### 3. 使用示例

#### Rust 代码示例

```rust
use pdf_etl::{
    config::{DatabaseConfig, LLMConfig, ExtractionConfig},
    pipeline::{AdvancedETLPipeline, AdvancedPipelineConfig, SchemaDefinition},
    schema::InvoiceSchema,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 配置
    let llm_config = LLMConfig {
        provider: LLMProvider::OpenAI,
        model: "gpt-4o".to_string(),
        api_key: std::env::var("OPENAI_API_KEY")?,
        // ... 其他配置
    };

    let db_config = DatabaseConfig {
        db_type: DatabaseType::Postgres,
        connection_string: "postgresql://localhost/pdf_etl".to_string(),
        // ... 其他配置
    };

    // 创建流水线
    let pipeline = AdvancedETLPipeline::new(
        ExtractionConfig::default(),
        llm_config,
        Some(db_config),
        Some(AdvancedPipelineConfig::default()),
    ).await?;

    // 执行提取
    let schema = SchemaDefinition::from_struct::<InvoiceSchema>();
    let result = pipeline.execute(
        "./invoice.pdf",
        schema,
        Some("请提取发票信息"),
        true, // 保存到数据库
    ).await?;

    println!("提取结果: {:?}", result.transform.data);
    Ok(())
}
```

#### REST API 调用

```bash
# 提取并保存
curl -X POST http://localhost:8000/api/v1/etl/extract \
  -H "Content-Type: multipart/form-data" \
  -F "file=@document.pdf" \
  -F "schema_name=InvoiceSchema" \
  -F "save_to_db=true"

# 查询提取结果
curl http://localhost:8000/api/v1/etl/query \
  -H "Content-Type: application/json" \
  -d '{
    "schema_name": "InvoiceSchema",
    "filters": {"vendor.name": "某公司"},
    "limit": 10
  }'
```

#### MCP 工具调用

在 Cursor IDE 或 Claude Desktop 中配置:

```json
{
  "mcpServers": {
    "pdf-advanced-etl": {
      "url": "http://localhost:8001/sse"
    }
  }
}
```

然后可以直接调用工具:

```json
{
  "name": "extract_and_save",
  "arguments": {
    "file_path": "/path/to/invoice.pdf",
    "schema_name": "InvoiceSchema",
    "save_to_db": true
  }
}
```

## 配置说明

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `LLM_PROVIDER` | LLM 提供商 | `openai` |
| `LLM_API_KEY` | API Key | (必填) |
| `LLM_MODEL` | 模型名称 | `gpt-4o` |
| `DB_TYPE` | 数据库类型 | `postgres` |
| `DB_CONNECTION_STRING` | 数据库连接串 | (必填) |
| `ENABLE_LAYOUT_ANALYSIS` | 启用版面分析 | `true` |
| `ENABLE_VALIDATION` | 启用验证 | `true` |
| `VALIDATION_THRESHOLD` | 验证阈值 | `0.8` |
| `ENABLE_PRIVACY_MASK` | 启用隐私脱敏 | `true` |
| `AUTO_PATH_SELECTION` | 自动路径选择 | `true` |

### 高级配置

```rust
let advanced_config = AdvancedPipelineConfig {
    enable_layout_analysis: true,    // 版面分析
    enable_validation: true,         // 闭环验证
    validation_threshold: 0.8,       // 验证阈值
    enable_privacy_mask: true,       // 隐私脱敏
    max_retries: 3,                  // 最大重试次数
    auto_path_selection: true,       // 自动路径选择
};
```

## 数据库 Schema

### 主要表结构

#### extracted_documents

存储提取的结构化数据:

```sql
CREATE TABLE extracted_documents (
    id UUID PRIMARY KEY,
    schema_name VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    source_file VARCHAR(512),
    extraction_metadata JSONB,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);
```

#### processing_tasks

跟踪处理任务状态:

```sql
CREATE TABLE processing_tasks (
    id UUID PRIMARY KEY,
    file_path VARCHAR(512) NOT NULL,
    schema_name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,  -- pending, processing, completed, failed
    processing_path VARCHAR(50),  -- fast_path, vision_path, hybrid_path
    result_id UUID,
    error_message TEXT,
    retry_count INTEGER,
    created_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);
```

#### validation_records

存储验证结果:

```sql
CREATE TABLE validation_records (
    id UUID PRIMARY KEY,
    document_id UUID,
    is_valid BOOLEAN NOT NULL,
    score DECIMAL(5,4) NOT NULL,
    errors JSONB,
    warnings JSONB,
    needs_review BOOLEAN DEFAULT FALSE,
    validated_at TIMESTAMPTZ
);
```

## 性能优化

### 1. 数据库优化

```sql
-- 创建 GIN 索引加速 JSONB 查询
CREATE INDEX idx_data_gin ON extracted_documents USING GIN(data);

-- 创建全文搜索索引
CREATE INDEX idx_schema_trgm ON extracted_documents 
  USING GIN(schema_name gin_trgm_ops);
```

### 2. 缓存策略

- Redis 缓存提取结果
- LLM 响应缓存
- Schema 定义缓存

### 3. 并发处理

```rust
// 批量处理
let results = pipeline.execute_batch(
    &pdf_paths,
    schema,
    prompt_template,
    save_to_db,
).await?;
```

## 监控与可观测性

### Prometheus 指标

- `pdf_etl_extraction_total`: 提取总数
- `pdf_etl_extraction_duration_seconds`: 提取耗时
- `pdf_etl_llm_tokens_total`: Token 使用量
- `pdf_etl_validation_score`: 验证分数

### Grafana 仪表板

访问 `http://localhost:3000` 查看可视化监控面板。

## 故障排查

### 常见问题

1. **数据库连接失败**
   ```bash
   # 检查数据库状态
   docker-compose -f docker-compose.advanced.yml ps postgres
   
   # 查看日志
   docker-compose -f docker-compose.advanced.yml logs postgres
   ```

2. **LLM API 调用失败**
   - 检查 API Key 是否正确
   - 检查网络连接
   - 查看速率限制

3. **提取结果不准确**
   - 调整 `validation_threshold`
   - 启用 Vision Path
   - 检查 Schema 定义

## 最佳实践

1. **Schema 设计**
   - 使用清晰的字段描述
   - 合理设置必填/可选字段
   - 利用嵌套结构组织复杂数据

2. **提示词优化**
   - 明确提取目标
   - 提供示例格式
   - 指定输出要求

3. **批量处理**
   - 合理设置并发数
   - 使用任务队列
   - 监控处理进度

4. **数据验证**
   - 启用闭环验证
   - 设置合理的阈值
   - 人工复核标记记录

## 扩展开发

### 添加自定义 Schema

```rust
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct CustomSchema {
    pub field1: String,
    pub field2: i32,
    pub nested: NestedSchema,
}

let schema = SchemaDefinition::from_struct::<CustomSchema>();
```

### 自定义处理逻辑

```rust
impl AdvancedETLPipeline {
    async fn custom_process(&self, pdf_path: &str) -> Result<()> {
        // 自定义处理逻辑
    }
}
```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request!
