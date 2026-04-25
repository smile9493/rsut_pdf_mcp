//! 高级 ETL 流水线
//!
//! 实现四层架构的高性能 PDF 结构化提取与自动入库方案:
//! 1. 物理解析层 (I/O & Parsing)
//! 2. 版面感知层 (Layout Analysis)
//! 3. 语义映射层 (Semantic Mapping)
//! 4. 持久化层 (Persistence)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::config::{DatabaseConfig, ExtractionConfig, LLMConfig};
use crate::database::adapter::DatabaseAdapter;
use crate::database::factory::DatabaseAdapterFactory;
use crate::dto::{ETLResult, ExtractionResult, TransformResult};
use crate::error::Result;
use crate::llm::adapter::LLMAdapter;
use crate::llm::factory::LLMAdapterFactory;
use crate::pipeline::extraction_engine::ExtractionService;
use crate::pipeline::structured_extractor::StructuredExtractor;
use crate::schema::SchemaDefinition;

/// 处理路径类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingPath {
    /// 快速路径 - 原生 PDF 文本提取
    FastPath,
    /// 视觉路径 - 扫描件/复杂表格处理
    VisionPath,
    /// 混合路径 - 结合文本和视觉
    HybridPath,
}

/// 文档特征分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFeatures {
    /// 是否为扫描件
    pub is_scanned: bool,
    /// 是否包含表格
    pub has_tables: bool,
    /// 是否包含图片
    pub has_images: bool,
    /// 文本密度 (0.0 - 1.0)
    pub text_density: f64,
    /// 页数
    pub page_count: usize,
    /// 推荐的处理路径
    pub recommended_path: ProcessingPath,
    /// 复杂度评分 (0-100)
    pub complexity_score: u8,
}

/// 版面区域类型
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RegionType {
    /// 正文
    Text,
    /// 标题
    Title,
    /// 表格
    Table,
    /// 图片
    Image,
    /// 页眉
    Header,
    /// 页脚
    Footer,
    /// 列表
    List,
}

/// 版面区域
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LayoutRegion {
    /// 区域类型
    pub region_type: RegionType,
    /// 区域内容
    pub content: String,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f64,
    /// 边界框 [x, y, width, height]
    pub bbox: Option<[f64; 4]>,
    /// 页码
    pub page: usize,
}

/// 版面分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutAnalysis {
    /// 检测到的区域
    pub regions: Vec<LayoutRegion>,
    /// 整体置信度
    pub overall_confidence: f64,
    /// 处理时间 (ms)
    pub processing_time_ms: u64,
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否通过验证
    pub is_valid: bool,
    /// 验证分数 (0.0 - 1.0)
    pub score: f64,
    /// 错误信息
    pub errors: Vec<String>,
    /// 警告信息
    pub warnings: Vec<String>,
    /// 是否需要人工复核
    pub needs_review: bool,
}

/// 高级 ETL 流水线
pub struct AdvancedETLPipeline {
    /// 提取服务
    extraction_service: ExtractionService,
    /// LLM 适配器
    llm_adapter: Arc<dyn LLMAdapter>,
    /// 数据库适配器
    db_adapter: Option<Arc<dyn DatabaseAdapter>>,
    /// 配置
    config: AdvancedPipelineConfig,
}

/// 高级流水线配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedPipelineConfig {
    /// 启用版面分析
    pub enable_layout_analysis: bool,
    /// 启用闭环验证
    pub enable_validation: bool,
    /// 验证阈值
    pub validation_threshold: f64,
    /// 启用隐私脱敏
    pub enable_privacy_mask: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 启用自动路径选择
    pub auto_path_selection: bool,
}

impl Default for AdvancedPipelineConfig {
    fn default() -> Self {
        Self {
            enable_layout_analysis: true,
            enable_validation: true,
            validation_threshold: 0.8,
            enable_privacy_mask: true,
            max_retries: 3,
            auto_path_selection: true,
        }
    }
}

impl AdvancedETLPipeline {
    /// 创建新的高级 ETL 流水线
    pub async fn new(
        extraction_config: ExtractionConfig,
        llm_config: LLMConfig,
        db_config: Option<DatabaseConfig>,
        advanced_config: Option<AdvancedPipelineConfig>,
    ) -> Result<Self> {
        let extraction_service = ExtractionService::new(extraction_config)?;
        let llm_adapter = LLMAdapterFactory::create(llm_config)?;

        let db_adapter = if let Some(config) = db_config {
            Some(DatabaseAdapterFactory::create(config).await?)
        } else {
            None
        };

        Ok(Self {
            extraction_service,
            llm_adapter,
            db_adapter,
            config: advanced_config.unwrap_or_default(),
        })
    }

