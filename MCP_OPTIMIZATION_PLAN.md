# pdf-module-rs MCP服务器优化方案

## 1. 方案概述

基于Unstract项目的企业级设计理念,为pdf-module-rs MCP服务器制定全面的优化方案。核心目标是打造一个高性能、可扩展、易集成的MCP服务器,为AI Agent提供强大的PDF处理能力。

### 1.1 当前现状分析

**现有优势**:
- ✅ 完整的MCP协议实现(stdio/SSE双传输)
- ✅ 多PDF引擎支持(lopdf/pdf-extract/pdfium)
- ✅ 缓存机制和性能监控
- ✅ Rust高性能实现
- ✅ 7个基础工具(文本提取、结构化提取、关键词搜索等)

**存在差距**:
- ❌ 缺乏流式处理和实时反馈机制
- ❌ 工具配置硬编码,缺乏动态配置能力
- ❌ 错误处理和验证机制不够完善
- ❌ 缺乏详细的审计日志和追踪
- ❌ 文件系统访问受限,仅支持本地文件
- ❌ 缺乏工具元数据管理

### 1.2 优化目标

1. **增强工具生态**: 借鉴Unstract的插件化设计,实现动态工具加载
2. **提升交互体验**: 引入流式处理和实时进度反馈
3. **强化安全合规**: 完善审计日志、权限控制和错误处理
4. **扩展文件访问**: 支持多种存储后端(S3、GCS等)
5. **优化性能监控**: 增强可观测性和成本追踪

## 2. 核心优化设计

### 2.1 工具协议标准化 (借鉴Unstract)

#### 2.1.1 工具定义协议

```rust
/// 工具定义配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具显示名称
    pub display_name: String,

    /// 工具唯一标识符(函数名)
    pub function_name: String,

    /// 工具描述
    pub description: String,

    /// 工具参数定义
    pub parameters: Vec<Parameter>,

    /// 工具版本历史
    pub versions: Vec<String>,

    /// 是否支持缓存
    pub is_cacheable: bool,

    /// 输入类型
    pub input_type: InputType,

    /// 输出类型
    pub output_type: OutputType,

    /// 工具需求
    pub requires: ToolRequirements,
}

/// 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
}

/// 输入类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    File,
    Database,
    Index,
}

/// 输出类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    File,
    Database,
    Index,
}

/// 工具需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequirements {
    pub files: ResourceRequirement,
    pub databases: ResourceRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirement {
    pub input: bool,
    pub output: bool,
}
```

#### 2.1.2 工具配置协议

```rust
/// 工具运行时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// 配置标题
    pub title: String,

    /// 配置描述
    pub description: String,

    /// 配置类型(固定为object)
    #[serde(rename = "type")]
    pub spec_type: String,

    /// 必需参数列表
    pub required: Vec<String>,

    /// 配置属性
    pub properties: std::collections::HashMap<String, Property>,
}

/// 属性定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "type")]
    pub property_type: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}
```

#### 2.1.3 运行时变量协议

```rust
/// 运行时环境变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVariables {
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub required: Vec<String>,
    pub properties: std::collections::HashMap<String, VariableProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableProperty {
    #[serde(rename = "type")]
    pub property_type: String,
    pub title: String,
    pub description: String,
}
```

### 2.2 消息协议增强

#### 2.2.1 统一消息格式

```rust
/// MCP工具消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ToolMessage {
    /// 工具规格消息
    Spec {
        spec: ToolSpec,
        emitted_at: DateTime<Utc>,
    },

    /// 工具属性消息
    Properties {
        properties: ToolDefinition,
        emitted_at: DateTime<Utc>,
    },

    /// 工具图标消息
    Icon {
        icon: String, // SVG内容
        emitted_at: DateTime<Utc>,
    },

    /// 运行时变量消息
    Variables {
        variables: RuntimeVariables,
        emitted_at: DateTime<Utc>,
    },

    /// 日志消息
    Log {
        level: LogLevel,
        log: String,
        emitted_at: DateTime<Utc>,
    },

    /// 成本追踪消息
    Cost {
        cost: f64,
        cost_units: String,
        emitted_at: DateTime<Utc>,
    },

    /// 结果消息
    Result {
        result: ToolExecutionResult,
        emitted_at: DateTime<Utc>,
    },

    /// 单步调试消息
    SingleStepMessage {
        message: String,
        emitted_at: DateTime<Utc>,
    },
}

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "uppercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub workflow_id: String,
    pub elapsed_time: u64, // 毫秒
    pub output: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ExecutionMetadata>,
}

/// 执行元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub file_name: String,
    pub file_size: u64,
    pub processing_time: u64,
    pub cache_hit: bool,
    pub adapter_used: String,
}
```

