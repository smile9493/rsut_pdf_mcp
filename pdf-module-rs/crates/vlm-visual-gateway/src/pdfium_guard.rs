use std::panic::catch_unwind;
use std::sync::Mutex;

use tracing::{error, warn};

use crate::error::{PdfiumGuardError, PdfiumGuardResult};

/// Pdfium FFI safety guard.
///
/// All calls that enter Pdfium must go through `safe_execute`.
/// The internal `Mutex<()>` serialises concurrent access so only one thread
/// ever enters the FFI boundary, and `catch_unwind` prevents panics from
/// crossing the FFI boundary.
pub struct PdfiumGuard {
    lock: Mutex<()>,
}

impl PdfiumGuard {
    pub fn new() -> Self {
        Self {
            lock: Mutex::new(()),
        }
    }

    /// Execute a closure inside the Pdfium serialisation + panic-isolation boundary.
    pub fn safe_execute<F, R>(&self, f: F) -> PdfiumGuardResult<R>
    where
        F: FnOnce() -> R + std::panic::UnwindSafe,
    {
        let guard = self.lock.lock().map_err(|_| {
            warn!("PdfiumGuard mutex poisoned - lock is contaminated");
            PdfiumGuardError::LockPoisoned
        })?;

        let result = catch_unwind(f);

        drop(guard);

        result.map_err(|_| {
            error!("Pdfium FFI call panicked - caught by PdfiumGuard");
            PdfiumGuardError::Panic
        })
    }
}

impl Default for PdfiumGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience: wrap a Pdfium render call with catch_unwind.
///
/// This is the primary public helper that callers should use for one-shot
/// FFI calls without needing to hold a `PdfiumGuard` reference.
pub fn catch_pdfium<F, R>(f: F) -> PdfiumGuardResult<R>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    catch_unwind(f).map_err(|_| {
        error!("Pdfium FFI call panicked");
        PdfiumGuardError::Panic
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_execute_ok() {
        let guard = PdfiumGuard::new();
        let result = guard.safe_execute(|| 42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn safe_execute_catches_panic() {
        let guard = PdfiumGuard::new();
        let result: PdfiumGuardResult<()> = guard.safe_execute(|| {
            panic!("ffi boom");
        });
        assert!(matches!(result, Err(PdfiumGuardError::Panic)));
    }

    #[test]
    fn catch_pdfium_ok() {
        let r = catch_pdfium(|| "hello");
        assert_eq!(r.unwrap(), "hello");
    }

    #[test]
    fn catch_pdfium_panic() {
        let r: PdfiumGuardResult<()> = catch_pdfium(|| panic!("boom"));
        assert!(matches!(r, Err(PdfiumGuardError::Panic)));
    }
}