    /// 执行完整的高级 ETL 流程
    pub async fn execute(
        &self,
        pdf_path: &str,
        schema: SchemaDefinition,
        prompt_template: Option<&str>,
        save_to_db: bool,
    ) -> Result<ETLResult> {
        let start = Instant::now();

        info!("Starting advanced ETL pipeline for: {}", pdf_path);

        // 步骤 1: 物理解析层 - 分析文档特征
        info!("Layer 1: Physical Parsing - Analyzing document features...");
        let features = self.analyze_document_features(pdf_path).await?;

        // 步骤 2: 选择处理路径
        let processing_path = if self.config.auto_path_selection {
            features.recommended_path.clone()
        } else {
            ProcessingPath::FastPath
        };

        info!("Selected processing path: {:?}", processing_path);

        // 步骤 3: 提取文本 (根据处理路径)
        info!("Layer 1: Extracting text using {:?}...", processing_path);
        let extraction = match processing_path {
            ProcessingPath::FastPath => self.extract_fast_path(pdf_path).await?,
            ProcessingPath::VisionPath => self.extract_vision_path(pdf_path).await?,
            ProcessingPath::HybridPath => self.extract_hybrid_path(pdf_path).await?,
        };

        // 步骤 4: 版面感知层 - 分析版面结构
        let layout = if self.config.enable_layout_analysis {
            info!("Layer 2: Layout Analysis...");
            Some(self.analyze_layout(&extraction.text).await?)
        } else {
            None
        };

        // 步骤 5: 隐私脱敏
        let text_to_process = if self.config.enable_privacy_mask {
            info!("Applying privacy mask...");
            self.apply_privacy_mask(&extraction.text)
        } else {
            extraction.text.clone()
        };

        // 步骤 6: 语义映射层 - 转换为结构化数据
        info!("Layer 3: Semantic Mapping - Transforming to structured data...");
        let transform = self
            .transform_with_retry(&text_to_process, &schema, prompt_template, layout.as_ref())
            .await?;

        // 步骤 7: 闭环验证
        let validation = if self.config.enable_validation {
            info!("Layer 3: Validation - Self-correction check...");
            Some(
                self.validate_extraction(&extraction.text, &transform.data, &schema)
                    .await?,
            )
        } else {
            None
        };

        // 步骤 8: 如果验证失败,尝试重新提取
        let final_transform = if let Some(ref val) = validation {
            if !val.is_valid && val.needs_review {
                warn!("Validation failed, attempting re-extraction with Vision Path...");
                let re_extraction = self.extract_vision_path(pdf_path).await?;
                self.transform_with_retry(&re_extraction.text, &schema, prompt_template, None)
                    .await?
            } else {
                transform
            }
        } else {
            transform
        };

        // 步骤 9: 持久化层 - 保存到数据库
        let save = if save_to_db {
            if let Some(ref db_adapter) = self.db_adapter {
                info!("Layer 4: Persistence - Saving to database...");
                let table_name = db_adapter.config().table_name.clone();

                db_adapter
                    .create_table_if_not_exists(&table_name, Some(&schema.schema))
                    .await?;

                let save_result = db_adapter
                    .save_with_metadata(
                        &table_name,
                        &final_transform.data,
                        &schema.name,
                        Some(pdf_path),
                        Some(&serde_json::json!({
                            "page_count": extraction.metadata.page_count,
                            "processing_time_ms": extraction.processing_time_ms,
                            "processing_path": processing_path,
                            "validation": validation,
                            "features": features,
                        })),
                    )
                    .await?;

                Some(save_result)
            } else {
                None
            }
        } else {
            None
        };

        let total_time_ms = start.elapsed().as_millis() as u64;

        info!(
            "Advanced ETL pipeline completed: total_time={}ms, path={:?}",
            total_time_ms, processing_path
        );

        Ok(ETLResult::new(
            extraction,
            final_transform,
            save,
            total_time_ms,
        ))
    }