#### 2.2.2 流式处理支持

```rust
/// 流式消息发送器
pub trait MessageStreamer: Send + Sync {
    /// 发送消息
    async fn send(&self, message: ToolMessage) -> Result<()>;

    /// 发送日志
    async fn send_log(&self, level: LogLevel, message: String) -> Result<()>;

    /// 发送进度更新
    async fn send_progress(&self, progress: f64, message: String) -> Result<()>;

    /// 发送结果
    async fn send_result(&self, result: ToolExecutionResult) -> Result<()>;
}

/// Stdio消息流实现
pub struct StdioMessageStreamer {
    stdout: Arc<Mutex<std::io::Stdout>>,
}

impl MessageStreamer for StdioMessageStreamer {
    async fn send(&self, message: ToolMessage) -> Result<()> {
        let json = serde_json::to_string(&message)?;
        let mut stdout = self.stdout.lock().await;
        writeln!(stdout, "{}", json)?;
        stdout.flush()?;
        Ok(())
    }

    async fn send_log(&self, level: LogLevel, message: String) -> Result<()> {
        self.send(ToolMessage::Log {
            level,
            log: message,
            emitted_at: Utc::now(),
        }).await
    }

    async fn send_progress(&self, progress: f64, message: String) -> Result<()> {
        self.send(ToolMessage::Log {
            level: LogLevel::Info,
            log: format!("[{:.1}%] {}", progress * 100.0, message),
            emitted_at: Utc::now(),
        }).await
    }

    async fn send_result(&self, result: ToolExecutionResult) -> Result<()> {
        self.send(ToolMessage::Result {
            result,
            emitted_at: Utc::now(),
        }).await
    }
}

/// SSE消息流实现
pub struct SseMessageStreamer {
    sender: tokio::sync::mpsc::UnboundedSender<String>,
}

impl MessageStreamer for SseMessageStreamer {
    async fn send(&self, message: ToolMessage) -> Result<()> {
        let json = serde_json::to_string(&message)?;
        self.sender.send(json)?;
        Ok(())
    }

    async fn send_log(&self, level: LogLevel, message: String) -> Result<()> {
        self.send(ToolMessage::Log {
            level,
            log: message,
            emitted_at: Utc::now(),
        }).await
    }

    async fn send_progress(&self, progress: f64, message: String) -> Result<()> {
        self.send(ToolMessage::Log {
            level: LogLevel::Info,
            log: format!("[{:.1}%] {}", progress * 100.0, message),
            emitted_at: Utc::now(),
        }).await
    }

    async fn send_result(&self, result: ToolExecutionResult) -> Result<()> {
        self.send(ToolMessage::Result {
            result,
            emitted_at: Utc::now(),
        }).await
    }
}
```

### 2.3 审计与监控体系

#### 2.3.1 审计日志模型

```rust
/// PDF提取审计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionAudit {
    /// 审计ID
    pub id: Uuid,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 组织ID(多租户支持)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,

    /// 工作流ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,

    /// 文件执行ID
    pub file_execution_id: Uuid,

    /// 文件名
    pub file_name: String,

    /// 文件类型
    pub file_type: String,

    /// 文件大小(KB)
    pub file_size_kb: f64,

    /// 提取状态
    pub status: ExtractionStatus,

    /// 使用的适配器
    pub adapter_used: String,

    /// 处理时间(毫秒)
    pub processing_time_ms: u64,

    /// 是否命中缓存
    pub cache_hit: bool,

    /// 提取的文本长度
    pub extracted_text_length: usize,

    /// 错误信息(如果失败)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// 成本追踪(如果有)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,

    /// 成本单位
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_units: Option<String>,
}

/// 提取状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtractionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}
```

#### 2.3.2 审计日志服务

