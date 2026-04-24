//! Schema 定义
//!
//! 基于 schemars 实现 JSON Schema 的定义和派生

use schemars::{schema::RootSchema, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::{EtlError, Result};

/// Schema 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    /// Schema 名称
    pub name: String,
    /// JSON Schema
    #[serde(with = "root_schema_serde")]
    pub schema: RootSchema,
    /// Rust 类型名称（可选）
    pub rust_type: Option<String>,
}

/// RootSchema 序列化/反序列化辅助模块
mod root_schema_serde {
    use schemars::schema::RootSchema;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(schema: &RootSchema, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_json::to_value(schema).unwrap().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<RootSchema, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        serde_json::from_value(value).map_err(serde::de::Error::custom)
    }
}

impl SchemaDefinition {
    /// 从 Rust struct 派生 Schema
    pub fn from_struct<T: JsonSchema>() -> Self {
        let schema = schemars::schema_for!(T);
        let name = schema
            .schema
            .metadata
            .as_ref()
            .and_then(|m| m.title.clone())
            .unwrap_or_else(|| {
                std::any::type_name::<T>()
                    .split("::")
                    .last()
                    .unwrap_or("Unknown")
                    .to_string()
            });

        Self {
            name,
            schema,
            rust_type: Some(std::any::type_name::<T>().to_string()),
        }
    }

    /// 从 JSON Schema 字符串解析
    pub fn from_json(schema_str: &str) -> Result<Self> {
        let schema: RootSchema = serde_json::from_str(schema_str)
            .map_err(|e| EtlError::SchemaError(format!("Failed to parse JSON Schema: {}", e)))?;

        let name = schema
            .schema
            .metadata
            .as_ref()
            .and_then(|m| m.title.clone())
            .unwrap_or_else(|| "DynamicSchema".to_string());

        Ok(Self {
            name,
            schema,
            rust_type: None,
        })
    }

    /// 从 JSON Value 解析
    pub fn from_value(value: &serde_json::Value) -> Result<Self> {
        let schema: RootSchema = serde_json::from_value(value.clone())
            .map_err(|e| EtlError::SchemaError(format!("Failed to parse JSON Schema: {}", e)))?;

        let name = schema
            .schema
            .metadata
            .as_ref()
            .and_then(|m| m.title.clone())
            .unwrap_or_else(|| "DynamicSchema".to_string());

        Ok(Self {
            name,
            schema,
            rust_type: None,
        })
    }

    /// 输出格式化的 JSON Schema 字符串
    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.schema)
            .map_err(|e| EtlError::SchemaError(format!("Failed to serialize schema: {}", e)))
    }

    /// 获取 Schema 的 JSON Value
    pub fn to_json_value(&self) -> Result<serde_json::Value> {
        serde_json::to_value(&self.schema)
            .map_err(|e| EtlError::SchemaError(format!("Failed to serialize schema: {}", e)))
    }
}

// ============================================================================
// 示例 Schema 定义
// ============================================================================

/// 发票 Schema
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InvoiceSchema {
    /// 发票编号
    pub invoice_number: String,
    /// 发票日期
    pub date: String,
    /// 供应商信息
    pub vendor: VendorInfo,
    /// 发票项目列表
    pub items: Vec<InvoiceItem>,
    /// 总金额
    pub total_amount: f64,
    /// 货币
    pub currency: String,
}

/// 供应商信息
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VendorInfo {
    /// 名称
    pub name: String,
    /// 地址
    pub address: Option<String>,
    /// 税号
    pub tax_id: Option<String>,
}

/// 发票项目
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InvoiceItem {
    /// 描述
    pub description: String,
    /// 数量
    pub quantity: u32,
    /// 单价
    pub unit_price: f64,
    /// 金额
    pub amount: f64,
}

/// 简历 Schema
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResumeSchema {
    /// 姓名
    pub name: String,
    /// 邮箱
    pub email: String,
    /// 电话
    pub phone: Option<String>,
    /// 工作经历
    pub work_experience: Vec<WorkExperience>,
    /// 教育经历
    pub education: Vec<Education>,
    /// 技能
    pub skills: Vec<String>,
}

/// 工作经历
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkExperience {
    /// 公司
    pub company: String,
    /// 职位
    pub position: String,
    /// 开始日期
    pub start_date: String,
    /// 结束日期
    pub end_date: Option<String>,
    /// 描述
    pub description: Option<String>,
}

/// 教育经历
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Education {
    /// 学校
    pub school: String,
    /// 学位
    pub degree: String,
    /// 专业
    pub major: Option<String>,
    /// 毕业年份
    pub graduation_year: Option<u32>,
}

/// 合同 Schema
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractSchema {
    /// 合同编号
    pub contract_number: String,
    /// 合同标题
    pub title: String,
    /// 甲方
    pub party_a: PartyInfo,
    /// 乙方
    pub party_b: PartyInfo,
    /// 签订日期
    pub sign_date: String,
    /// 生效日期
    pub effective_date: String,
    /// 到期日期
    pub expiry_date: Option<String>,
    /// 合同金额
    pub amount: Option<f64>,
    /// 合同条款
    pub terms: Vec<String>,
}

/// 合同方信息
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartyInfo {
    /// 名称
    pub name: String,
    /// 地址
    pub address: Option<String>,
    /// 联系人
    pub contact: Option<String>,
    /// 联系方式
    pub contact_info: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_from_struct() {
        let schema = SchemaDefinition::from_struct::<InvoiceSchema>();
        assert_eq!(schema.name, "InvoiceSchema");
        assert!(schema.rust_type.is_some());
    }

    #[test]
    fn test_schema_to_json_string() {
        let schema = SchemaDefinition::from_struct::<InvoiceSchema>();
        let json = schema.to_json_string().unwrap();
        assert!(json.contains("invoice_number"));
        assert!(json.contains("InvoiceSchema"));
    }

    #[test]
    fn test_invoice_schema() {
        let invoice = InvoiceSchema {
            invoice_number: "INV-001".to_string(),
            date: "2024-01-01".to_string(),
            vendor: VendorInfo {
                name: "Test Vendor".to_string(),
                address: Some("Test Address".to_string()),
                tax_id: None,
            },
            items: vec![InvoiceItem {
                description: "Test Item".to_string(),
                quantity: 1,
                unit_price: 100.0,
                amount: 100.0,
            }],
            total_amount: 100.0,
            currency: "USD".to_string(),
        };
        let json = serde_json::to_string(&invoice).unwrap();
        assert!(json.contains("INV-001"));
    }

    #[test]
    fn test_resume_schema() {
        let schema = SchemaDefinition::from_struct::<ResumeSchema>();
        assert_eq!(schema.name, "ResumeSchema");
    }

    #[test]
    fn test_contract_schema() {
        let schema = SchemaDefinition::from_struct::<ContractSchema>();
        assert_eq!(schema.name, "ContractSchema");
    }
}