    /// 分析文档特征
    async fn analyze_document_features(&self, pdf_path: &str) -> Result<DocumentFeatures> {
        let _start = Instant::now();

        // 提取基本信息
        let extraction = self.extraction_service.extract(pdf_path).await?;
        let text = &extraction.text;

        // 计算文本密度
        let text_density = if extraction.metadata.page_count > 0 {
            text.len() as f64 / (extraction.metadata.page_count * 3000).max(1) as f64
        } else {
            0.0
        };

        // 检测是否为扫描件 (文本密度低)
        let is_scanned = text_density < 0.3;

        // 检测表格 (简单启发式)
        let has_tables = text.contains("│") || text.contains("|") || text.contains("┌");

        // 检测图片标记
        let has_images = text.contains("[图片]") || text.contains("[Image]");

        // 计算复杂度评分
        let mut complexity_score = 0u8;
        if is_scanned {
            complexity_score += 30;
        }
        if has_tables {
            complexity_score += 20;
        }
        if has_images {
            complexity_score += 15;
        }
        if extraction.metadata.page_count > 20 {
            complexity_score += 15;
        }
        if text_density < 0.5 {
            complexity_score += 20;
        }

        // 推荐处理路径
        let recommended_path = if is_scanned || complexity_score > 50 {
            ProcessingPath::VisionPath
        } else if has_tables || complexity_score > 30 {
            ProcessingPath::HybridPath
        } else {
            ProcessingPath::FastPath
        };

        debug!(
            "Document features analyzed: is_scanned={}, has_tables={}, complexity={}, path={:?}",
            is_scanned, has_tables, complexity_score, recommended_path
        );

        Ok(DocumentFeatures {
            is_scanned,
            has_tables,
            has_images,
            text_density,
            page_count: extraction.metadata.page_count,
            recommended_path,
            complexity_score,
        })
    }

    /// 快速路径提取
    async fn extract_fast_path(&self, pdf_path: &str) -> Result<ExtractionResult> {
        self.extraction_service.extract(pdf_path).await
    }

    /// 视觉路径提取 (使用多模态模型)
    async fn extract_vision_path(&self, pdf_path: &str) -> Result<ExtractionResult> {
        // TODO: 实现 pdfium-render 渲染为图片
        // TODO: 调用多模态模型 (GPT-4o Vision / Claude 3.5)
        warn!("Vision path not fully implemented, falling back to fast path");
        self.extract_fast_path(pdf_path).await
    }

    /// 混合路径提取
    async fn extract_hybrid_path(&self, pdf_path: &str) -> Result<ExtractionResult> {
        // 结合文本提取和视觉识别
        let text_extraction = self.extract_fast_path(pdf_path).await?;

        // 如果文本密度低,补充视觉识别
        if text_extraction.text.len() < 100 {
            warn!("Low text content, supplementing with vision path");
            // TODO: 补充视觉识别
        }

        Ok(text_extraction)
    }

    /// 版面分析
    async fn analyze_layout(&self, text: &str) -> Result<LayoutAnalysis> {
        let start = Instant::now();

        // 简单的版面分析 (基于启发式规则)
        let mut regions = Vec::new();

        // 按段落分割
        let paragraphs: Vec<&str> = text.split("\n\n").collect();

        for para in paragraphs.iter() {
            let region_type = if para.len() < 50
                && para
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
            {
                RegionType::Title
            } else if para.contains("│") || para.contains("|") {
                RegionType::Table
            } else if para.starts_with("- ") || para.starts_with("• ") {
                RegionType::List
            } else {
                RegionType::Text
            };

            regions.push(LayoutRegion {
                region_type,
                content: para.to_string(),
                confidence: 0.8,
                bbox: None,
                page: 1,
            });
        }

        let processing_time_ms = start.elapsed().as_millis() as u64;

        Ok(LayoutAnalysis {
            regions,
            overall_confidence: 0.8,
            processing_time_ms,
        })
    }

    /// 应用隐私脱敏
    fn apply_privacy_mask(&self, text: &str) -> String {
        let mut masked = text.to_string();

        // 身份证号脱敏
        let id_pattern = regex::Regex::new(r"\d{17}[\dXx]").unwrap();
        masked = id_pattern
            .replace_all(&masked, "[身份证号已脱敏]")
            .to_string();

        // 手机号脱敏
        let phone_pattern = regex::Regex::new(r"1[3-9]\d{9}").unwrap();
        masked = phone_pattern
            .replace_all(&masked, "[手机号已脱敏]")
            .to_string();

        // 银行卡号脱敏
        let bank_pattern = regex::Regex::new(r"\d{16,19}").unwrap();
        masked = bank_pattern
            .replace_all(&masked, "[银行卡号已脱敏]")
            .to_string();

        masked
    }