```rust
/// 审计日志服务
pub struct AuditService {
    // 可以支持多种后端: 文件、数据库、远程服务
    backend: AuditBackend,
}

/// 审计后端
pub enum AuditBackend {
    /// 文件后端
    File {
        log_dir: PathBuf,
    },
    /// 数据库后端
    Database {
        // 数据库连接配置
    },
    /// 远程服务后端
    Remote {
        endpoint: String,
        api_key: String,
    },
    /// 内存后端(用于测试)
    Memory,
}

impl AuditService {
    /// 创建新的审计服务
    pub fn new(backend: AuditBackend) -> Self {
        Self { backend }
    }

    /// 记录提取审计
    pub async fn log_extraction(&self, audit: ExtractionAudit) -> Result<()> {
        match &self.backend {
            AuditBackend::File { log_dir } => {
                self.log_to_file(log_dir, &audit).await
            }
            AuditBackend::Database { .. } => {
                self.log_to_database(&audit).await
            }
            AuditBackend::Remote { endpoint, api_key } => {
                self.log_to_remote(endpoint, api_key, &audit).await
            }
            AuditBackend::Memory => {
                // 内存存储,用于测试
                Ok(())
            }
        }
    }

    async fn log_to_file(&self, log_dir: &Path, audit: &ExtractionAudit) -> Result<()> {
        tokio::fs::create_dir_all(log_dir).await?;

        let date = audit.created_at.format("%Y-%m-%d").to_string();
        let filename = format!("audit_{}.jsonl", date);
        let filepath = log_dir.join(filename);

        let line = serde_json::to_string(audit)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filepath)
            .await?;

        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;

        Ok(())
    }

    async fn log_to_database(&self, audit: &ExtractionAudit) -> Result<()> {
        // TODO: 实现数据库存储
        Ok(())
    }

    async fn log_to_remote(&self, endpoint: &str, api_key: &str, audit: &ExtractionAudit) -> Result<()> {
        // TODO: 实现远程API调用
        Ok(())
    }

    /// 查询审计记录
    pub async fn query_audits(
        &self,
        filters: AuditFilters,
    ) -> Result<Vec<ExtractionAudit>> {
        match &self.backend {
            AuditBackend::File { log_dir } => {
                self.query_from_file(log_dir, filters).await
            }
            _ => {
                // TODO: 实现其他后端查询
                Ok(vec![])
            }
        }
    }

    async fn query_from_file(
        &self,
        log_dir: &Path,
        filters: AuditFilters,
    ) -> Result<Vec<ExtractionAudit>> {
        let mut audits = vec![];

        // 读取指定日期范围的日志文件
        if let Some(start_date) = filters.start_date {
            let end_date = filters.end_date.unwrap_or_else(|| Utc::now().naive_utc().date());
            let mut current_date = start_date;

            while current_date <= end_date {
                let filename = format!("audit_{}.jsonl", current_date.format("%Y-%m-%d"));
                let filepath = log_dir.join(filename);

                if filepath.exists() {
                    let content = tokio::fs::read_to_string(&filepath).await?;
                    for line in content.lines() {
                        if let Ok(audit) = serde_json::from_str::<ExtractionAudit>(line) {
                            if filters.matches(&audit) {
                                audits.push(audit);
                            }
                        }
                    }
                }

                current_date = current_date.succ_opt().unwrap();
            }
        }

        Ok(audits)
    }
}

/// 审计查询过滤器
#[derive(Debug, Clone)]
pub struct AuditFilters {
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub org_id: Option<String>,
    pub workflow_id: Option<String>,
    pub status: Option<ExtractionStatus>,
    pub file_name: Option<String>,
}

impl AuditFilters {
    pub fn matches(&self, audit: &ExtractionAudit) -> bool {
        if let Some(org_id) = &self.org_id {
            if audit.org_id.as_ref() != Some(org_id) {
                return false;
            }
        }

        if let Some(workflow_id) = &self.workflow_id {
            if audit.workflow_id.as_ref() != Some(workflow_id) {
                return false;
            }
        }

        if let Some(status) = &self.status {
            if &audit.status != status {
                return false;
            }
        }

        if let Some(file_name) = &self.file_name {
            if !audit.file_name.contains(file_name) {
                return false;
            }
        }

        true
    }
}
```

### 2.4 文件系统抽象

#### 2.4.1 统一文件存储接口

