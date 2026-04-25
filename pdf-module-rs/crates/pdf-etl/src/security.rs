//! 安全验证模块
//!
//! 提供输入验证和安全检查功能，防止 SQL 注入等安全漏洞

use regex::Regex;
use std::sync::LazyLock;

use crate::error::{EtlError, Result};

/// 合法标识符正则表达式（只允许字母、数字、下划线）
static IDENTIFIER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap());

/// 合法表名正则表达式（允许字母、数字、下划线、连字符）
static TABLE_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]{0,63}$").unwrap());

/// 验证表名是否安全
///
/// 规则：
/// - 以字母或下划线开头
/// - 只包含字母、数字、下划线、连字符
/// - 长度不超过 64 个字符
/// - 不能是 SQL 关键字
pub fn validate_table_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(EtlError::ValidationError(
            "Table name cannot be empty".to_string(),
            vec![],
        ));
    }

    if name.len() > 64 {
        return Err(EtlError::ValidationError(
            "Table name too long (max 64 characters)".to_string(),
            vec![],
        ));
    }

    if !TABLE_NAME_REGEX.is_match(name) {
        return Err(EtlError::ValidationError(
            format!("Invalid table name: '{}'. Must start with letter or underscore, contain only alphanumeric, underscore, or hyphen", name),
            vec![],
        ));
    }

    // 检查 SQL 关键字
    if is_sql_keyword(name) {
        return Err(EtlError::ValidationError(
            format!("'{}' is a reserved SQL keyword", name),
            vec![],
        ));
    }

    Ok(())
}

/// 验证 JSON 字段名是否安全
///
/// 规则：
/// - 以字母或下划线开头
/// - 只包含字母、数字、下划线
/// - 长度不超过 128 个字符
pub fn validate_json_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(EtlError::ValidationError(
            "JSON key cannot be empty".to_string(),
            vec![],
        ));
    }

    if key.len() > 128 {
        return Err(EtlError::ValidationError(
            "JSON key too long (max 128 characters)".to_string(),
            vec![],
        ));
    }

    if !IDENTIFIER_REGEX.is_match(key) {
        return Err(EtlError::ValidationError(
            format!("Invalid JSON key: '{}'. Must start with letter or underscore, contain only alphanumeric or underscore", key),
            vec![],
        ));
    }

    Ok(())
}

/// 验证 Schema 名称是否安全
pub fn validate_schema_name(name: &str) -> Result<()> {
    validate_identifier(name, "Schema name")
}

/// 验证通用标识符
pub fn validate_identifier(name: &str, context: &str) -> Result<()> {
    if name.is_empty() {
        return Err(EtlError::ValidationError(
            format!("{} cannot be empty", context),
            vec![],
        ));
    }

    if name.len() > 64 {
        return Err(EtlError::ValidationError(
            format!("{} too long (max 64 characters)", context),
            vec![],
        ));
    }

    if !IDENTIFIER_REGEX.is_match(name) {
        return Err(EtlError::ValidationError(
            format!(
                "Invalid {}: '{}'. Must be a valid identifier",
                context, name
            ),
            vec![],
        ));
    }

    Ok(())
}

/// 检查是否是 SQL 关键字
fn is_sql_keyword(name: &str) -> bool {
    let upper = name.to_uppercase();
    matches!(
        upper.as_str(),
        // 数据定义
        "SELECT" | "INSERT" | "UPDATE" | "DELETE" | "CREATE" | "DROP" | "ALTER" | "TRUNCATE"
        // 数据控制
        | "GRANT" | "REVOKE"
        // 事务控制
        | "COMMIT" | "ROLLBACK" | "SAVEPOINT"
        // 访问控制
        | "WHERE" | "FROM" | "JOIN" | "ON" | "AND" | "OR" | "IN" | "EXISTS"
        // 类型
        | "TABLE" | "INDEX" | "VIEW" | "DATABASE" | "SCHEMA" | "COLUMN"
        // 约束
        | "PRIMARY" | "FOREIGN" | "KEY" | "UNIQUE" | "CHECK" | "DEFAULT" | "NULL" | "NOT"
        // 类型关键字
        | "INT" | "INTEGER" | "VARCHAR" | "TEXT" | "BOOLEAN" | "DATE" | "TIME" | "TIMESTAMP"
        | "JSON" | "JSONB" | "UUID" | "FLOAT" | "DOUBLE" | "DECIMAL" | "NUMERIC"
        // 函数
        | "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "CAST" | "COALESCE" | "NULLIF"
        // 其他
        | "ORDER" | "GROUP" | "HAVING" | "LIMIT" | "OFFSET" | "AS" | "DISTINCT" | "ALL"
        | "UNION" | "INTERSECT" | "EXCEPT" | "CASE" | "WHEN" | "THEN" | "ELSE" | "END"
        | "RETURNING" | "VALUES" | "SET" | "INTO" | "WITH" | "RECURSIVE"
    )
}

/// 转义 SQL 标识符（用于 PostgreSQL）
pub fn quote_identifier_postgres(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

/// 转义 SQL 标识符（用于 MySQL）
pub fn quote_identifier_mysql(ident: &str) -> String {
    format!("`{}`", ident.replace('`', "``"))
}

/// 转义 SQL 标识符（用于 SQLite）
pub fn quote_identifier_sqlite(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_table_name() {
        // 有效表名
        assert!(validate_table_name("users").is_ok());
        assert!(validate_table_name("user_profiles").is_ok());
        assert!(validate_table_name("table123").is_ok());
        assert!(validate_table_name("_private").is_ok());

        // 无效表名
        assert!(validate_table_name("").is_err());
        assert!(validate_table_name("123table").is_err());
        assert!(validate_table_name("user-table").is_ok()); // 连字符允许
        assert!(validate_table_name("user table").is_err());
        assert!(validate_table_name("SELECT").is_err()); // SQL 关键字
    }

    #[test]
    fn test_validate_json_key() {
        // 有效 key
        assert!(validate_json_key("name").is_ok());
        assert!(validate_json_key("user_id").is_ok());
        assert!(validate_json_key("_private").is_ok());

        // 无效 key
        assert!(validate_json_key("").is_err());
        assert!(validate_json_key("123key").is_err());
        assert!(validate_json_key("user-name").is_err()); // 连字符不允许
        assert!(validate_json_key("user name").is_err());
    }

    #[test]
    fn test_sql_keywords() {
        assert!(is_sql_keyword("SELECT"));
        assert!(is_sql_keyword("select"));
        assert!(is_sql_keyword("INSERT"));
        assert!(is_sql_keyword("TABLE"));
        assert!(!is_sql_keyword("users"));
        assert!(!is_sql_keyword("data"));
    }

    #[test]
    fn test_quote_identifier() {
        assert_eq!(quote_identifier_postgres("users"), "\"users\"");
        assert_eq!(quote_identifier_postgres("user\"name"), "\"user\"\"name\"");
        assert_eq!(quote_identifier_mysql("users"), "`users`");
        assert_eq!(quote_identifier_mysql("user`name"), "`user``name`");
    }
}
