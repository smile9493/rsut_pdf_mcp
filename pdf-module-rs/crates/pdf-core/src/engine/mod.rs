//! PDF engine abstraction and implementations
//! Corresponds to Python: adapters/

mod lopdf;
mod pdf_extract;
mod pdfium;
mod r#trait;

pub use lopdf::LopdfEngine;
pub use pdf_extract::PdfExtractEngine;
pub use pdfium::PdfiumEngine;
pub use r#trait::PdfEngine;