```rust
/// 文件存储抽象
#[async_trait]
pub trait FileStorage: Send + Sync {
    /// 读取文件
    async fn read(&self, path: &str) -> Result<Bytes>;

    /// 写入文件
    async fn write(&self, path: &str, data: &[u8]) -> Result<()>;

    /// 检查文件是否存在
    async fn exists(&self, path: &str) -> Result<bool>;

    /// 删除文件
    async fn delete(&self, path: &str) -> Result<()>;

    /// 列出文件
    async fn list(&self, path: &str, recursive: bool) -> Result<Vec<FileInfo>>;

    /// 获取文件元数据
    async fn metadata(&self, path: &str) -> Result<FileMetadata>;

    /// 获取存储类型
    fn storage_type(&self) -> StorageType;
}

/// 存储类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    Local,
    S3,
    GCS,
    AzureBlob,
    MinIO,
    Http,
}

/// 文件元数据
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub content_type: Option<String>,
}

/// 本地文件系统实现
pub struct LocalFileStorage {
    base_dir: PathBuf,
}

#[async_trait]
impl FileStorage for LocalFileStorage {
    async fn read(&self, path: &str) -> Result<Bytes> {
        let full_path = self.base_dir.join(path);
        let data = tokio::fs::read(&full_path).await?;
        Ok(Bytes::from(data))
    }

    async fn write(&self, path: &str, data: &[u8]) -> Result<()> {
        let full_path = self.base_dir.join(path);
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&full_path, data).await?;
        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.base_dir.join(path);
        Ok(full_path.exists())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.base_dir.join(path);
        tokio::fs::remove_file(&full_path).await?;
        Ok(())
    }

    async fn list(&self, path: &str, recursive: bool) -> Result<Vec<FileInfo>> {
        let full_path = self.base_dir.join(path);
        let mut files = vec![];

        if recursive {
            let mut entries = WalkDir::new(&full_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            while let Some(entry) = entries.next() {
                let metadata = entry.metadata()?;
                files.push(FileInfo {
                    path: entry.path().to_string_lossy().to_string(),
                    size: metadata.len(),
                });
            }
        } else {
            let mut entries = tokio::fs::read_dir(&full_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    let metadata = entry.metadata().await?;
                    files.push(FileInfo {
                        path: path.to_string_lossy().to_string(),
                        size: metadata.len(),
                    });
                }
            }
        }

        Ok(files)
    }

    async fn metadata(&self, path: &str) -> Result<FileMetadata> {
        let full_path = self.base_dir.join(path);
        let metadata = tokio::fs::metadata(&full_path).await?;
        let modified: DateTime<Utc> = metadata.modified()?.into();

        Ok(FileMetadata {
            path: path.to_string(),
            size: metadata.len(),
            modified,
            content_type: infer::get_from_path(&full_path)
                .ok()
                .and_then(|t| t.mime_type().map(String::from)),
        })
    }

    fn storage_type(&self) -> StorageType {
        StorageType::Local
    }
}

/// S3文件存储实现
pub struct S3FileStorage {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: Option<String>,
}

#[async_trait]
impl FileStorage for S3FileStorage {
    async fn read(&self, path: &str) -> Result<Bytes> {
        let key = self.resolve_key(path);
        let resp = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await?;

        let data = resp.body.collect().await?.into_bytes();
        Ok(data)
    }

    async fn write(&self, path: &str, data: &[u8]) -> Result<()> {
        let key = self.resolve_key(path);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(Bytes::from(data.to_vec())))
            .send()
            .await?;
        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let key = self.resolve_key(path);
        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.is_service_error()
                    && e.as_service_error()
                    .map(|s| s.kind() == aws_sdk_s3::error::HeadObjectErrorKind::NotFound)
                    .unwrap_or(false)
                {
                    Ok(false)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let key = self.resolve_key(path);
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await?;
        Ok(())
    }

    async fn list(&self, path: &str, recursive: bool) -> Result<Vec<FileInfo>> {
        let prefix = self.resolve_prefix(path);
        let mut files = vec![];

        let mut continuation_token: Option<String> = None;
        loop {
            let mut request = self.client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&prefix);

            if !recursive {
                request = request.delimiter("/");
            }

            if let Some(token) = continuation_token {
                request = request.continuation_token(&token);
            }

            let response = request.send().await?;

            if let Some(contents) = response.contents {
                for obj in contents {
                    if let Some(key) = obj.key() {
                        if let Some(size) = obj.size() {
                            files.push(FileInfo {
                                path: key.clone(),
                                size: size as u64,
                            });
                        }
                    }
                }
            }

            if response.next_continuation_token().is_none() {
                break;
            }
            continuation_token = response.next_continuation_token().map(String::from);
        }

        Ok(files)
    }

    async fn metadata(&self, path: &str) -> Result<FileMetadata> {
        let key = self.resolve_key(path);
        let resp = self.client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await?;

        Ok(FileMetadata {
            path: path.to_string(),
            size: resp.content_length().unwrap_or(0) as u64,
            modified: resp.last_modified().unwrap().into(),
            content_type: resp.content_type().map(String::from),
        })
    }

    fn storage_type(&self) -> StorageType {
        StorageType::S3
    }
}

impl S3FileStorage {
    fn resolve_key(&self, path: &str) -> String {
        match &self.prefix {
            Some(prefix) => format!("{}/{}", prefix.trim_end_matches('/'), path.trim_start_matches('/')),
            None => path.trim_start_matches('/').to_string(),
        }
    }

    fn resolve_prefix(&self, path: &str) -> String {
        format!("{}/{}", self.resolve_key(path), if path.ends_with('/') { "" } else { "/" })
    }
}
```

