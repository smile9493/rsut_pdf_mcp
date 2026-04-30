# PDF模块改进方案设计文档

## 项目现状分析

### 当前架构
```
pdf-module-rs/
├── crates/
│   ├── pdf-core/          # 核心PDF处理引擎
│   │   ├── engine/        # Pdfium引擎实现
│   │   ├── extractor.rs   # 提取器
│   │   └── vlm_pipeline.rs # VLM增强管道
│   ├── pdf-mcp/           # MCP服务器实现
│   ├── vlm-visual-gateway/ # VLM视觉网关
│   ├── pdf-common/        # 共享类型
│   └── pdf-dashboard/     # 仪表板
```

### 当前技术栈
- **PDF引擎**: pdfium-render 0.8 (sync模式)
- **异步运行时**: Tokio
- **VLM集成**: 已实现GLM-4.6V系列模型支持
- **并发控制**: Semaphore限流
- **进度追踪**: 基础统计 (ToolStats)

---

## 一、短期改进方案 (1-3个月)

### 1.1 并行处理优化 - Rayon集成

#### 设计目标
- 实现批量PDF文件的并行处理
- 页面级别的并行提取
- 保持内存效率

#### 技术方案

**1. 添加Rayon依赖**
```toml
# Cargo.toml
[workspace.dependencies]
rayon = "1.10"
num_cpus = "1.16"
```

**2. 批量文件并行处理架构**

```rust
// crates/pdf-core/src/parallel/batch_processor.rs
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

pub struct BatchProcessor {
    pipeline: Arc<McpPdfPipeline>,
    config: BatchConfig,
}

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_files_parallel: usize,
    pub max_pages_parallel: usize,
    pub chunk_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_files_parallel: num_cpus::get(),
            max_pages_parallel: 4,
            chunk_size: 10,
        }
    }
}

impl BatchProcessor {
    pub fn new(pipeline: Arc<McpPdfPipeline>, config: BatchConfig) -> Self {
        Self { pipeline, config }
    }

    pub fn process_batch(
        &self,
        files: &[PathBuf],
        options: &ExtractOptions,
    ) -> Vec<(PathBuf, Result<StructuredExtractionResult, PdfModuleError>)> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.config.max_files_parallel)
            .thread_name(|i| format!("pdf-batch-{}", i))
            .build()
            .unwrap();

        pool.install(|| {
            files
                .par_iter()
                .map(|file_path| {
                    let rt = tokio::runtime::Handle::current();
                    let result = rt.block_on(self.pipeline.extract_structured(file_path, options));
                    (file_path.clone(), result)
                })
                .collect()
        })
    }
}
```

**3. 页面级并行提取**

```rust
// crates/pdf-core/src/engine/parallel_pdfium.rs
impl PdfiumEngine {
    pub fn extract_structured_parallel(
        data: &[u8],
        page_range: Option<(u32, u32)>,
        config: &ParallelConfig,
    ) -> PdfResult<StructuredExtractionResult> {
        let pdfium = Self::get_pdfium()?;
        let document = pdfium.load_pdf_from_byte_slice(data, None)?;
        let pages = document.pages();
        let total_pages = pages.len() as u32;

        let (start, end) = page_range.unwrap_or((0, total_pages));
        let page_indices: Vec<u32> = (start..end).collect();

        let page_metas: Vec<PageMetadata> = page_indices
            .par_iter()
            .map(|&i| {
                let page = pages.get(i as u16)?;
                let text = page.text()?;
                let text_str = text.all();

                Ok(PageMetadata {
                    page_number: i + 1,
                    text: text_str.trim().to_string(),
                    bbox: Some((0.0, 0.0, page.width().value as f64, page.height().value as f64)),
                    lines: vec![],
                })
            })
            .collect::<Result<Vec<_>, PdfiumError>>()
            .map_err(|e| PdfModuleError::Extraction(e.to_string()))?;

        let extracted_text = page_metas.iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(StructuredExtractionResult {
            extracted_text,
            page_count: total_pages,
            pages: page_metas,
            extraction_metadata: None,
            file_info: FileInfo::default(),
        })
    }
}
```

**4. 性能优化策略**

