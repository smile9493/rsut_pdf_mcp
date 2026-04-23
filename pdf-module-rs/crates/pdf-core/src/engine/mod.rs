//! PDF engine abstraction and implementations
//! Corresponds to Python: adapters/

mod r#trait;
mod lopdf;
mod pdf_extract;
mod pdfium;

pub use r#trait::PdfEngine;
pub use lopdf::LopdfEngine;
pub use pdf_extract::PdfExtractEngine;
pub use pdfium::PdfiumEngine;
