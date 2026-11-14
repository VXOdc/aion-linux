//! Minimal, dependency-free GGUF header parser.
//!
//! Reads the magic, version, tensor count, and the metadata key/value
//! pairs we care about (model name, architecture, file type, context
//! length). Does not read tensor data.
//!
//! Value-type table follows the GGUF v2/v3 specification:
//! 0=uint8 1=int8 2=uint16 3=int16 4=uint32 5=int32 6=float32 7=bool
//! 8=string 9=array 10=uint64 11=int64 12=float64.

use anyhow::{anyhow, Context, Result};
use std::io::Read;

pub const GGUF_MAGIC: &[u8; 4] = b"GGUF";

/// The subset of GGUF metadata AION consumes.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct GgufMetadata {
    /// general.name
    pub name: Option<String>,
    /// general.architecture (e.g. "llama", "qwen2")
    pub architecture: Option<String>,
    /// general.file_type (numeric GGML file-type enum; AION reports the
    /// number rather than guessing a quantization label).
    pub file_type: Option<u64>,
    /// <arch>.context_length
    pub context_length: Option<u64>,
    /// Number of metadata key/value pairs in the file.
    pub kv_pairs: u64,
    /// Number of tensors.
    pub tensors: u64,
}

#[derive(Debug, Clone)]
pub struct GgufHeader {
    pub version: u32,
    pub metadata: GgufMetadata,
}

/// Build a tiny in-memory GGUF file for tests (shared with the store module).
#[cfg(test)]
pub fn sample_gguf_bytes() -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(GGUF_MAGIC);
    buf.extend_from_slice(&3u32.to_le_bytes()); // version
    buf.extend_from_slice(&0u64.to_le_bytes()); // tensor count
    buf.extend_from_slice(&2u64.to_le_bytes()); // kv count

    // general.name = "test-model" (string, type 8)
    buf.extend_from_slice(&(12u64).to_le_bytes());
    buf.extend_from_slice(b"general.name");
    buf.extend_from_slice(&8u32.to_le_bytes());
    buf.extend_from_slice(&(10u64).to_le_bytes());
    buf.extend_from_slice(b"test-model");

    // llama.context_length = 4096 (uint32, type 4)
    buf.extend_from_slice(&(20u64).to_le_bytes());
    buf.extend_from_slice(b"llama.context_length");
    buf.extend_from_slice(&4u32.to_le_bytes());
    buf.extend_from_slice(&4096u32.to_le_bytes());

    buf
}

