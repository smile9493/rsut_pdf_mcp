//! Lopdf engine implementation
//! Corresponds to Python: adapters/pymupdf_adapter.py

use async_trait::async_trait;
use lopdf::Document;
use std::path::Path;

use crate::dto::{
    ExtractOptions, FileInfo, PageMetadata, StructuredExtractionResult, TextExtractionResult,
};
use crate::engine::PdfEngine;
use crate::error::{PdfModuleError, PdfResult};

/// Layout-aware PDF engine based on lopdf
/// Corresponds to Python: PyMuPDFAdapter
pub struct LopdfEngine;

impl LopdfEngine {
    pub fn new() -> Self {
        Self
    }

    /// Parse MediaBox from a page object
    /// MediaBox is typically an array of 4 numbers: [x0, y0, x1, y1]
    fn parse_mediabox(doc: &Document, page_obj_id: u32) -> Option<(f64, f64, f64, f64)> {
        // Get the page object
        let page_obj = doc.get_object((page_obj_id, 0)).ok()?;
        
        // Get the page dictionary
        let page_dict = page_obj.as_dict().ok()?;
        
        // Look for MediaBox in the page object
        if let Ok(mediabox) = page_dict.get(b"MediaBox") {
            return Self::parse_bbox_array(mediabox);
        }
        
        // If not found directly, check if there's a Parent and look up the hierarchy
        if let Ok(parent_ref) = page_dict.get(b"Parent") {
            return Self::find_mediabox_in_parent(doc, parent_ref);
        }
        
        // Default A4 size if nothing found
        None
    }

    /// Parse a bbox array from an Object
    fn parse_bbox_array(obj: &lopdf::Object) -> Option<(f64, f64, f64, f64)> {
        match obj {
            lopdf::Object::Array(arr) => {
                if arr.len() >= 4 {
                    let x0 = Self::object_to_f64(&arr[0])?;
                    let y0 = Self::object_to_f64(&arr[1])?;
                    let x1 = Self::object_to_f64(&arr[2])?;
                    let y1 = Self::object_to_f64(&arr[3])?;
                    Some((x0, y0, x1, y1))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Convert Object to f64
    fn object_to_f64(obj: &lopdf::Object) -> Option<f64> {
        match obj {
            lopdf::Object::Integer(i) => Some(*i as f64),
            lopdf::Object::Real(r) => Some(*r as f64), // f32 to f64
            _ => None,
        }
    }

    /// Find MediaBox in parent objects (page tree hierarchy)
    fn find_mediabox_in_parent(doc: &Document, parent_ref: &lopdf::Object) -> Option<(f64, f64, f64, f64)> {
        // Get the parent object ID from reference
        let parent_id = match parent_ref {
            lopdf::Object::Reference((id, _gen)) => *id,
            _ => return None,
        };
        
        // Get the parent object
        let parent_obj = doc.get_object((parent_id, 0)).ok()?;
        let parent_dict = parent_obj.as_dict().ok()?;
        
        // Check for MediaBox in this parent
        if let Ok(mediabox) = parent_dict.get(b"MediaBox") {
            return Self::parse_bbox_array(mediabox);
        }
        
        // Recursively check parent's parent
        if let Ok(grandparent_ref) = parent_dict.get(b"Parent") {
            return Self::find_mediabox_in_parent(doc, grandparent_ref);
        }
        
        None
    }
}

impl Default for LopdfEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PdfEngine for LopdfEngine {
    fn id(&self) -> &str {
        "lopdf"
    }

    fn name(&self) -> &str {
        "LopdfEngine"
    }

    fn description(&self) -> &str {
        "Layout-aware PDF engine based on lopdf"
    }

    async fn extract_text(&self, file_path: &Path) -> PdfResult<TextExtractionResult> {
        let doc = Document::load(file_path)
            .map_err(|e| PdfModuleError::Extraction(format!("Failed to load PDF: {}", e)))?;

        let mut all_text = String::new();

        // get_pages() returns BTreeMap<u32, (u32, u16)> where:
        // - key is page number (1-based)
        // - value is (object_id, generation_number)
        for (page_num, (obj_id, _gen)) in doc.get_pages() {
            // extract_text takes page object IDs
            if let Ok(text) = doc.extract_text(&[obj_id]) {
                all_text.push_str(&text);
                all_text.push('\n');
            }
            // Avoid unused variable warning
            let _ = page_num;
        }

        Ok(TextExtractionResult {
            extracted_text: all_text.trim().to_string(),
            extraction_metadata: None,
        })
    }

    async fn extract_structured(
        &self,
        file_path: &Path,
        _options: &ExtractOptions,
    ) -> PdfResult<StructuredExtractionResult> {
        let doc = Document::load(file_path)
            .map_err(|e| PdfModuleError::Extraction(format!("Failed to load PDF: {}", e)))?;

        let pages_map = doc.get_pages();
        let page_count = pages_map.len() as u32;
        let mut pages = Vec::with_capacity(page_count as usize);
        let mut all_text = String::new();

        for (page_num, (obj_id, _gen)) in pages_map.iter() {
            let page_text = doc.extract_text(&[*obj_id]).unwrap_or_default();

            // Parse MediaBox for this page
            let bbox = Self::parse_mediabox(&doc, *obj_id);

            let page_meta = PageMetadata {
                page_number: *page_num,
                text: page_text.trim().to_string(),
                bbox,
                lines: vec![], // lopdf line-level info requires Tm parsing
            };

            all_text.push_str(&page_text);
            all_text.push('\n');
            pages.push(page_meta);
        }

        let file_info = FileInfo::from_path(file_path)?;

        Ok(StructuredExtractionResult {
            extracted_text: all_text.trim().to_string(),
            page_count,
            pages,
            extraction_metadata: None,
            file_info,
        })
    }

    async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32> {
        let doc = Document::load(file_path)
            .map_err(|e| PdfModuleError::Extraction(format!("Failed to load PDF: {}", e)))?;
        Ok(doc.get_pages().len() as u32)
    }

    fn test_connection(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_metadata() {
        let engine = LopdfEngine::new();
        assert_eq!(engine.id(), "lopdf");
        assert_eq!(engine.name(), "LopdfEngine");
        assert!(engine.test_connection());
    }
}
