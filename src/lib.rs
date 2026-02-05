use libloading::Library;
use std::ffi::{OsStr, c_void};
use std::fmt;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum OodleFuzzSafe {
    No = 0,
    Yes = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum OodleCheckCrc {
    No = 0,
    Yes = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum OodleVerbosity {
    None = 0,
    Minimal = 1,
    Some = 2,
    Lots = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum OodleDecodeThreadPhase {
    Phase1 = 1,
    Phase2 = 2,
    All = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum OodleCompressor {
    Invalid = -1,
    None = 3,
    Kraken = 8,
    Mermaid = 9,
    Selkie = 11,
    Hydra = 12,
    Leviathan = 13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum OodleCompressionLevel {
    HyperFast4 = -4,
    HyperFast3 = -3,
    HyperFast2 = -2,
    HyperFast1 = -1,
    None = 0,
    SuperFast = 1,
    VeryFast = 2,
    Fast = 3,
    Normal = 4,
    Optimal1 = 5,
    Optimal2 = 6,
    Optimal3 = 7,
    Optimal4 = 8,
    Optimal5 = 9,
}

// ---------------------------------------------------------------------------
// FFI function types — matching oodle2.h 2.9.11
// ---------------------------------------------------------------------------

type CompressFn = unsafe extern "C" fn(
    compressor: OodleCompressor,
    raw_buf: *const c_void,
    raw_len: isize,
    comp_buf: *mut c_void,
    level: OodleCompressionLevel,
    p_options: *const c_void,
    dictionary_base: *const c_void,
    lrm: *const c_void,
    scratch_mem: *mut c_void,
    scratch_size: isize,
) -> isize;

type DecompressFn = unsafe extern "C" fn(
    comp_buf: *const c_void,
    comp_buf_size: isize,
    raw_buf: *mut c_void,
    raw_len: isize,
    fuzz_safe: OodleFuzzSafe,
    check_crc: OodleCheckCrc,
    verbosity: OodleVerbosity,
    dec_buf_base: *mut c_void,
    dec_buf_size: isize,
    fp_callback: *mut c_void,
    callback_user_data: *mut c_void,
    decoder_memory: *mut c_void,
    decoder_memory_size: isize,
    thread_phase: OodleDecodeThreadPhase,
) -> isize;

type GetCompressedBufferSizeNeededFn =
    unsafe extern "C" fn(compressor: OodleCompressor, raw_size: isize) -> isize;

type GetDecodeBufferSizeFn = unsafe extern "C" fn(
    compressor: OodleCompressor,
    raw_size: isize,
    corruption_possible: i32,
) -> isize;

type GetCompressScratchMemBoundFn = unsafe extern "C" fn(
    compressor: OodleCompressor,
    level: OodleCompressionLevel,
    raw_len: isize,
    p_options: *const c_void,
) -> isize;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum Error {
    LibLoadError(libloading::Error),
    FunctionLoadError(libloading::Error),
    CompressFailed,
    DecompressFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::LibLoadError(e) => write!(f, "failed to load library: {e}"),
            Error::FunctionLoadError(e) => write!(f, "failed to load function: {e}"),
            Error::CompressFailed => write!(f, "compression failed"),
            Error::DecompressFailed => write!(f, "decompression failed"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::LibLoadError(e) | Error::FunctionLoadError(e) => Some(e),
            Error::CompressFailed | Error::DecompressFailed => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Oodle
// ---------------------------------------------------------------------------

/// Oodle compression library loaded at runtime.
pub struct Oodle {
    compress_fn: CompressFn,
    decompress_fn: DecompressFn,
    get_compressed_buffer_size_needed_fn: GetCompressedBufferSizeNeededFn,
    get_decode_buffer_size_fn: GetDecodeBufferSizeFn,
    get_compress_scratch_mem_bound_fn: GetCompressScratchMemBoundFn,
    _lib: Library,
}

impl fmt::Debug for Oodle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Oodle").finish_non_exhaustive()
    }
}

// SAFETY: The raw `fn` pointers are plain function pointers into the loaded
// shared library. They contain no interior mutability and are safe to share
// across threads. The `Library` handle itself is `Send` and `Sync` in
// libloading 0.8.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Oodle>();
};

impl Oodle {
    /// Load the Oodle shared library from the given path.
    pub fn load(path: impl AsRef<OsStr>) -> Result<Self, Error> {
        unsafe {
            let lib = Library::new(path.as_ref()).map_err(Error::LibLoadError)?;

            let compress_fn = *lib
                .get::<CompressFn>(b"OodleLZ_Compress")
                .map_err(Error::FunctionLoadError)?;
            let decompress_fn = *lib
                .get::<DecompressFn>(b"OodleLZ_Decompress")
                .map_err(Error::FunctionLoadError)?;
            let get_compressed_buffer_size_needed_fn = *lib
                .get::<GetCompressedBufferSizeNeededFn>(b"OodleLZ_GetCompressedBufferSizeNeeded")
                .map_err(Error::FunctionLoadError)?;
            let get_decode_buffer_size_fn = *lib
                .get::<GetDecodeBufferSizeFn>(b"OodleLZ_GetDecodeBufferSize")
                .map_err(Error::FunctionLoadError)?;
            let get_compress_scratch_mem_bound_fn = *lib
                .get::<GetCompressScratchMemBoundFn>(b"OodleLZ_GetCompressScratchMemBound")
                .map_err(Error::FunctionLoadError)?;

            Ok(Self {
                compress_fn,
                decompress_fn,
                get_compressed_buffer_size_needed_fn,
                get_decode_buffer_size_fn,
                get_compress_scratch_mem_bound_fn,
                _lib: lib,
            })
        }
    }

    /// Compress `input` into `output` using the given compressor and level.
    ///
    /// Returns the number of bytes written to `output`.
    pub fn compress(
        &self,
        compressor: OodleCompressor,
        level: OodleCompressionLevel,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, Error> {
        let result = unsafe {
            (self.compress_fn)(
                compressor,
                input.as_ptr() as *const c_void,
                input.len() as isize,
                output.as_mut_ptr() as *mut c_void,
                level,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                0,
            )
        };
        if result == 0 {
            Err(Error::CompressFailed)
        } else {
            Ok(result as usize)
        }
    }

    /// Decompress `source` into `dest` with default options.
    ///
    /// Returns the number of decompressed bytes written to `dest`.
    pub fn decompress(&self, source: &[u8], dest: &mut [u8]) -> Result<usize, Error> {
        self.decompress_with_options(
            source,
            dest,
            OodleFuzzSafe::Yes,
            OodleCheckCrc::No,
            OodleVerbosity::None,
            OodleDecodeThreadPhase::All,
        )
    }

    /// Decompress `source` into `dest` with explicit options.
    ///
    /// Returns the number of decompressed bytes written to `dest`.
    pub fn decompress_with_options(
        &self,
        source: &[u8],
        dest: &mut [u8],
        fuzz_safe: OodleFuzzSafe,
        check_crc: OodleCheckCrc,
        verbosity: OodleVerbosity,
        thread_phase: OodleDecodeThreadPhase,
    ) -> Result<usize, Error> {
        let result = unsafe {
            (self.decompress_fn)(
                source.as_ptr() as *const c_void,
                source.len() as isize,
                dest.as_mut_ptr() as *mut c_void,
                dest.len() as isize,
                fuzz_safe,
                check_crc,
                verbosity,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                thread_phase,
            )
        };
        if result == 0 {
            Err(Error::DecompressFailed)
        } else {
            Ok(result as usize)
        }
    }

    /// Returns the buffer size needed to hold the compressed output for the
    /// given compressor and raw data size.
    pub fn get_compressed_buffer_size_needed(
        &self,
        compressor: OodleCompressor,
        raw_size: usize,
    ) -> usize {
        unsafe {
            (self.get_compressed_buffer_size_needed_fn)(compressor, raw_size as isize) as usize
        }
    }

    /// Returns the buffer size needed for decompression, optionally accounting
    /// for possible data corruption.
    pub fn get_decode_buffer_size(
        &self,
        compressor: OodleCompressor,
        raw_size: usize,
        corruption_possible: bool,
    ) -> usize {
        unsafe {
            (self.get_decode_buffer_size_fn)(
                compressor,
                raw_size as isize,
                corruption_possible as i32,
            ) as usize
        }
    }

    /// Returns the scratch memory size bound needed for compression.
    pub fn get_compress_scratch_mem_bound(
        &self,
        compressor: OodleCompressor,
        level: OodleCompressionLevel,
        raw_len: usize,
    ) -> usize {
        unsafe {
            (self.get_compress_scratch_mem_bound_fn)(
                compressor,
                level,
                raw_len as isize,
                std::ptr::null(),
            ) as usize
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_compressor_discriminants() {
        assert_eq!(OodleCompressor::Invalid as i32, -1);
        assert_eq!(OodleCompressor::None as i32, 3);
        assert_eq!(OodleCompressor::Kraken as i32, 8);
        assert_eq!(OodleCompressor::Mermaid as i32, 9);
        assert_eq!(OodleCompressor::Selkie as i32, 11);
        assert_eq!(OodleCompressor::Hydra as i32, 12);
        assert_eq!(OodleCompressor::Leviathan as i32, 13);
    }

    #[test]
    fn enum_compression_level_discriminants() {
        assert_eq!(OodleCompressionLevel::HyperFast4 as i32, -4);
        assert_eq!(OodleCompressionLevel::HyperFast3 as i32, -3);
        assert_eq!(OodleCompressionLevel::HyperFast2 as i32, -2);
        assert_eq!(OodleCompressionLevel::HyperFast1 as i32, -1);
        assert_eq!(OodleCompressionLevel::None as i32, 0);
        assert_eq!(OodleCompressionLevel::SuperFast as i32, 1);
        assert_eq!(OodleCompressionLevel::VeryFast as i32, 2);
        assert_eq!(OodleCompressionLevel::Fast as i32, 3);
        assert_eq!(OodleCompressionLevel::Normal as i32, 4);
        assert_eq!(OodleCompressionLevel::Optimal1 as i32, 5);
        assert_eq!(OodleCompressionLevel::Optimal2 as i32, 6);
        assert_eq!(OodleCompressionLevel::Optimal3 as i32, 7);
        assert_eq!(OodleCompressionLevel::Optimal4 as i32, 8);
        assert_eq!(OodleCompressionLevel::Optimal5 as i32, 9);
    }

    #[test]
    fn enum_copy_and_eq() {
        let a = OodleCompressor::Kraken;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn enum_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(OodleCompressor::Kraken);
        set.insert(OodleCompressor::Mermaid);
        assert_eq!(set.len(), 2);
        assert!(set.contains(&OodleCompressor::Kraken));
    }

    #[test]
    fn error_display() {
        assert_eq!(Error::CompressFailed.to_string(), "compression failed");
        assert_eq!(Error::DecompressFailed.to_string(), "decompression failed");
    }

    #[test]
    fn error_is_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<Error>();
    }

    #[test]
    fn load_nonexistent_library() {
        let result = Oodle::load("nonexistent_library.so");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::LibLoadError(_)));
    }

    #[test]
    fn oodle_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Oodle>();
    }
}