#### 2.4.2 文件存储工厂

```rust
/// 文件存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageConfig {
    pub storage_type: StorageType,
    pub local: Option<LocalStorageConfig>,
    pub s3: Option<S3StorageConfig>,
    pub gcs: Option<GCSStorageConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    pub base_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3StorageConfig {
    pub bucket: String,
    pub region: String,
    pub prefix: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub endpoint: Option<String>,
}

/// 文件存储工厂
pub struct FileStorageFactory;

impl FileStorageFactory {
    /// 根据配置创建文件存储
    pub async fn create(config: FileStorageConfig) -> Result<Arc<dyn FileStorage>> {
        match config.storage_type {
            StorageType::Local => {
                let local_config = config.local.ok_or_else(|| {
                    anyhow::anyhow!("Local storage config is required")
                })?;

                let storage = LocalFileStorage {
                    base_dir: PathBuf::from(&local_config.base_dir),
                };

                // 确保基础目录存在
                tokio::fs::create_dir_all(&storage.base_dir).await?;

                Ok(Arc::new(storage))
            }

            StorageType::S3 => {
                let s3_config = config.s3.ok_or_else(|| {
                    anyhow::anyhow!("S3 storage config is required")
                })?;

                let mut config_loader = aws_config::defaults(aws_config::Behavior::latest());

                if let (Some(access_key), Some(secret_key)) = (&s3_config.access_key, &s3_config.secret_key) {
                    config_loader = config_loader.credentials_provider(
                        aws_sdk_s3::config::Credentials::new(
                            access_key,
                            secret_key,
                            None,
                            None,
                            "static",
                        )
                    );
                }

                let sdk_config = config_loader.load().await;

                let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&sdk_config)
                    .region(aws_sdk_s3::config::Region::new(s3_config.region.clone()));

                if let Some(endpoint) = &s3_config.endpoint {
                    s3_config_builder = s3_config_builder.endpoint_url(endpoint);
                }

                let s3_config = s3_config_builder.build();
                let client = aws_sdk_s3::Client::from_conf(s3_config);

                let storage = S3FileStorage {
                    client,
                    bucket: s3_config.bucket,
                    prefix: s3_config.prefix,
                };

                Ok(Arc::new(storage))
            }

            _ => {
                anyhow::bail!("Storage type {:?} is not yet implemented", config.storage_type)
            }
        }
    }
}
```

### 2.5 工具插件化架构

#### 2.5.1 工具注册表

```rust
/// 工具注册表
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, RegisteredTool>>,
}

/// 已注册的工具
pub struct RegisteredTool {
    pub definition: ToolDefinition,
    pub spec: ToolSpec,
    pub runtime_variables: RuntimeVariables,
    pub handler: Arc<dyn ToolHandler>,
}

/// 工具处理器trait
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// 验证输入
    async fn validate(&self, input: &ToolInput) -> Result<ValidationResult>;

    /// 执行工具
    async fn execute(
        &self,
        input: ToolInput,
        context: ToolContext,
    ) -> Result<ToolOutput>;

    /// 获取工具图标
    fn icon(&self) -> Option<String>;
}

/// 工具输入
pub struct ToolInput {
    pub arguments: serde_json::Value,
    pub file_path: Option<String>,
    pub output_dir: Option<String>,
}

/// 工具上下文
pub struct ToolContext {
    pub workflow_id: String,
    pub file_execution_id: Uuid,
    pub storage: Arc<dyn FileStorage>,
    pub message_streamer: Arc<dyn MessageStreamer>,
    pub audit_service: Arc<AuditService>,
    pub cache: Arc<ExtractionCache>,
}

/// 工具输出
pub struct ToolOutput {
    pub result: serde_json::Value,
    pub metadata: ExecutionMetadata,
}

/// 验证结果
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    /// 注册工具
    pub async fn register(&self, tool: RegisteredTool) -> Result<()> {
        let function_name = tool.definition.function_name.clone();
        let mut tools = self.tools.write().await;
        tools.insert(function_name, tool);
        Ok(())
    }

    /// 获取工具
    pub async fn get(&self, name: &str) -> Option<RegisteredTool> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    /// 列出所有工具
    pub async fn list(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        tools.values().map(|t| t.definition.clone()).collect()
    }

    /// 执行工具
    pub async fn execute(
        &self,
        tool_name: &str,
        input: ToolInput,
        context: ToolContext,
    ) -> Result<ToolOutput> {
        let tool = self.get(tool_name).await
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

        // 验证输入
        let validation = tool.handler.validate(&input).await?;
        if !validation.is_valid {
            let error_msg = validation.errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect::<Vec<_>>()
                .join("; ");
            anyhow::bail!("Input validation failed: {}", error_msg);
        }

        // 发送警告
        for warning in validation.warnings {
            context.message_streamer.send_log(
                LogLevel::Warn,
                format!("Validation warning: {}: {}", warning.field, warning.message)
            ).await?;
        }

        // 执行工具
        let output = tool.handler.execute(input, context).await?;

        Ok(output)
    }
}
```

