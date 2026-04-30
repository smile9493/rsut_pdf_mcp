//! Zero-copy slice wrapper for JS-WASM boundary data passing.
//!
//! Avoids serialization overhead by passing raw pointers with length
//! across the JS-WASM boundary. This is critical for large PDF images
//! and extracted text buffers.

use std::marker::PhantomData;

/// Zero-copy slice wrapper for JS-WASM boundary.
///
/// Avoids serialization overhead by passing raw pointers with length.
/// This enables passing large buffers (rendered pages, extracted text)
/// without copying.
///
/// # Safety
///
/// The caller must ensure the pointer and length are valid for the
/// lifetime of the `WasmSlice`. The slice must not be used after
/// the underlying memory is freed.
///
/// Prefer [`OwnedSlice`] for safe ownership management.
pub struct WasmSlice {
    ptr: *const u8,
    len: usize,
    _marker: PhantomData<*const [u8]>,
}

// SAFETY: WasmSlice is designed for controlled JS-WASM interop.
// The caller must ensure thread safety and lifetime validity.
unsafe impl Send for WasmSlice {}
unsafe impl Sync for WasmSlice {}

impl WasmSlice {
    /// Create a new WasmSlice from a pointer and length.
    ///
    /// # Safety
    ///
    /// The caller must ensure `ptr` is valid for `len` bytes and
    /// remains valid for the lifetime of the returned `WasmSlice`.
    /// The underlying memory must not be freed while this slice exists.
    pub unsafe fn new(ptr: *const u8, len: usize) -> Self {
        Self {
            ptr,
            len,
            _marker: PhantomData,
        }
    }

    /// Convert to Rust slice for internal processing.
    ///
    /// # Safety
    ///
    /// Caller must ensure the pointer and length are valid.
    pub unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.ptr, self.len)
    }

    /// Get the raw pointer for JS interop.
    pub fn ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Get the length for JS interop.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// A safe wrapper that owns data and provides a zero-copy [`WasmSlice`] view.
///
/// This prevents dangling pointer issues by keeping the underlying data alive
/// for the entire lifetime of the `OwnedSlice`. The `WasmSlice` returned by
/// [`as_wasm_slice`] is only valid while the `OwnedSlice` is alive.
///
/// # Example
///
/// ```
/// use pdf_wasm::slice::OwnedSlice;
///
/// let owned = OwnedSlice::from_vec(vec![1, 2, 3, 4, 5]);
/// let wasm_slice = owned.as_wasm_slice();
/// assert_eq!(wasm_slice.len(), 5);
/// // Data stays alive as long as `owned` is alive
/// ```
pub struct OwnedSlice {
    data: Vec<u8>,
}

impl OwnedSlice {
    /// Create an OwnedSlice from a byte vector.
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get a zero-copy view of the data as a WasmSlice.
    ///
    /// The returned slice is valid for the lifetime of this `OwnedSlice`.
    pub fn as_wasm_slice(&self) -> WasmSlice {
        WasmSlice {
            ptr: self.data.as_ptr(),
            len: self.data.len(),
            _marker: PhantomData,
        }
    }

    /// Get a reference to the underlying data.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the length of the data.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the data is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Consume the OwnedSlice and return the underlying Vec.
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}

impl From<Vec<u8>> for OwnedSlice {
    fn from(data: Vec<u8>) -> Self {
        Self::from_vec(data)
    }
}

impl AsRef<[u8]> for OwnedSlice {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_slice_creation() {
        let data = vec![1u8, 2, 3, 4, 5];
        let owned = OwnedSlice::from_vec(data);
        let slice = owned.as_wasm_slice();
        assert_eq!(slice.len(), 5);
        assert!(!slice.is_empty());
    }

    #[test]
    fn test_wasm_slice_empty() {
        let owned = OwnedSlice::from_vec(vec![]);
        let slice = owned.as_wasm_slice();
        assert!(slice.is_empty());
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn test_wasm_slice_as_slice() {
        let owned = OwnedSlice::from_vec(vec![10, 20, 30]);
        let slice = owned.as_wasm_slice();
        unsafe {
            let inner = slice.as_slice();
            assert_eq!(inner, &[10, 20, 30]);
        }
    }

    #[test]
    fn test_wasm_slice_ptr() {
        let owned = OwnedSlice::from_vec(vec![42u8]);
        let slice = owned.as_wasm_slice();
        assert_eq!(slice.ptr(), owned.data.as_ptr());
    }

    #[test]
    fn test_owned_slice_into_vec() {
        let data = vec![1, 2, 3];
        let owned = OwnedSlice::from_vec(data);
        let recovered = owned.into_vec();
        assert_eq!(recovered, vec![1, 2, 3]);
    }

    #[test]
    fn test_from_trait() {
        let owned: OwnedSlice = vec![1u8, 2, 3].into();
        assert_eq!(owned.len(), 3);
    }

    #[test]
    fn test_as_ref_trait() {
        let owned = OwnedSlice::from_vec(vec![1, 2, 3]);
        let slice: &[u8] = owned.as_ref();
        assert_eq!(slice, &[1, 2, 3]);
    }

    #[test]
    fn test_no_dangling_ptr() {
        // Safety test: WasmSlice stays valid as long as OwnedSlice is alive
        let owned = OwnedSlice::from_vec(vec![100, 200]);
        let wasm_slice = owned.as_wasm_slice();
        unsafe {
            assert_eq!(wasm_slice.as_slice(), &[100, 200]);
        }
        // Drop owned after using wasm_slice - would cause UB if we allowed it
        drop(owned);
    }
}