/// Parse the GGUF header from a reader positioned at the start of the file.
pub fn parse_gguf_header<R: Read>(mut reader: R) -> Result<GgufHeader> {
    let mut magic = [0u8; 4];
    reader
        .read_exact(&mut magic)
        .context("failed to read GGUF magic (file too short?)")?;
    if &magic != GGUF_MAGIC {
        return Err(anyhow!("not a GGUF file (bad magic)"));
    }

    let version = read_u32(&mut reader)?;
    if !(1..=3).contains(&version) {
        return Err(anyhow!("unsupported GGUF version {version}"));
    }

    let tensor_count = read_u64(&mut reader)?;
    let kv_count = read_u64(&mut reader)?;

    let mut metadata = GgufMetadata {
        kv_pairs: kv_count,
        tensors: tensor_count,
        ..Default::default()
    };

    // Read up to kv_count pairs. A corrupt count aborts with an error
    // rather than looping forever.
    for _ in 0..kv_count.min(100_000) {
        let key = read_string(&mut reader).context("failed to read metadata key")?;
        let value_type = read_u32(&mut reader).context("failed to read metadata type")?;

        match value_type {
            8 => {
                // String.
                let value = read_string(&mut reader)?;
                match key.as_str() {
                    "general.name" => metadata.name = Some(value),
                    "general.architecture" => metadata.architecture = Some(value),
                    _ => {}
                }
            }
            4 | 5 => {
                // 32-bit integers (uint32/int32).
                let value = read_u32(&mut reader)? as u64;
                if key == "general.file_type" {
                    metadata.file_type = Some(value);
                } else if key.ends_with(".context_length") {
                    metadata.context_length = Some(value);
                }
            }
            10 | 11 => {
                // 64-bit integers.
                let value = read_u64(&mut reader)?;
                if key == "general.file_type" {
                    metadata.file_type = Some(value);
                } else if key.ends_with(".context_length") {
                    metadata.context_length = Some(value);
                }
            }
            0 | 1 | 7 => {
                skip_bytes(&mut reader, 1)?; // uint8 / int8 / bool
            }
            2 | 3 => {
                skip_bytes(&mut reader, 2)?; // uint16 / int16
            }
            6 => {
                skip_bytes(&mut reader, 4)?; // float32
            }
            12 => {
                skip_bytes(&mut reader, 8)?; // float64
            }
            9 => {
                // Array: item type, count, then items.
                let item_type = read_u32(&mut reader)?;
                let count = read_u64(&mut reader)?;
                for _ in 0..count.min(1_000_000) {
                    skip_typed_value(&mut reader, item_type)?;
                }
            }
            other => return Err(anyhow!("unknown GGUF value type {other}")),
        }
    }

    Ok(GgufHeader { version, metadata })
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64<R: Read>(r: &mut R) -> Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn read_string<R: Read>(r: &mut R) -> Result<String> {
    let len = read_u64(r)? as usize;
    if len > 1 << 20 {
        return Err(anyhow!("string too long: {len}"));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn skip_bytes<R: Read>(r: &mut R, n: usize) -> Result<()> {
    let mut buf = vec![0u8; n];
    r.read_exact(&mut buf)?;
    Ok(())
}

fn skip_typed_value<R: Read>(r: &mut R, value_type: u32) -> Result<()> {
    match value_type {
        0 | 1 | 7 => skip_bytes(r, 1)?,
        2 | 3 => skip_bytes(r, 2)?,
        4..=6 => skip_bytes(r, 4)?,
        8 => {
            read_string(r)?;
        }
        10..=12 => skip_bytes(r, 8)?,
        9 => {
            let item_type = read_u32(r)?;
            let count = read_u64(r)?;
            for _ in 0..count.min(1_000_000) {
                skip_typed_value(r, item_type)?;
            }
        }
        other => return Err(anyhow!("unknown GGUF value type {other}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_sample_gguf() {
        let header = parse_gguf_header(Cursor::new(sample_gguf_bytes())).unwrap();
        assert_eq!(header.version, 3);
        assert_eq!(header.metadata.name.as_deref(), Some("test-model"));
        assert_eq!(header.metadata.context_length, Some(4096));
        assert_eq!(header.metadata.kv_pairs, 2);
    }

    #[test]
    fn rejects_bad_magic() {
        let err = parse_gguf_header(Cursor::new(b"NOPE....")).unwrap_err();
        assert!(err.to_string().contains("magic"));
    }

    #[test]
    fn skips_string_and_bool_values() {
        // general.name (string), general.some_bool (bool), then a context
        // length — verifies alignment survives mixed types.
        let mut buf = sample_gguf_bytes();
        // append: general.flag = true (bool)
        buf.extend_from_slice(&(12u64).to_le_bytes());
        buf.extend_from_slice(b"general.flag");
        buf.extend_from_slice(&7u32.to_le_bytes());
        buf.push(1);
        // patch kv count to 3
        let kv_off = 4 + 4 + 8;
        buf[kv_off..kv_off + 8].copy_from_slice(&3u64.to_le_bytes());

        let header = parse_gguf_header(Cursor::new(&buf)).unwrap();
        assert_eq!(header.metadata.kv_pairs, 3);
        assert_eq!(header.metadata.name.as_deref(), Some("test-model"));
    }
}