#### 2.5.2 基础工具实现

```rust
/// PDF文本提取工具
pub struct ExtractTextTool {
    service: Arc<PdfExtractorService>,
}

#[async_trait]
impl ToolHandler for ExtractTextTool {
    async fn validate(&self, input: &ToolInput) -> Result<ValidationResult> {
        let mut errors = vec![];
        let mut warnings = vec![];

        // 验证文件路径
        let file_path = input.arguments.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ValidationError {
                field: "file_path".to_string(),
                message: "file_path is required".to_string(),
            })?;

        if file_path.is_empty() {
            errors.push(ValidationError {
                field: "file_path".to_string(),
                message: "file_path cannot be empty".to_string(),
            });
        }

        // 验证适配器
        if let Some(adapter) = input.arguments.get("adapter").and_then(|v| v.as_str()) {
            let valid_adapters = vec!["lopdf", "pdf-extract", "pdfium"];
            if !valid_adapters.contains(&adapter) {
                errors.push(ValidationError {
                    field: "adapter".to_string(),
                    message: format!("Invalid adapter: {}. Must be one of: {}", adapter, valid_adapters.join(", ")),
                });
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    async fn execute(
        &self,
        input: ToolInput,
        context: ToolContext,
    ) -> Result<ToolOutput> {
        let start_time = std::time::Instant::now();

        let file_path_str = input.arguments["file_path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;

        let adapter = input.arguments.get("adapter")
            .and_then(|v| v.as_str());

        // 发送开始日志
        context.message_streamer.send_log(
            LogLevel::Info,
            format!("Starting text extraction for file: {}", file_path_str)
        ).await?;

        // 执行提取
        let result = self.service
            .extract_text(Path::new(file_path_str), adapter)
            .await?;

        // 发送进度更新
        context.message_streamer.send_progress(1.0, "Extraction completed".to_string()).await?;

        // 记录审计
        let audit = ExtractionAudit {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            org_id: None,
            workflow_id: Some(context.workflow_id.clone()),
            file_execution_id: context.file_execution_id,
            file_name: Path::new(file_path_str)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            file_type: "pdf".to_string(),
            file_size_kb: 0.0, // TODO: 获取文件大小
            status: ExtractionStatus::Completed,
            adapter_used: adapter.unwrap_or("default").to_string(),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            cache_hit: false, // TODO: 从结果中获取
            extracted_text_length: result.extracted_text.len(),
            error_message: None,
            cost: None,
            cost_units: None,
        };

        context.audit_service.log_extraction(audit).await?;

        // 返回结果
        Ok(ToolOutput {
            result: serde_json::json!({
                "extracted_text": result.extracted_text,
                "metadata": result.extraction_metadata
            }),
            metadata: ExecutionMetadata {
                file_name: file_path_str.to_string(),
                file_size: 0,
                processing_time: start_time.elapsed().as_millis() as u64,
                cache_hit: false,
                adapter_used: adapter.unwrap_or("default").to_string(),
            },
        })
    }

    fn icon(&self) -> Option<String> {
        Some(include_str!("../assets/icons/extract_text.svg").to_string())
    }
}
```

### 2.6 配置管理增强

