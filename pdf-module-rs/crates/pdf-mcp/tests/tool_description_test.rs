//! Tool description validation tests
//! Ensures all tool descriptions are detailed and helpful for AI agents

use serde_json::Value;

/// Get all tool definitions (simulating tools/list response)
fn get_all_tools() -> Vec<Value> {
    vec![
        // Tool 1: extract_text
        serde_json::json!({
            "name": "extract_text",
            "description": include_str!("../descriptions/extract_text.md"),
            "input_schema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件的绝对路径。路径必须存在且可读。支持的安全检查：禁止路径遍历（../）、符号链接验证。"
                    },
                    "adapter": {
                        "type": "string",
                        "description": "指定提取引擎。不指定时使用智能路由自动选择。可选值：lopdf（布局感知）、pdf-extract（快速）、pdfium（高兼容）",
                        "enum": ["lopdf", "pdf-extract", "pdfium", "pymupdf", "pdfplumber"]
                    }
                },
                "required": ["file_path"]
            }
        }),
        // Tool 2: extract_structured
        serde_json::json!({
            "name": "extract_structured",
            "description": include_str!("../descriptions/extract_structured.md"),
            "input_schema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件的绝对路径"
                    },
                    "adapter": {
                        "type": "string",
                        "description": "提取引擎（推荐 lopdf 以获得最佳位置精度）"
                    },
                    "enable_highlight": {
                        "type": "boolean",
                        "description": "是否包含高亮元数据（用于前端渲染）。默认 false。"
                    }
                },
                "required": ["file_path"]
            }
        }),
        // Tool 3: search_keywords
        serde_json::json!({
            "name": "search_keywords",
            "description": include_str!("../descriptions/search_keywords.md"),
            "input_schema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件的绝对路径"
                    },
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "要搜索的关键词列表。支持正则表达式特殊字符（会被自动转义）。"
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "是否区分大小写。默认 false（不区分）。"
                    },
                    "context_length": {
                        "type": "integer",
                        "description": "匹配上下文的字符数（前后各取 N 个字符）。默认 50。建议范围：30-200。",
                        "default": 50,
                        "minimum": 10,
                        "maximum": 500
                    }
                },
                "required": ["file_path", "keywords"]
            }
        }),
        // Tool 4: extract_keywords
        serde_json::json!({
            "name": "extract_keywords",
            "description": include_str!("../descriptions/extract_keywords.md"),
            "input_schema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件的绝对路径"
                    },
                    "top_n": {
                        "type": "integer",
                        "description": "返回前 N 个高频关键词。默认 10。建议范围：5-50。",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 100
                    }
                },
                "required": ["file_path"]
            }
        }),
        // Tool 5: get_page_count
        serde_json::json!({
            "name": "get_page_count",
            "description": include_str!("../descriptions/get_page_count.md"),
            "input_schema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件的绝对路径"
                    }
                },
                "required": ["file_path"]
            }
        }),
        // Tool 6: list_adapters
        serde_json::json!({
            "name": "list_adapters",
            "description": include_str!("../descriptions/list_adapters.md"),
            "input_schema": {
                "type": "object",
                "properties": {}
            }
        }),
        // Tool 7: cache_stats
        serde_json::json!({
            "name": "cache_stats",
            "description": include_str!("../descriptions/cache_stats.md"),
            "input_schema": {
                "type": "object",
                "properties": {}
            }
        }),
    ]
}

#[test]
fn test_tool_descriptions_not_empty() {
    let tools = get_all_tools();
    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        let description = tool["description"].as_str().unwrap();

        assert!(
            !description.is_empty(),
            "Tool {} has empty description",
            name
        );
    }
}

#[test]
fn test_tool_descriptions_minimum_length() {
    let tools = get_all_tools();
    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        let description = tool["description"].as_str().unwrap();

        // Descriptions should be at least 100 characters for meaningful content
        assert!(
            description.len() > 100,
            "Tool {} description too short ({} chars), should be > 100",
            name,
            description.len()
        );
    }
}

#[test]
fn test_parameter_descriptions_not_empty() {
    let tools = get_all_tools();
    for tool in tools {
        let tool_name = tool["name"].as_str().unwrap();
        let props = tool["input_schema"]["properties"].as_object().unwrap();

        for (param_name, param_def) in props {
            let desc = param_def["description"].as_str();
            assert!(
                desc.is_some(),
                "Parameter {} in tool {} has no description",
                param_name,
                tool_name
            );

            let desc_str = desc.unwrap();
            assert!(
                desc_str.len() > 10,
                "Parameter {} in tool {} description too short ({} chars)",
                param_name,
                tool_name,
                desc_str.len()
            );
        }
    }
}

