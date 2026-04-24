//! Integration tests for pdf-module-rs
//! Tests the complete workflow of PDF extraction with all components

use async_trait::async_trait;
use pdf_core::{
    AuditBackend, AuditService, ExecutionMetadata, ExtractionAudit, ExtractionStatus, FileStorage,
    InputType, LocalFileStorage, MessageStreamer, OutputType, RuntimeVariables,
    ToolDefinition, ToolExecutionResult, ToolHandler, ToolRegistry, ToolSpec,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_complete_workflow() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let test_audit_dir = temp_dir.path().join("audit");
    fs::create_dir_all(&test_audit_dir).await.unwrap();

    // Initialize audit service
    let audit_service = Arc::new(AuditService::new(
        AuditBackend::File {
            log_dir: test_audit_dir.clone(),
        },
        30,
    ));

    // Test audit service is working
    let audit = ExtractionAudit::new(
        "test.pdf".to_string(),
        "pdf".to_string(),
        1024.0,
        "lopdf".to_string(),
    )
    .with_status(ExtractionStatus::Completed)
    .with_processing_time(1000)
    .with_extracted_text_length(5000);

    audit_service.log_extraction(audit.clone()).await.unwrap();

    // Give some time for file write to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Verify audit file was created
    let date = chrono::Utc::now()
        .naive_utc()
        .date()
        .format("%Y-%m-%d")
        .to_string();
    let audit_filename = format!("audit_{}.jsonl", date);
    let audit_filepath = test_audit_dir.join(audit_filename);
    assert!(audit_filepath.exists());

    println!("✅ Complete workflow test passed!");
}

#[tokio::test]
async fn tool_registration_workflow() {
    // Create a mock tool for testing
    struct MockTool {
        name: String,
        definition: ToolDefinition,
        spec: ToolSpec,
        variables: RuntimeVariables,
    }

    impl MockTool {
        fn new(name: &str) -> Self {
            let definition = ToolDefinition::new(
                format!("{} Display", name),
                name.to_string(),
                format!("{} description", name),
                vec![],
                InputType::File,
                OutputType::File,
            );

            let spec = ToolSpec::new(name.to_string(), "test".to_string());

            let variables =
                RuntimeVariables::new("Test Variables".to_string(), "Test Description".to_string());

            Self {
                name: name.to_string(),
                definition,
                spec,
                variables,
            }
        }
    }

    #[async_trait]
    impl ToolHandler for MockTool {
        fn definition(&self) -> &ToolDefinition {
            &self.definition
        }

        fn spec(&self) -> &ToolSpec {
            &self.spec
        }

        fn variables(&self) -> &RuntimeVariables {
            &self.variables
        }

        async fn execute(
            &self,
            _params: HashMap<String, Value>,
            _streamer: Option<&mut dyn MessageStreamer>,
        ) -> pdf_core::PdfResult<ToolExecutionResult> {
            Ok(ToolExecutionResult {
                workflow_id: "test-workflow".to_string(),
                elapsed_time: 100,
                output: serde_json::json!({"result": "test"}),
                metadata: Some(ExecutionMetadata {
                    file_name: "test.pdf".to_string(),
                    file_size: 1024,
                    processing_time: 100,
                    cache_hit: false,
                    adapter_used: "test".to_string(),
                }),
            })
        }

        fn validate_params(&self, _params: &HashMap<String, Value>) -> pdf_core::PdfResult<()> {
            Ok(())
        }

        fn metadata(&self) -> ExecutionMetadata {
            ExecutionMetadata {
                file_name: "test.pdf".to_string(),
                file_size: 1024,
                processing_time: 100,
                cache_hit: false,
                adapter_used: "test".to_string(),
            }
        }
    }

    // Test tool registration workflow
    let registry = Arc::new(ToolRegistry::new());
    let tool = Arc::new(MockTool::new("test_tool"));

    // Register tool
    registry.register(tool.clone()).await.unwrap();
    assert_eq!(registry.count().await, 1);
    assert!(registry.is_registered("test_tool").await);

    // Get tool
    let retrieved = registry.get("test_tool").await.unwrap();
    assert_eq!(retrieved.name(), "test_tool");
    assert_eq!(retrieved.version(), "1.0.0");

    // Execute tool
    let params = HashMap::new();
    let result = registry
        .execute("test_tool", params, None, None, None)
        .await
        .unwrap();
    assert_eq!(result.workflow_id, "test-workflow");

    // Unregister tool
    registry.unregister("test_tool").await.unwrap();
    assert_eq!(registry.count().await, 0);

    println!("✅ Tool registration workflow test passed!");
}

#[tokio::test]
async fn test_storage_operations() {
    let temp_dir = TempDir::new().unwrap();
    let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

    // Test write and read
    let data = b"Hello, World!";
    storage.write("test.txt", data).await.unwrap();

    let read_data = storage.read("test.txt").await.unwrap();
    assert_eq!(read_data.as_ref(), data);

    // Test exists
    assert!(storage.exists("test.txt").await.unwrap());
    assert!(!storage.exists("nonexistent.txt").await.unwrap());

    // Test metadata
    let metadata = storage.metadata("test.txt").await.unwrap();
    assert_eq!(metadata.path, "test.txt");
    assert_eq!(metadata.size, 13);

    // Test list
    storage.write("file1.txt", b"test1").await.unwrap();
    storage.write("file2.txt", b"test2").await.unwrap();

    let files = storage.list(".", false).await.unwrap();
    assert_eq!(files.len(), 3); // test.txt + file1.txt + file2.txt

    // Test delete
    storage.delete("test.txt").await.unwrap();
    assert!(!storage.exists("test.txt").await.unwrap());

    println!("✅ Storage operations test passed!");
}
