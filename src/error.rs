use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubstrateError {
    #[error("mmap failed: {0}")]
    MmapFailed(i32),
    #[error("mprotect failed: {0}")]
    MprotectFailed(i32),
    #[error("null symbol pointer")]
    NullSymbol,
    #[error("ELF parse failed: {0}")]
    ElfParseFailed(String),
    #[error("symbol not found: {0}")]
    SymbolNotFound(String),
    #[error("library not found: {0}")]
    LibraryNotFound(String),
    #[error("memory map error: {0}")]
    MemoryMapError(String),
    #[error("instruction decode error at offset {0}")]
    DecodeError(usize),
    #[error("unsupported pc-relative instruction at offset {0}")]
    UnsupportedPcRelative(usize),
}

pub type Result<T> = core::result::Result<T, SubstrateError>;