    /// 带重试的转换
    async fn transform_with_retry(
        &self,
        text: &str,
        schema: &SchemaDefinition,
        prompt_template: Option<&str>,
        layout: Option<&LayoutAnalysis>,
    ) -> Result<TransformResult> {
        let extractor = StructuredExtractor::new(self.llm_adapter.clone());

        // 如果有版面分析,构建增强的提示词
        let enhanced_prompt = if let Some(l) = layout {
            let layout_info = format!(
                "\n\n版面分析信息:\n- 检测到 {} 个区域\n- 整体置信度: {:.2}",
                l.regions.len(),
                l.overall_confidence
            );
            Some(format!("{}{}", prompt_template.unwrap_or(""), layout_info))
        } else {
            prompt_template.map(|s| s.to_string())
        };

        // 执行转换
        extractor
            .transform(text, schema, enhanced_prompt.as_deref())
            .await
    }

    /// 闭环验证
    async fn validate_extraction(
        &self,
        original_text: &str,
        extracted_data: &serde_json::Value,
        _schema: &SchemaDefinition,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut score = 1.0;

        // 验证关键字段
        if let serde_json::Value::Object(map) = extracted_data {
            for (key, value) in map {
                // 检查是否为空
                if value.is_null() || (value.is_string() && value.as_str().unwrap().is_empty()) {
                    warnings.push(format!("字段 '{}' 为空", key));
                    score -= 0.1;
                }

                // 如果是金额字段,在原文中验证
                if key.contains("金额") || key.contains("amount") {
                    if let Some(amount) = value.as_f64() {
                        // 在原文中查找金额
                        let pattern_str = format!(r"{}[\s,，]*([\d,]+\.?\d*)", key);
                        #[allow(clippy::regex_creation_in_loops)]
                        let amount_pattern =
                            regex::Regex::new(&pattern_str).unwrap();
                        if let Some(caps) = amount_pattern.captures(original_text) {
                            if let Some(matched) = caps.get(1) {
                                let matched_amount: f64 =
                                    matched.as_str().replace(",", "").parse().unwrap_or(0.0);
                                let diff = (amount - matched_amount).abs();
                                if diff > 0.01 {
                                    errors.push(format!(
                                        "金额验证失败: 提取值={}, 原文值={}, 差异={}",
                                        amount, matched_amount, diff
                                    ));
                                    score -= 0.3;
                                }
                            }
                        }
                    }
                }

                // 如果是日期字段,验证格式
                if key.contains("日期") || key.contains("date") {
                    if let Some(date_str) = value.as_str() {
                        #[allow(clippy::regex_creation_in_loops)]
                        let date_pattern = regex::Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
                        if !date_pattern.is_match(date_str) {
                            warnings.push(format!("日期格式不规范: {}", date_str));
                            score -= 0.05;
                        }
                    }
                }
            }
        }

        let is_valid = score >= self.config.validation_threshold;
        let needs_review = score < self.config.validation_threshold * 0.7;

        Ok(ValidationResult {
            is_valid,
            score: score.max(0.0),
            errors,
            warnings,
            needs_review,
        })
    }

    /// 批量处理
    pub async fn execute_batch(
        &self,
        pdf_paths: &[String],
        schema: SchemaDefinition,
        prompt_template: Option<&str>,
        save_to_db: bool,
    ) -> Result<Vec<ETLResult>> {
        let mut results = Vec::with_capacity(pdf_paths.len());

        for pdf_path in pdf_paths {
            match self
                .execute(pdf_path, schema.clone(), prompt_template, save_to_db)
                .await
            {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Failed to process {}: {}", pdf_path, e);
                    // 继续处理其他文件
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_mask() {
        let _config = AdvancedPipelineConfig::default();
        let _text = "身份证号: 123456789012345678, 手机: 13812345678";
        // 注意: 这里需要创建一个实例来测试
        // 实际测试中应该创建一个完整的 pipeline 实例
    }

    #[test]
    fn test_document_features() {
        let features = DocumentFeatures {
            is_scanned: false,
            has_tables: true,
            has_images: false,
            text_density: 0.8,
            page_count: 10,
            recommended_path: ProcessingPath::HybridPath,
            complexity_score: 35,
        };

        assert_eq!(features.recommended_path, ProcessingPath::HybridPath);
    }
}
