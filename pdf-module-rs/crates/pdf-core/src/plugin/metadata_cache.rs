//! Metadata cache for tool definitions
//! Provides caching mechanism to accelerate tool metadata queries

use crate::protocol::{RuntimeVariables, ToolDefinition, ToolSpec};
use moka::sync::Cache;
use std::time::Duration;

/// Metadata cache
/// Caches tool definitions, specs, and variables for fast lookup
pub struct MetadataCache {
    /// Tool definition cache
    definitions: Cache<String, ToolDefinition>,
    /// Tool spec cache
    specs: Cache<String, ToolSpec>,
    /// Runtime variables cache
    variables: Cache<String, RuntimeVariables>,
}

impl MetadataCache {
    /// Create a new metadata cache with default TTL (5 minutes)
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(300))
    }

    /// Create a new metadata cache with custom TTL
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            definitions: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(1000)
                .build(),
            specs: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(1000)
                .build(),
            variables: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(1000)
                .build(),
        }
    }

    /// Get a tool definition from cache
    pub fn get_definition(&self, name: &str) -> Option<ToolDefinition> {
        self.definitions.get(name)
    }

    /// Cache a tool definition
    pub fn cache_definition(&self, name: String, definition: ToolDefinition) {
        self.definitions.insert(name, definition);
    }

    /// Get a tool spec from cache
    pub fn get_spec(&self, name: &str) -> Option<ToolSpec> {
        self.specs.get(name)
    }

    /// Cache a tool spec
    pub fn cache_spec(&self, name: String, spec: ToolSpec) {
        self.specs.insert(name, spec);
    }

    /// Get runtime variables from cache
    pub fn get_variables(&self, name: &str) -> Option<RuntimeVariables> {
        self.variables.get(name)
    }

    /// Cache runtime variables
    pub fn cache_variables(&self, name: String, variables: RuntimeVariables) {
        self.variables.insert(name, variables);
    }

    /// Invalidate cache for a specific tool
    pub fn invalidate(&self, name: &str) {
        self.definitions.invalidate(name);
        self.specs.invalidate(name);
        self.variables.invalidate(name);
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.definitions.invalidate_all();
        self.specs.invalidate_all();
        self.variables.invalidate_all();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            definition_count: self.definitions.entry_count(),
            spec_count: self.specs.entry_count(),
            variable_count: self.variables.entry_count(),
        }
    }
}

impl Default for MetadataCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached definitions
    pub definition_count: u64,
    /// Number of cached specs
    pub spec_count: u64,
    /// Number of cached variables
    pub variable_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{InputType, OutputType};

    #[test]
    fn test_metadata_cache_basic() {
        let cache = MetadataCache::new();

        let definition = ToolDefinition::new(
            "Test Tool".to_string(),
            "test_tool".to_string(),
            "A test tool".to_string(),
            vec![],
            InputType::File,
            OutputType::Json,
        );

        // Cache and retrieve
        cache.cache_definition("test_tool".to_string(), definition.clone());
        let retrieved = cache.get_definition("test_tool");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().function_name, "test_tool");

        // Invalidate
        cache.invalidate("test_tool");
        assert!(cache.get_definition("test_tool").is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = MetadataCache::new();

        let definition = ToolDefinition::new(
            "Test Tool".to_string(),
            "test_tool".to_string(),
            "A test tool".to_string(),
            vec![],
            InputType::File,
            OutputType::Json,
        );

        cache.cache_definition("test_tool".to_string(), definition);
        
        // Verify the entry is retrievable (Moka's entry_count is approximate)
        let retrieved = cache.get_definition("test_tool");
        assert!(retrieved.is_some());
        
        let stats = cache.stats();
        assert!(stats.definition_count <= 1);
        assert_eq!(stats.spec_count, 0);
        assert_eq!(stats.variable_count, 0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = MetadataCache::new();

        let definition = ToolDefinition::new(
            "Test Tool".to_string(),
            "test_tool".to_string(),
            "A test tool".to_string(),
            vec![],
            InputType::File,
            OutputType::Json,
        );

        cache.cache_definition("test_tool".to_string(), definition);
        cache.clear();
        
        let stats = cache.stats();
        assert_eq!(stats.definition_count, 0);
    }
}
