//! 集成测试：验证 V3.1 新增 Trait 之间的协作

use std::sync::Arc;

use pdf_common::vector::{
    AccessLevel, InMemoryVectorStorage, VectorQuery, VectorRecord, VectorStorage,
};
use pdf_common::logic_storage::{
    InMemoryLogicStorage, LogicStorage, TransactionOp,
};
use pdf_common::outbox::{
    InMemoryOutboxProcessor, OutboxRecord, OutboxStatus, OutboxProcessor,
};
use pdf_common::rbac::{
    InMemoryRbacChecker, Permission, RbacChecker, TokenSource, UserContext,
};
use pdf_common::score_calibrator::{PercentileCalibrator, ScoreCalibrator};
use pdf_common::compensation::CompensationService;
use pdf_common::dual_store::{DualStoreWriter, OutboxWorker};
use pdf_common::hybrid_search::HybridSearchFlow;
use pdf_common::backpressure::{
    AdaptiveBackpressure, AdaptiveBackpressureController, SlidingWindowConfig,
};
use pdf_common::reconciliation::{ReconciliationCursor, ReconciliationWorker};
use pdf_common::progressive_router::ProgressiveVectorRouter;
use pdf_common::graph_storage::{GraphEdge, GraphStorage};
use pdf_common::prompt_hint::{self, tool_description_with_hint, PromptHint};
use pdf_common::confidence_interceptor::{ConfidenceInterceptor, ConfidenceInterceptorConfig};
use pdf_common::vector_gc::{QueryTracker, VectorGcConfig, VectorGcWorker};
use pdf_common::ast_chunker::{AstChunker, AstChunkerConfig};

// ============================================================
// 辅助函数
// ============================================================

fn make_user(role: &str, max_level: AccessLevel) -> UserContext {
    UserContext {
        user_id: "user-1".to_string(),
        org_id: Some("org-1".to_string()),
        roles: vec![role.to_string()],
        max_access_level: max_level,
        token_source: TokenSource::Header,
    }
}

fn make_vector_record(id: &str, vector: Vec<f32>, seq: u64) -> VectorRecord {
    VectorRecord {
        id: id.to_string(),
        vector,
        metadata: serde_json::json!({"access_level": "public"}),
        embedding_version: 1,
        created_seq: seq,
    }
}

// ============================================================
// T1: 向量存储 + 逻辑存储 + Outbox 基础协作
// ============================================================

#[tokio::test]
async fn test_vector_and_logic_storage_workflow() {
    let vs = Arc::new(InMemoryVectorStorage::new("test-vs"));
    let ls = Arc::new(InMemoryLogicStorage::new("test-ls"));
    let ob = Arc::new(InMemoryOutboxProcessor::new());

    vs.initialize(3).await.unwrap();

    let wr = ls
        .write_with_seq("etl_results", serde_json::json!({"content": "hello"}))
        .await
        .unwrap();
    assert_eq!(wr.seq_id, 1);
    assert!(!wr.record_id.is_empty());

    let record = OutboxRecord::new(wr.seq_id, "etl_results", &wr.record_id, 3);
    ob.push(record.clone());
    assert_eq!(record.status, OutboxStatus::Pending);

    let pending = ob.fetch_pending(10).await.unwrap();
    assert_eq!(pending.len(), 1);

    let lance_id = ob.process(&pending[0]).await.unwrap();
    ob.mark_completed(&pending[0].id, &lance_id).await.unwrap();

    let vr = make_vector_record("v1", vec![1.0, 0.0, 0.0], wr.seq_id);
    vs.insert(vec![vr]).await.unwrap();

    let stats = vs.stats().await.unwrap();
    assert_eq!(stats.total_vectors, 1);

    let query = VectorQuery {
        vector: vec![1.0, 0.0, 0.0],
        top_k: 10,
        filter: None,
        embedding_version: None,
    };
    let results = vs.search(query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "v1");
}

// ============================================================
// T2: 双库锚定闭环
// ============================================================

