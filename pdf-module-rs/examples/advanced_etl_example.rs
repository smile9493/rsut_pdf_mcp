//! 高级 ETL 流水线使用示例

use pdf_etl::{
    config::{DatabaseConfig, DatabaseType, ExtractionConfig, LLMConfig, LLMProvider},
    pipeline::{AdvancedETLPipeline, AdvancedPipelineConfig, SchemaDefinition},
    schema::{ContractSchema, InvoiceSchema, ResumeSchema},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("=== PDF 高级 ETL 流水线示例 ===\n");

    // 1. 配置 LLM
    let llm_config = LLMConfig {
        provider: LLMProvider::OpenAI,
        model: "gpt-4o".to_string(),
        api_key: std::env::var("OPENAI_API_KEY")
            .expect("请设置 OPENAI_API_KEY 环境变量"),
        base_url: None,
        temperature: 0.0,
        max_tokens: Some(4096),
        timeout: 60,
        deployment_name: None,
        api_version: "2024-02-15-preview".to_string(),
    };

    // 2. 配置数据库
    let db_config = DatabaseConfig {
        db_type: DatabaseType::Postgres,
        connection_string: "postgresql://postgres:postgres@localhost:5432/pdf_etl".to_string(),
        table_name: "extracted_documents".to_string(),
        pool_size: 10,
        use_jsonb: true,
    };

    // 3. 配置提取引擎
    let extraction_config = ExtractionConfig::default();

    // 4. 配置高级流水线
    let advanced_config = AdvancedPipelineConfig {
        enable_layout_analysis: true,
        enable_validation: true,
        validation_threshold: 0.8,
        enable_privacy_mask: true,
        max_retries: 3,
        auto_path_selection: true,
    };

    // 5. 创建高级 ETL 流水线
    println!("创建高级 ETL 流水线...");
    let pipeline = AdvancedETLPipeline::new(
        extraction_config,
        llm_config,
        Some(db_config),
        Some(advanced_config),
    )
    .await?;

    println!("流水线创建成功!\n");

    // 6. 示例 1: 提取发票信息
    println!("--- 示例 1: 发票提取 ---");
    let invoice_schema = SchemaDefinition::from_struct::<InvoiceSchema>();
    let invoice_pdf = "./test_data/invoice.pdf";

    // 检查文件是否存在
    if std::path::Path::new(invoice_pdf).exists() {
        println!("处理发票: {}", invoice_pdf);
        let result = pipeline
            .execute(
                invoice_pdf,
                invoice_schema,
                Some("请从发票中提取所有关键信息,包括发票号、日期、供应商、项目和金额。"),
                true, // 保存到数据库
            )
            .await?;

        println!("提取结果:");
        println!("  - 处理时间: {}ms", result.total_time_ms);
        println!("  - 输入 Token: {}", result.transform.input_tokens);
        println!("  - 输出 Token: {}", result.transform.output_tokens);
        println!("  - 验证状态: {}", result.transform.is_valid);
        println!("  - 数据: {}", serde_json::to_string_pretty(&result.transform.data)?);

        if let Some(save_result) = result.save {
            println!("  - 数据库记录 ID: {:?}", save_result.record_id);
        }
    } else {
        println!("跳过发票示例: 文件不存在");
    }

    println!();

    // 7. 示例 2: 提取合同信息
    println!("--- 示例 2: 合同提取 ---");
    let contract_schema = SchemaDefinition::from_struct::<ContractSchema>();
    let contract_pdf = "./test_data/contract.pdf";

    if std::path::Path::new(contract_pdf).exists() {
        println!("处理合同: {}", contract_pdf);
        let result = pipeline
            .execute(
                contract_pdf,
                contract_schema,
                Some("请从合同中提取关键条款,包括合同编号、甲乙方信息、签署日期和金额。"),
                true,
            )
            .await?;

        println!("提取结果:");
        println!("  - 处理时间: {}ms", result.total_time_ms);
        println!("  - 数据: {}", serde_json::to_string_pretty(&result.transform.data)?);
    } else {
        println!("跳过合同示例: 文件不存在");
    }

    println!();

    // 8. 示例 3: 批量处理简历
    println!("--- 示例 3: 批量简历提取 ---");
    let resume_schema = SchemaDefinition::from_struct::<ResumeSchema>();
    let resume_pdfs = vec![
        "./test_data/resume1.pdf".to_string(),
        "./test_data/resume2.pdf".to_string(),
    ];

    // 过滤存在的文件
    let existing_resumes: Vec<String> = resume_pdfs
        .into_iter()
        .filter(|path| std::path::Path::new(path).exists())
        .collect();

    if !existing_resumes.is_empty() {
        println!("批量处理 {} 份简历...", existing_resumes.len());
        let results = pipeline
            .execute_batch(
                &existing_resumes,
                resume_schema,
                Some("请从简历中提取个人信息、工作经历、教育背景和技能。"),
                true,
            )
            .await?;

        println!("批量处理完成:");
        for (i, result) in results.iter().enumerate() {
            println!(
                "  [{}] 处理时间: {}ms, 有效: {}",
                i + 1,
                result.total_time_ms,
                result.transform.is_valid
            );
        }
    } else {
        println!("跳过简历示例: 文件不存在");
    }

    println!();

    // 9. 示例 4: 使用自定义 Schema
    println!("--- 示例 4: 自定义 Schema ---");
    let custom_schema_json = r#"{
        "type": "object",
        "properties": {
            "document_type": {
                "type": "string",
                "description": "文档类型"
            },
            "summary": {
                "type": "string",
                "description": "文档摘要"
            },
            "key_points": {
                "type": "array",
                "items": {
                    "type": "string"
                },
                "description": "关键要点"
            },
            "confidence": {
                "type": "number",
                "description": "置信度 (0-1)"
            }
        }
    }"#;

    let custom_schema = SchemaDefinition::from_json(custom_schema_json)?;
    println!("自定义 Schema: {}", custom_schema.name);

    let custom_pdf = "./test_data/document.pdf";
    if std::path::Path::new(custom_pdf).exists() {
        let result = pipeline
            .execute(
                custom_pdf,
                custom_schema,
                Some("请分析文档并提取文档类型、摘要和关键要点。"),
                false, // 不保存到数据库
            )
            .await?;

        println!("提取结果: {}", serde_json::to_string_pretty(&result.transform.data)?);
    } else {
        println!("跳过自定义 Schema 示例: 文件不存在");
    }

    println!("\n=== 示例完成 ===");

    Ok(())
}