```rust
// crates/pdf-core/src/parallel/work_stealing.rs
use crossbeam_deque::{Injector, Stealer, Worker};

pub struct AdaptiveScheduler {
    global_queue: Injector<PageTask>,
    workers: Vec<Worker<PageTask>>,
    stealers: Vec<Stealer<PageTask>>,
}

pub struct PageTask {
    pub page_index: u32,
    pub priority: u8,
}

impl AdaptiveScheduler {
    pub fn schedule_pages(&self, total_pages: u32) -> Vec<u32> {
        // 基于页面大小和复杂度的智能调度
        // 大页面优先处理，小页面用于填补间隙
        todo!()
    }
}
```

#### 性能指标
- **目标**: 4核CPU上批量处理速度提升 3-4x
- **内存**: 控制在单文件处理的 1.5x 以内
- **延迟**: 单文件处理延迟不增加

---

### 1.2 MCP采样能力实现

#### 设计目标
- 实现Server-initiated LLM调用
- 支持采样请求/响应协议
- 集成到现有MCP服务器

#### 技术方案

**1. MCP采样协议扩展**

```rust
// crates/pdf-mcp/src/sampling/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SamplingRequest {
    pub messages: Vec<SamplingMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<ModelPreferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_context: Option<IncludeContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SamplingMessage {
    pub role: Role,
    pub content: SamplingContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SamplingContent {
    Text { text: String },
    Image { data: String, mime_type: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelPreferences {
    pub hints: Vec<ModelHint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_priority: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_priority: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intelligence_priority: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SamplingResponse {
    pub model: String,
    pub role: Role,
    pub content: SamplingContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}
```

**2. 采样管理器实现**

```rust
// crates/pdf-mcp/src/sampling/manager.rs
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

pub struct SamplingManager {
    request_tx: mpsc::Sender<SamplingTask>,
}

struct SamplingTask {
    request: SamplingRequest,
    response_tx: oneshot::Sender<Result<SamplingResponse, SamplingError>>,
}

impl SamplingManager {
    pub fn new() -> Self {
        let (request_tx, mut request_rx) = mpsc::channel::<SamplingTask>(100);

        tokio::spawn(async move {
            while let Some(task) = request_rx.recv().await {
                let response = Self::handle_sampling_request(task.request).await;
                let _ = task.response_tx.send(response);
            }
        });

        Self { request_tx }
    }

    pub async fn request_sampling(
        &self,
        request: SamplingRequest,
    ) -> Result<SamplingResponse, SamplingError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.request_tx
            .send(SamplingTask {
                request,
                response_tx,
            })
            .await
            .map_err(|_| SamplingError::ChannelClosed)?;

        response_rx.await.map_err(|_| SamplingError::ResponseTimeout)?
    }

    async fn handle_sampling_request(
        request: SamplingRequest,
    ) -> Result<SamplingResponse, SamplingError> {
        // 通过MCP协议向客户端发送采样请求
        // 客户端调用其LLM并返回结果
        todo!()
    }
}
```

**3. 集成到PDF提取流程**

```rust
// crates/pdf-core/src/smart_extraction.rs
impl McpPdfPipeline {
    pub async fn extract_with_sampling(
        &self,
        file_path: &Path,
        sampling_manager: &SamplingManager,
    ) -> PdfResult<EnhancedExtractionResult> {
        let base_result = self.extract_structured(file_path, &ExtractOptions::default()).await?;

        // 对复杂页面发起采样请求
        let mut enhanced_pages = Vec::new();
        for page in &base_result.pages {
            if Self::needs_enhancement(page) {
                let sampling_request = SamplingRequest {
                    messages: vec![
                        SamplingMessage {
                            role: Role::User,
                            content: SamplingContent::Text {
                                text: format!(
                                    "Analyze this PDF page content and suggest improvements:\n{}",
                                    page.text
                                ),
                            },
                        },
                    ],
                    model_preferences: Some(ModelPreferences {
                        hints: vec![ModelHint { name: "claude-3".to_string() }],
                        speed_priority: Some(0.8),
                        ..Default::default()
                    }),
                    max_tokens: Some(1000),
                    ..Default::default()
                };

                let response = sampling_manager.request_sampling(sampling_request).await?;
                enhanced_pages.push(EnhancedPage {
                    base: page.clone(),
                    llm_insights: Some(response),
                });
            } else {
                enhanced_pages.push(EnhancedPage {
                    base: page.clone(),
                    llm_insights: None,
                });
            }
        }

        Ok(EnhancedExtractionResult {
            base: base_result,
            enhanced_pages,
        })
    }

    fn needs_enhancement(page: &PageMetadata) -> bool {
        page.text.len() < 100 || page.lines.is_empty()
    }
}
```