#[tokio::test]
async fn test_dual_store_closed_loop() {
    let vs = Arc::new(InMemoryVectorStorage::new("test-vs"));
    let ls = Arc::new(InMemoryLogicStorage::new("test-ls"));
    let ob = Arc::new(InMemoryOutboxProcessor::new());

    vs.initialize(3).await.unwrap();

    let writer = DualStoreWriter::new(ls.clone(), ob.clone(), 3);
    let r1 = writer
        .write("docs", serde_json::json!({"title": "doc1"}))
        .await
        .unwrap();
    let r2 = writer
        .write("docs", serde_json::json!({"title": "doc2"}))
        .await
        .unwrap();
    assert_eq!(r1.seq_id, 1);
    assert_eq!(r2.seq_id, 2);

    let records = ls.query("docs", None).await.unwrap();
    assert_eq!(records.len(), 2);

    let worker = OutboxWorker::new(ob.clone(), vs, 10);
    let results = worker.process_batch().await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.success));
}

// ============================================================
// T3: RBAC 权限过滤 + 混合检索
// ============================================================

#[tokio::test]
async fn test_rbac_filtered_hybrid_search() {
    let vs = Arc::new(InMemoryVectorStorage::new("test-vs"));
    vs.initialize(3).await.unwrap();

    let records = vec![
        VectorRecord {
            id: "pub-1".to_string(),
            vector: vec![1.0, 0.0, 0.0],
            metadata: serde_json::json!({"access_level": "public"}),
            embedding_version: 1,
            created_seq: 1,
        },
        VectorRecord {
            id: "int-1".to_string(),
            vector: vec![0.9, 0.1, 0.0],
            metadata: serde_json::json!({"access_level": "internal"}),
            embedding_version: 1,
            created_seq: 2,
        },
        VectorRecord {
            id: "sec-1".to_string(),
            vector: vec![0.8, 0.2, 0.0],
            metadata: serde_json::json!({"access_level": "secret"}),
            embedding_version: 1,
            created_seq: 3,
        },
    ];
    vs.insert(records).await.unwrap();

    let rbac = Arc::new(InMemoryRbacChecker::new());
    rbac.register_permission(
        "vectors".to_string(),
        "reader".to_string(),
        vec![Permission::Read],
    );

    let calibrator = Arc::new(PercentileCalibrator::new());
    let outbox = Arc::new(InMemoryOutboxProcessor::new());
    let compensation = Arc::new(CompensationService::new(outbox));
    let flow = HybridSearchFlow::new(vs, rbac, calibrator, compensation);

    // Public 用户只能看到 public 记录
    let public_user = make_user("reader", AccessLevel::Public);
    let query = VectorQuery {
        vector: vec![1.0, 0.0, 0.0],
        top_k: 10,
        filter: None,
        embedding_version: None,
    };
    let results = flow.search(&public_user, query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "pub-1");

    // Internal 用户可以看到 public + internal
    let internal_user = make_user("reader", AccessLevel::Internal);
    let query2 = VectorQuery {
        vector: vec![1.0, 0.0, 0.0],
        top_k: 10,
        filter: None,
        embedding_version: None,
    };
    let results2 = flow.search(&internal_user, query2).await.unwrap();
    assert_eq!(results2.len(), 2);

    // 无权限用户返回空
    let no_perm_user = UserContext {
        user_id: "u-no-perm".to_string(),
        org_id: None,
        roles: vec!["guest".to_string()],
        max_access_level: AccessLevel::Public,
        token_source: TokenSource::Header,
    };
    let query3 = VectorQuery {
        vector: vec![1.0, 0.0, 0.0],
        top_k: 10,
        filter: None,
        embedding_version: None,
    };
    let results3 = flow.search(&no_perm_user, query3).await.unwrap();
    assert!(results3.is_empty());
}

// ============================================================
// T4: 分数校准
// ============================================================

#[tokio::test]
async fn test_score_calibration_in_search() {
    let calibrator = PercentileCalibrator::new();

    calibrator.update_stats("table_a", &[0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]);
    let stats = calibrator.stats("table_a").unwrap();
    assert_eq!(stats.sample_count, 7);

    let calibrated = calibrator.calibrate(0.6, "table_a");
    assert!((calibrated - 0.5).abs() < 0.1);

    let raw = calibrator.calibrate(0.8, "unknown_table");
    assert!((raw - 0.8).abs() < f32::EPSILON);

    let batch = vec![(0.5, "table_a"), (0.9, "table_a"), (0.3, "table_a")];
    let calibrated_batch = calibrator.calibrate_batch(batch);
    assert_eq!(calibrated_batch.len(), 3);
    assert!(calibrated_batch[0] < calibrated_batch[1]);
}