#[test]
fn test_extract_text_description_quality() {
    let tools = get_all_tools();
    let extract_text = tools
        .iter()
        .find(|t| t["name"] == "extract_text")
        .expect("extract_text tool not found");

    let description = extract_text["description"].as_str().unwrap();

    // Should mention engines
    assert!(
        description.contains("引擎") || description.contains("engine"),
        "extract_text description should mention engines"
    );

    // Should mention specific engines
    assert!(
        description.contains("lopdf"),
        "extract_text description should mention lopdf"
    );
    assert!(
        description.contains("pdf-extract"),
        "extract_text description should mention pdf-extract"
    );
    assert!(
        description.contains("pdfium"),
        "extract_text description should mention pdfium"
    );

    // Should mention performance
    assert!(
        description.contains("性能") || description.contains("performance"),
        "extract_text description should mention performance"
    );

    // Should mention limitations
    assert!(
        description.contains("限制") || description.contains("不支持"),
        "extract_text description should mention limitations"
    );
}

#[test]
fn test_search_keywords_description_quality() {
    let tools = get_all_tools();
    let search_keywords = tools
        .iter()
        .find(|t| t["name"] == "search_keywords")
        .expect("search_keywords tool not found");

    let description = search_keywords["description"].as_str().unwrap();

    // Should mention algorithm
    assert!(
        description.contains("算法") || description.contains("algorithm"),
        "search_keywords description should mention algorithm"
    );

    // Should mention regex
    assert!(
        description.contains("正则") || description.contains("regex"),
        "search_keywords description should mention regex"
    );

    // Should mention performance
    assert!(
        description.contains("性能") || description.contains("缓存"),
        "search_keywords description should mention performance/caching"
    );
}

#[test]
fn test_extract_keywords_description_quality() {
    let tools = get_all_tools();
    let extract_keywords = tools
        .iter()
        .find(|t| t["name"] == "extract_keywords")
        .expect("extract_keywords tool not found");

    let description = extract_keywords["description"].as_str().unwrap();

    // Should mention segmentation/tokenization
    assert!(
        description.contains("分词") || description.contains("segmentation"),
        "extract_keywords description should mention segmentation"
    );

    // Should mention Jieba for Chinese
    assert!(
        description.contains("Jieba") || description.contains("结巴"),
        "extract_keywords description should mention Jieba for Chinese support"
    );

    // Should mention Chinese support
    assert!(
        description.contains("中文") || description.contains("Chinese"),
        "extract_keywords description should mention Chinese support"
    );
}

#[test]
fn test_all_tools_have_required_parameters() {
    let tools = get_all_tools();
    for tool in tools {
        let tool_name = tool["name"].as_str().unwrap();
        let required = tool["input_schema"]["required"].as_array();

        if let Some(required_params) = required {
            let props = tool["input_schema"]["properties"].as_object().unwrap();

            // All required parameters should exist in properties
            for req_param in required_params {
                let param_name = req_param.as_str().unwrap();
                assert!(
                    props.contains_key(param_name),
                    "Tool {} has required parameter '{}' not defined in properties",
                    tool_name,
                    param_name
                );
            }
        }
    }
}

#[test]
fn test_parameter_enum_values_valid() {
    let tools = get_all_tools();
    for tool in tools {
        let tool_name = tool["name"].as_str().unwrap();
        let props = tool["input_schema"]["properties"].as_object().unwrap();

        for (param_name, param_def) in props {
            if let Some(enum_values) = param_def.get("enum") {
                let enum_array = enum_values.as_array().expect(&format!(
                    "Enum for {} in {} is not an array",
                    param_name, tool_name
                ));

                assert!(
                    !enum_array.is_empty(),
                    "Enum for {} in {} is empty",
                    param_name,
                    tool_name
                );
            }
        }
    }
}

#[test]
fn test_description_contains_sections() {
    let tools = get_all_tools();

    // Tools with complex functionality should have structured descriptions
    let complex_tools = vec![
        "extract_text",
        "extract_structured",
        "search_keywords",
        "extract_keywords",
    ];

    for tool_name in complex_tools {
        let tool = tools
            .iter()
            .find(|t| t["name"] == tool_name)
            .expect(&format!("{} tool not found", tool_name));

        let description = tool["description"].as_str().unwrap();

        // Should have multiple sections (indicated by ## or **)
        let section_count = description.matches("##").count() + description.matches("**").count();
        assert!(
            section_count >= 3,
            "Tool {} description should have at least 3 sections, found {}",
            tool_name,
            section_count
        );
    }
}