#### 2.6.1 分层配置系统

```rust
/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 基础配置
    pub base: BaseConfig,

    /// 存储配置
    pub storage: FileStorageConfig,

    /// 缓存配置
    pub cache: CacheConfig,

    /// 审计配置
    pub audit: AuditConfig,

    /// 日志配置
    pub logging: LoggingConfig,

    /// 安全配置
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseConfig {
    pub server_name: String,
    pub server_version: String,
    pub environment: Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_size: usize,
    pub ttl_seconds: u64,
    pub cache_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub backend: AuditBackendConfig,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuditBackendConfig {
    File { log_dir: String },
    Database {
        connection_string: String,
        table_name: String,
    },
    Remote {
        endpoint: String,
        api_key: String,
    },
    Memory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub outputs: Vec<LogOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LogOutput {
    Stdout,
    File { path: String },
    Syslog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub path_validation: PathValidationConfig,
    pub max_file_size_mb: u64,
    pub allowed_file_types: Vec<String>,
}

impl ServerConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let environment = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .parse()?;

        Ok(Self {
            base: BaseConfig {
                server_name: std::env::var("SERVER_NAME")
                    .unwrap_or_else(|_| "pdf-module-mcp".to_string()),
                server_version: std::env::var("SERVER_VERSION")
                    .unwrap_or_else(|_| "0.1.0".to_string()),
                environment,
            },
            storage: FileStorageConfig {
                storage_type: std::env::var("STORAGE_TYPE")
                    .unwrap_or_else(|_| "local".to_string())
                    .parse()?,
                local: Some(LocalStorageConfig {
                    base_dir: std::env::var("LOCAL_STORAGE_BASE_DIR")
                        .unwrap_or_else(|_| "./data".to_string()),
                }),
                s3: None,
                gcs: None,
            },
            cache: CacheConfig {
                enabled: std::env::var("CACHE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                max_size: std::env::var("CACHE_MAX_SIZE")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()?,
                ttl_seconds: std::env::var("CACHE_TTL_SECONDS")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()?,
                cache_dir: std::env::var("CACHE_DIR").ok(),
            },
            audit: AuditConfig {
                enabled: std::env::var("AUDIT_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                backend: AuditBackendConfig::File {
                    log_dir: std::env::var("AUDIT_LOG_DIR")
                        .unwrap_or_else(|_| "./logs/audit".to_string()),
                },
                retention_days: std::env::var("AUDIT_RETENTION_DAYS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
            },
            logging: LoggingConfig {
                level: std::env::var("LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
                format: std::env::var("LOG_FORMAT")
                    .unwrap_or_else(|_| "json".to_string())
                    .parse()?,
                outputs: vec![LogOutput::Stdout],
            },
            security: SecurityConfig {
                path_validation: PathValidationConfig {
                    require_absolute: std::env::var("PATH_REQUIRE_ABSOLUTE")
                        .unwrap_or_else(|_| "true".to_string())
                        .parse()?,
                    allow_traversal: std::env::var("PATH_ALLOW_TRAVERSAL")
                        .unwrap_or_else(|_| "false".to_string())
                        .parse()?,
                    base_dir: std::env::var("PATH_BASE_DIR").ok(),
                },
                max_file_size_mb: std::env::var("MAX_FILE_SIZE_MB")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                allowed_file_types: vec!["pdf".to_string()],
            },
        })
    }

    /// 初始化日志系统
    pub fn init_tracing(&self) {
        let level = match self.logging.level.to_lowercase().as_str() {
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        };

        let subscriber = tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(true)
            .with_thread_ids(true);

        match self.logging.format {
            LogFormat::Json => {
                subscriber.json().init();
            }
            LogFormat::Text => {
                subscriber.pretty().init();
            }
        }
    }
}
```

## 3. 实施计划

### 3.1 第一阶段: 核心协议增强 (2-3周)

**目标**: 实现标准化的工具协议和消息流

**任务**:
1. ✅ 设计并实现工具定义协议(ToolDefinition、ToolSpec、RuntimeVariables)
2. ✅ 实现统一消息协议(ToolMessage枚举)
3. ✅ 开发流式消息发送器(MessageStreamer trait及实现)
4. ✅ 重构现有工具以支持新的协议
5. ✅ 添加工具配置验证

**交付物**:
- `crates/pdf-mcp/src/protocol/` 模块
- `crates/pdf-mcp/src/streamer/` 模块
- 更新的MCP服务器实现