// ============================================================
// T5: 自适应背压
// ============================================================

#[tokio::test]
async fn test_adaptive_backpressure_full_cycle() {
    let config = SlidingWindowConfig {
        backpressure_threshold: 0.05,
        alert_threshold: 0.15,
        ..Default::default()
    };
    let controller = AdaptiveBackpressureController::new(config, 10);

    assert_eq!(controller.suggested_concurrency(), 10);
    assert!(!controller.is_backpressure_triggered());

    for _ in 0..15 {
        controller.record_compensation(true);
    }
    for _ in 0..5 {
        controller.record_compensation(false);
    }

    assert!(controller.is_backpressure_triggered());
    assert!(controller.is_alert_triggered());
    assert!(controller.suggested_concurrency() < 10);

    let snapshot = controller.snapshot();
    assert!(snapshot.compensation_hit_rate > 0.5);
    assert!(snapshot.backpressure_triggered);
}

// ============================================================
// T6: 对账 Worker
// ============================================================

#[tokio::test]
async fn test_reconciliation_ghost_detection() {
    let logic = Arc::new(InMemoryLogicStorage::new("test-ls"));
    let outbox = Arc::new(InMemoryOutboxProcessor::new());

    logic
        .write_with_seq(
            "outbox",
            serde_json::json!({"source_id": "rec-1", "lance_row_id": "lance-1"}),
        )
        .await
        .unwrap();

    logic
        .write_with_seq("outbox", serde_json::json!({"source_id": "rec-2"}))
        .await
        .unwrap();

    let worker = ReconciliationWorker::new(logic, outbox);
    let (stats, cursor) = worker.reconcile(&ReconciliationCursor::default()).await.unwrap();

    assert_eq!(stats.records_checked, 2);
    assert_eq!(stats.ghost_references_found, 1);
    assert!(cursor.last_seq_id > 0);
}

// ============================================================
// T7: 渐进式路由
// ============================================================

#[tokio::test]
async fn test_progressive_router_fallback() {
    let router = ProgressiveVectorRouter::new(1);

    let v0 = Arc::new(InMemoryVectorStorage::new("v0"));
    v0.initialize(3).await.unwrap();
    v0.insert(vec![make_vector_record("old-1", vec![0.5, 0.5, 0.0], 1)])
        .await
        .unwrap();
    router.register_table(0, v0);

    let v1 = Arc::new(InMemoryVectorStorage::new("v1"));
    v1.initialize(3).await.unwrap();
    v1.insert(vec![make_vector_record("new-1", vec![1.0, 0.0, 0.0], 2)])
        .await
        .unwrap();
    router.register_table(1, v1);

    let query = VectorQuery {
        vector: vec![0.5, 0.5, 0.0],
        top_k: 10,
        filter: None,
        embedding_version: None,
    };
    let results = router.search(query).await.unwrap();
    assert!(results.len() >= 2);

    assert_eq!(router.current_version(), 1);
    router.upgrade_version(2);
    assert_eq!(router.current_version(), 2);
}

// ============================================================
// T8: 图存储
// ============================================================

#[tokio::test]
async fn test_graph_storage_relationships() {
    let graph = GraphStorage::new();

    graph.add_edge(GraphEdge {
        from_id: "doc-a".to_string(),
        to_id: "doc-b".to_string(),
        relationship: "references".to_string(),
        weight: 1.0,
    });
    graph.add_edge(GraphEdge {
        from_id: "doc-b".to_string(),
        to_id: "doc-c".to_string(),
        relationship: "references".to_string(),
        weight: 0.8,
    });
    graph.add_edge(GraphEdge {
        from_id: "doc-c".to_string(),
        to_id: "doc-d".to_string(),
        relationship: "cites".to_string(),
        weight: 0.6,
    });

    let n1 = graph.neighbors_n_degree("doc-a", 1);
    assert_eq!(n1.len(), 1);
    assert_eq!(n1[0].0, "doc-b");

    let n2 = graph.neighbors_n_degree("doc-a", 2);
    assert_eq!(n2.len(), 2);

    let n3 = graph.neighbors_n_degree("doc-a", 3);
    assert_eq!(n3.len(), 3);

    assert_eq!(graph.edge_count(), 3);
}

