# oodle

A Rust wrapper for the [Oodle](http://www.radgametools.com/oodle.htm) compression library. Loads the Oodle shared library at runtime via `libloading`, providing safe access to compress and decompress functions.

## Installation

```sh
cargo add oodle
```

## Usage

You must have the Oodle shared library (`.dll`, `.so`, or `.dylib`) available on your system. The library is loaded at runtime by path.

### Loading the library

```rust
use oodle::Oodle;

let oodle = Oodle::load("path/to/oo2core.dll")?;
```

`Oodle` is `Send + Sync`, so it can be shared across threads.

### Compressing data

```rust
use oodle::{Oodle, OodleCompressor, OodleCompressionLevel};

let oodle = Oodle::load("path/to/oo2core.dll")?;
let input = b"some data to compress";

let max_size = oodle.get_compressed_buffer_size_needed(OodleCompressor::Kraken, input.len());
let mut output = vec![0u8; max_size];

let compressed_size = oodle.compress(
    OodleCompressor::Kraken,
    OodleCompressionLevel::Normal,
    input,
    &mut output,
)?;
output.truncate(compressed_size);
```

### Decompressing data

```rust
use oodle::Oodle;

let oodle = Oodle::load("path/to/oo2core.dll")?;

let mut decompressed = vec![0u8; original_size];
let result = oodle.decompress(&compressed_data, &mut decompressed)?;
```

### Decompressing with options

```rust
use oodle::{Oodle, OodleFuzzSafe, OodleCheckCrc, OodleVerbosity, OodleDecodeThreadPhase};

let oodle = Oodle::load("path/to/oo2core.dll")?;

let mut decompressed = vec![0u8; original_size];
let result = oodle.decompress_with_options(
    &compressed_data,
    &mut decompressed,
    OodleFuzzSafe::Yes,
    OodleCheckCrc::Yes,
    OodleVerbosity::None,
    OodleDecodeThreadPhase::All,
)?;
```

### Other utilities

```rust
// Get the decode buffer size (with corruption safety margin)
let buf_size = oodle.get_decode_buffer_size(OodleCompressor::Kraken, raw_size, true);

// Get the scratch memory bound for compression
let scratch = oodle.get_compress_scratch_mem_bound(
    OodleCompressor::Kraken,
    OodleCompressionLevel::Normal,
    raw_size,
);
```

## Compressors

| Compressor | Value |
|------------|-------|
| Invalid    | -1    |
| None       | 3     |
| Kraken     | 8     |
| Mermaid    | 9     |
| Selkie     | 11    |
| Hydra      | 12    |
| Leviathan  | 13    |

## Compression levels

`HyperFast4` (-4) through `HyperFast1` (-1), `None` (0), `SuperFast` (1), `VeryFast` (2), `Fast` (3), `Normal` (4), `Optimal1` (5) through `Optimal5` (9)

## Error handling

`Oodle::load` returns `Result<Oodle, Error>` where `Error` is:

- `LibLoadError` — the shared library could not be loaded
- `FunctionLoadError` — a required function symbol was not found in the library

`compress` and `decompress` return `Result<usize, Error>` where additional variants are:

- `CompressFailed` — Oodle returned `OODLELZ_FAILED` (0)
- `DecompressFailed` — Oodle returned `OODLELZ_FAILED` (0)

`Error` implements `Display` and `std::error::Error`.

## Running integration tests

Integration tests require the Oodle shared library. They are `#[ignore]`d by default.

```sh
OODLE_LIB_PATH=/path/to/oo2core.so cargo test -- --ignored
```

## License

MIT