### 3.2 第二阶段: 审计与监控 (2-3周)

**目标**: 建立完整的审计日志和监控体系

**任务**:
1. ✅ 设计审计数据模型(ExtractionAudit)
2. ✅ 实现审计服务(AuditService)
3. ✅ 支持多种审计后端(文件、数据库、远程)
4. ✅ 集成审计日志到现有工具
5. ✅ 实现审计查询功能
6. ✅ 增强Prometheus指标

**交付物**:
- `crates/pdf-core/src/audit/` 模块
- 审计日志查询API
- 增强的监控指标

### 3.3 第三阶段: 文件系统抽象 (3-4周)

**目标**: 实现统一的文件存储接口,支持多种存储后端

**任务**:
1. ✅ 设计文件存储抽象(FileStorage trait)
2. ✅ 实现本地文件系统存储(LocalFileStorage)
3. ✅ 实现S3存储(S3FileStorage)
4. ✅ 实现文件存储工厂(FileStorageFactory)
5. ✅ 集成文件存储到现有工具
6. ✅ 添加存储配置管理

**交付物**:
- `crates/pdf-core/src/storage/` 模块
- 多存储后端支持
- 存储配置文档

### 3.4 第四阶段: 工具插件化 (3-4周)

**目标**: 实现动态工具加载和插件化架构

**任务**:
1. ✅ 设计工具注册表(ToolRegistry)
2. ✅ 实现工具处理器trait(ToolHandler)
3. ✅ 重构现有工具为插件
4. ✅ 实现工具动态加载
5. ✅ 添加工具元数据管理
6. ✅ 实现工具生命周期管理

**交付物**:
- `crates/pdf-mcp/src/registry/` 模块
- 插件化的工具实现
- 工具开发文档

### 3.5 第五阶段: 配置管理优化 (1-2周)

**目标**: 建立分层配置系统,提升配置管理能力

**任务**:
1. ✅ 设计分层配置结构(ServerConfig)
2. ✅ 实现配置加载和验证
3. ✅ 支持配置热更新
4. ✅ 添加配置文档和示例

**交付物**:
- 更新的配置系统
- 配置文档和示例
- 配置验证工具

## 4. 预期收益

### 4.1 功能提升
- ✅ 支持7+种存储后端,扩展文件访问能力
- ✅ 完整的审计日志,满足合规要求
- ✅ 流式处理和实时反馈,提升用户体验
- ✅ 插件化架构,支持工具动态扩展
- ✅ 标准化协议,便于集成和维护

### 4.2 性能优化
- ✅ 多存储后端支持,优化大文件处理
- ✅ 增强的缓存策略,提升重复处理性能
- ✅ 详细的性能监控,便于性能优化

### 4.3 可维护性
- ✅ 清晰的模块划分,降低耦合度
- ✅ 标准化的接口定义,便于扩展
- ✅ 完善的文档和示例,降低学习成本

### 4.4 企业级特性
- ✅ 审计日志和追踪,满足合规要求
- ✅ 多租户支持,便于SaaS部署
- ✅ 安全增强,提升系统安全性
- ✅ 监控和告警,便于运维管理

## 5. 风险评估与缓解

### 5.1 技术风险

**风险**: 新架构可能引入性能瓶颈
**缓解**:
- 进行性能基准测试
- 采用渐进式重构,保持向后兼容
- 提供性能优化指南

**风险**: 存储后端集成复杂性
**缓解**:
- 优先实现核心存储后端(Local、S3)
- 提供详细的集成文档和示例
- 建立存储后端测试套件

### 5.2 实施风险

**风险**: 开发周期较长
**缓解**:
- 分阶段实施,每个阶段都有明确的交付物
- 采用敏捷开发,及时调整计划
- 保持现有功能稳定,逐步迁移

**风险**: 向后兼容性问题
**缓解**:
- 保持现有API不变
- 提供迁移指南和工具
- 充分测试兼容性

## 6. 总结

本优化方案借鉴了Unstract项目的企业级设计理念,为pdf-module-rs MCP服务器制定了全面的优化计划。通过实施工具协议标准化、消息流增强、审计监控、文件系统抽象、插件化架构和配置管理优化,将显著提升MCP服务器的功能性、可扩展性和企业级特性。

方案采用分阶段实施策略,每个阶段都有明确的目标和交付物,确保项目稳步推进。预期通过本方案的实施,将pdf-module-rs打造成为一个高性能、可扩展、易集成的企业级MCP服务器。

🎯