// ============================================================
// T9: Prompt Hint
// ============================================================

#[test]
fn test_prompt_hint_enrichment() {
    let base = "Search documents";
    let hints = vec![
        PromptHint {
            category: "Usage".to_string(),
            constraint: "Only for semantic search".to_string(),
        },
        PromptHint {
            category: "Limitation".to_string(),
            constraint: "Max 1000 results".to_string(),
        },
    ];

    let enriched = tool_description_with_hint(base, &hints);
    assert!(enriched.contains("**Constraints**"));
    assert!(enriched.contains("Only for semantic search"));
    assert!(enriched.contains("Max 1000 results"));

    let plain = tool_description_with_hint(base, &[]);
    assert_eq!(plain, base);

    let search_hints = prompt_hint::hints::search_hints();
    assert_eq!(search_hints.len(), 2);
    let extract_hints = prompt_hint::hints::extract_hints();
    assert_eq!(extract_hints.len(), 2);
}

// ============================================================
// T10: 置信度拦截器 (熔断器)
// ============================================================

#[tokio::test]
async fn test_confidence_interceptor_circuit_breaker() {
    let interceptor = ConfidenceInterceptor::new(ConfidenceInterceptorConfig {
        min_confidence: 0.5,
        failure_threshold: 3,
        recovery_timeout_secs: 3600,
    });

    assert!(!interceptor.is_circuit_open());
    assert_eq!(
        interceptor.circuit_state(),
        pdf_common::confidence_interceptor::ConfidenceCircuitState::Closed
    );

    assert!(interceptor.check_confidence(0.8));
    assert!(!interceptor.check_confidence(0.3));

    interceptor.record_failure();
    interceptor.record_failure();
    assert_eq!(
        interceptor.circuit_state(),
        pdf_common::confidence_interceptor::ConfidenceCircuitState::HalfOpen
    );
    interceptor.record_failure();
    assert!(interceptor.is_circuit_open());
    assert_eq!(
        interceptor.circuit_state(),
        pdf_common::confidence_interceptor::ConfidenceCircuitState::Open
    );

    interceptor.record_success();
    assert!(!interceptor.is_circuit_open());
    assert_eq!(interceptor.consecutive_failures(), 0);
}

// ============================================================
// T11: 向量 GC
// ============================================================

#[tokio::test]
async fn test_vector_gc_idle_detection() {
    let tracker = Arc::new(QueryTracker::new());
    let worker = VectorGcWorker::new(
        VectorGcConfig {
            idle_days_threshold: 0,
            check_interval_secs: 60,
        },
        tracker.clone(),
    );

    assert!(!worker.should_gc("never-queried"));

    tracker.record_query("recently-queried");
    assert!(worker.should_gc("recently-queried"));
}

// ============================================================
// T12: AST Chunker
// ============================================================

#[tokio::test]
async fn test_ast_chunker_various_inputs() {
    let chunker = AstChunker::with_defaults();
    let chunks = chunker.chunk("Hello world").await;
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, "Hello world");

    let chunks = chunker.chunk("").await;
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].content.is_empty());

    let chunker_no_overlap = AstChunker::new(AstChunkerConfig {
        max_chunk_size: 10,
        overlap_size: 0,
        fallback_enabled: true,
    });
    let long_text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let chunks = chunker_no_overlap.chunk(long_text).await;
    assert!(chunks.len() > 1);
    for chunk in &chunks {
        assert!(chunk.content.len() <= 10);
    }
    let reconstructed: String = chunks.iter().map(|c| c.content.as_str()).collect();
    assert_eq!(reconstructed, long_text);

    let chunker_with_overlap = AstChunker::new(AstChunkerConfig {
        max_chunk_size: 10,
        overlap_size: 3,
        fallback_enabled: true,
    });
    let chunks = chunker_with_overlap.chunk(long_text).await;
    assert!(chunks.len() > 1);
    for i in 1..chunks.len() {
        assert!(chunks[i].start_offset < chunks[i - 1].end_offset);
    }
}

// ============================================================
// T13: 补偿服务端到端
// ============================================================