**4. MCP协议扩展**

```rust
// crates/pdf-mcp/src/server.rs
fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    let result = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "pdf-mcp",
            "version": "0.4.0",
        },
        "capabilities": {
            "tools": { "listChanged": false },
            "sampling": {
                "supported": true,
                "messageTypes": ["text", "image"]
            }
        }
    });
    JsonRpcResponse::success(request.id.clone(), result)
}

async fn handle_sampling_request(
    sampling_manager: &SamplingManager,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let params = request.params.as_ref().unwrap();
    let sampling_request: SamplingRequest = match serde_json::from_value(params.clone()) {
        Ok(req) => req,
        Err(e) => return JsonRpcResponse::error(request.id.clone(), JsonRpcError::invalid_params(&e.to_string())),
    };

    match sampling_manager.request_sampling(sampling_request).await {
        Ok(response) => JsonRpcResponse::success(request.id.clone(), serde_json::to_value(response).unwrap()),
        Err(e) => JsonRpcResponse::error(request.id.clone(), JsonRpcError::internal_error(&e.to_string())),
    }
}
```

#### 应用场景
1. **智能摘要**: 对长文档发起LLM摘要请求
2. **质量评估**: 让LLM评估提取质量
3. **结构优化**: 请求LLM优化文档结构
4. **多语言翻译**: 请求翻译服务

---

### 1.3 进度追踪系统 - Indicatif集成

#### 设计目标
- 实时进度条显示
- 多任务并行进度追踪
- 详细的性能指标

#### 技术方案

**1. 添加依赖**

```toml
# Cargo.toml
[workspace.dependencies]
indicatif = "0.17"
console = "0.15"
```

**2. 进度追踪系统架构**

```rust
// crates/pdf-core/src/progress/tracker.rs
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct ProgressTracker {
    multi_progress: Arc<MultiProgress>,
    config: ProgressConfig,
}

#[derive(Debug, Clone)]
pub struct ProgressConfig {
    pub show_speed: bool,
    pub show_eta: bool,
    pub show_percentage: bool,
    pub template: String,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            show_speed: true,
            show_eta: true,
            show_percentage: true,
            template: "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}".to_string(),
        }
    }
}

impl ProgressTracker {
    pub fn new(config: ProgressConfig) -> Self {
        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            config,
        }
    }

    pub fn create_file_progress(&self, file_name: &str, total_pages: u64) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(total_pages));
        pb.set_style(
            ProgressStyle::with_template(&self.config.template)
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  ")
                .tick_chars("⠁⠃⠇⡇⣇⣧⣷⣿")
        );
        pb.set_message(format!("Processing {}", file_name));
        pb
    }

    pub fn create_batch_progress(&self, total_files: u64) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(total_files));
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({percent}%) ETA: {eta}"
            )
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  ")
        );
        pb.set_message("Batch processing");
        pb
    }
}
```

**3. 集成到提取流程**

```rust
// crates/pdf-core/src/extractor.rs
impl McpPdfPipeline {
    pub async fn extract_structured_with_progress(
        &self,
        file_path: &Path,
        progress_tracker: &ProgressTracker,
    ) -> PdfResult<StructuredExtractionResult> {
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let page_count = self.get_page_count(file_path).await?;
        
        let pb = progress_tracker.create_file_progress(file_name, page_count as u64);
        
        let result = self.extract_structured_internal(file_path, |page_num| {
            pb.set_position(page_num as u64);
            pb.set_message(format!("Page {}/{}", page_num, page_count));
        }).await?;

        pb.finish_with_message(format!("✓ Completed {} ({} pages)", file_name, page_count));
        
        Ok(result)
    }

    pub async fn extract_batch_with_progress(
        &self,
        files: &[PathBuf],
        options: &ExtractOptions,
    ) -> Vec<(PathBuf, PdfResult<StructuredExtractionResult>)> {
        let tracker = ProgressTracker::new(ProgressConfig::default());
        let batch_pb = tracker.create_batch_progress(files.len() as u64);

        let results: Vec<_> = files
            .iter()
            .enumerate()
            .map(|(i, file_path)| {
                let result = tokio::runtime::Handle::current()
                    .block_on(self.extract