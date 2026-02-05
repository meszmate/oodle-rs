use oodle::{
    Error, Oodle, OodleCheckCrc, OodleCompressionLevel, OodleCompressor, OodleDecodeThreadPhase,
    OodleFuzzSafe, OodleVerbosity,
};

fn load_oodle() -> Oodle {
    let path = std::env::var("OODLE_LIB_PATH").expect("set OODLE_LIB_PATH to the Oodle library");
    Oodle::load(&path).expect("failed to load Oodle library")
}

#[test]
#[ignore]
fn compress_decompress_roundtrip_kraken() {
    let oodle = load_oodle();
    let input = b"Hello, Oodle! This is a test of compress and decompress roundtrip.";

    let max_size = oodle.get_compressed_buffer_size_needed(OodleCompressor::Kraken, input.len());
    let mut compressed = vec![0u8; max_size];

    let compressed_size = oodle
        .compress(
            OodleCompressor::Kraken,
            OodleCompressionLevel::Normal,
            input,
            &mut compressed,
        )
        .expect("compression failed");
    compressed.truncate(compressed_size);

    let mut decompressed = vec![0u8; input.len()];
    let decompressed_size = oodle
        .decompress(&compressed, &mut decompressed)
        .expect("decompression failed");

    assert_eq!(decompressed_size, input.len());
    assert_eq!(&decompressed, input.as_slice());
}

#[test]
#[ignore]
fn compress_decompress_with_options_mermaid() {
    let oodle = load_oodle();
    let input: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();

    let max_size = oodle.get_compressed_buffer_size_needed(OodleCompressor::Mermaid, input.len());
    let mut compressed = vec![0u8; max_size];

    let compressed_size = oodle
        .compress(
            OodleCompressor::Mermaid,
            OodleCompressionLevel::Optimal2,
            &input,
            &mut compressed,
        )
        .expect("compression failed");
    compressed.truncate(compressed_size);

    let mut decompressed = vec![0u8; input.len()];
    let decompressed_size = oodle
        .decompress_with_options(
            &compressed,
            &mut decompressed,
            OodleFuzzSafe::Yes,
            OodleCheckCrc::No,
            OodleVerbosity::None,
            OodleDecodeThreadPhase::All,
        )
        .expect("decompression failed");

    assert_eq!(decompressed_size, input.len());
    assert_eq!(decompressed, input);
}

#[test]
#[ignore]
fn get_decode_buffer_size_returns_reasonable_value() {
    let oodle = load_oodle();
    let raw_size = 65536;
    let buf_size = oodle.get_decode_buffer_size(OodleCompressor::Kraken, raw_size, true);
    assert!(buf_size >= raw_size);
}

#[test]
#[ignore]
fn get_compress_scratch_mem_bound_returns_nonzero() {
    let oodle = load_oodle();
    let bound = oodle.get_compress_scratch_mem_bound(
        OodleCompressor::Kraken,
        OodleCompressionLevel::Normal,
        65536,
    );
    assert!(bound > 0);
}

#[test]
#[ignore]
fn decompress_invalid_data_returns_error() {
    let oodle = load_oodle();
    let garbage = [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02, 0x03];
    let mut output = vec![0u8; 256];

    let result = oodle.decompress(&garbage, &mut output);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::DecompressFailed));
}