#[tokio::test]
async fn test_compensation_end_to_end() {
    let outbox = Arc::new(InMemoryOutboxProcessor::new());

    let records: Vec<OutboxRecord> = (0..5)
        .map(|i| {
            let r = OutboxRecord::new(i, "etl_results", format!("rec-{}", i), 3);
            outbox.push(r.clone());
            r
        })
        .collect();

    let service = CompensationService::new(outbox.clone());
    let results = service.compensate_batch(&records).await;
    assert_eq!(results.len(), 5);
    assert!(results.iter().all(|r| r.success));

    let pending = service.fetch_pending_compensation(10).await;
    assert!(pending.is_empty());
}

// ============================================================
// T14: 事务操作
// ============================================================

#[tokio::test]
async fn test_logic_storage_transactions() {
    let ls = InMemoryLogicStorage::new("test-tx");

    let ops = vec![
        TransactionOp::Create {
            table: "users".to_string(),
            data: serde_json::json!({"name": "Alice", "age": 30}),
        },
        TransactionOp::Create {
            table: "users".to_string(),
            data: serde_json::json!({"name": "Bob", "age": 25}),
        },
    ];
    let results = ls.execute_transaction(ops).await.unwrap();
    assert_eq!(results.len(), 2);

    let records = ls.query("users", None).await.unwrap();
    assert_eq!(records.len(), 2);

    let id_to_delete = results[0].get("_id").and_then(|v| v.as_str()).unwrap().to_string();
    let ops2 = vec![TransactionOp::Delete { id: id_to_delete }];
    ls.execute_transaction(ops2).await.unwrap();

    let records = ls.query("users", None).await.unwrap();
    assert_eq!(records.len(), 1);
}

// ============================================================
// T15: 流式插入
// ============================================================

#[tokio::test]
async fn test_vector_stream_insert() {
    use futures::stream;

    let vs = InMemoryVectorStorage::new("test-stream");
    vs.initialize(3).await.unwrap();

    let records = (0..5).map(|i| {
        make_vector_record(&format!("s-{}", i), vec![1.0, 0.0, 0.0], i)
    });

    let count = vs.insert_stream(Box::pin(stream::iter(records))).await.unwrap();
    assert_eq!(count, 5);

    let stats = vs.stats().await.unwrap();
    assert_eq!(stats.total_vectors, 5);
}

// ============================================================
// T16: RBAC Token 验证 + 记录过滤
// ============================================================

#[tokio::test]
async fn test_rbac_token_and_filter() {
    let checker = InMemoryRbacChecker::new();

    let user = make_user("admin", AccessLevel::Confidential);
    checker.register_token("valid-token-123".to_string(), user.clone());

    let ctx = checker.get_user_context("valid-token-123").await.unwrap();
    assert_eq!(ctx.user_id, "user-1");
    assert_eq!(ctx.max_access_level, AccessLevel::Confidential);

    let err = checker.get_user_context("invalid-token").await;
    assert!(err.is_err());

    let records = vec![
        serde_json::json!({"id": "1", "access_level": "public"}),
        serde_json::json!({"id": "2", "access_level": "internal"}),
        serde_json::json!({"id": "3", "access_level": "confidential"}),
        serde_json::json!({"id": "4", "access_level": "secret"}),
    ];
    let filtered = checker.filter_accessible(&user, records).await.unwrap();
    assert_eq!(filtered.len(), 3);
}

// ============================================================
// T17: Outbox 状态机完整生命周期
// ============================================================

#[tokio::test]
async fn test_outbox_state_machine() {
    let mut record = OutboxRecord::new(1, "etl", "rec-1", 2);

    assert_eq!(record.status, OutboxStatus::Pending);
    assert!(record.can_retry());

    record.mark_processing();
    assert_eq!(record.status, OutboxStatus::Processing);

    record.mark_failed("error 1");
    assert_eq!(record.status, OutboxStatus::Failed);
    assert_eq!(record.retry_count, 1);
    assert!(record.can_retry());

    record.mark_processing();
    record.mark_failed("error 2");
    assert_eq!(record.status, OutboxStatus::TerminalFailed);
    assert_eq!(record.retry_count, 2);
    assert!(!record.can_retry());

    let mut record2 = OutboxRecord::new(2, "etl", "rec-2", 3);
    record2.mark_terminal_failed("permanent error");
    assert_eq!(record2.status, OutboxStatus::TerminalFailed);
    assert!(!record2.can_retry());
}
