//! Raw LLVM profile writer for the coverage build (`.profraw`, format 10).
//!
//! `-C instrument-coverage` gives every function a counter array in the
//! `__llvm_prf_cnts` section, a descriptor record in `__llvm_prf_data` and a
//! name blob in `__llvm_prf_names`. The LLVM profiler runtime that normally
//! writes those out needs libc and a C compiler for the target; the Skyline
//! target has neither, so this module serialises the three sections in the
//! exact raw-profile layout that `llvm-profdata` expects. Nothing here is
//! specific to the Switch: the runtime module passes the section bytes and
//! their load addresses, and the host tests pass synthetic ones.
//!
//! Layout (all little-endian `u64` words, from compiler-rt's
//! `InstrProfData.inc` for raw version 10): a 16-word header, the binary ids
//! (none are written), the data records as linked, the counters, the names,
//! and padding to eight bytes. The data records store each counter pointer
//! relative to the record itself, so they are copied as they are; the header's
//! `CountersDelta` is the runtime distance from the data section to the
//! counter section, which the reader subtracts record by record.
use std::io::{self, Write};

/// `0xff` `l` `p` `r` `o` `f` `r` `0x81`, the 64-bit raw-profile magic.
pub const RAW_MAGIC: u64 = 0xff6c70726f667281;
/// Raw format emitted by the Skyline toolchain's LLVM (22.1.0).
pub const RAW_VERSION: u64 = 10;
/// Words in the raw header.
pub const HEADER_WORDS: usize = 16;
/// Bytes per `__llvm_profile_data` record in version 10.
pub const DATA_RECORD_BYTES: usize = 64;
/// Bytes per counter (`-C instrument-coverage` uses 64-bit counters).
pub const COUNTER_BYTES: usize = 8;
/// `IPVK_Last`: the last value-profiling kind the format knows.
pub const VALUE_KIND_LAST: u64 = 2;

/// The three profile sections of one loaded module and where they are loaded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sections<'a> {
    pub data: &'a [u8],
    pub counters: &'a [u8],
    pub names: &'a [u8],
    pub data_begin: u64,
    pub counters_begin: u64,
    pub names_begin: u64,
}

impl Sections<'_> {
    /// Instrumented functions (data records).
    pub fn functions(&self) -> usize {
        self.data.len() / DATA_RECORD_BYTES
    }

    /// Counter words.
    pub fn counters(&self) -> usize {
        self.counters.len() / COUNTER_BYTES
    }

    /// Bytes `write_profraw` produces for these sections.
    pub fn profraw_len(&self) -> usize {
        HEADER_WORDS * 8 + self.data.len() + self.counters.len() + self.names.len() + pad8(self.names.len())
    }

    fn validate(&self) -> io::Result<()> {
        if self.data.is_empty() || !self.data.len().is_multiple_of(DATA_RECORD_BYTES) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "data section is {} bytes, not a non-zero multiple of {DATA_RECORD_BYTES}",
                    self.data.len()
                ),
            ));
        }
        if !self.counters.len().is_multiple_of(COUNTER_BYTES) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "counter section is {} bytes, not a multiple of {COUNTER_BYTES}",
                    self.counters.len()
                ),
            ));
        }
        Ok(())
    }
}

/// Padding that brings `len` to the next multiple of eight.
pub const fn pad8(len: usize) -> usize {
    (8 - len % 8) % 8
}

/// The header words in file order.
pub fn header(sections: &Sections) -> [u64; HEADER_WORDS] {
    [
        RAW_MAGIC,
        RAW_VERSION,
        0, // BinaryIdsSize: no build id is written; llvm-cov matches by name hash.
        sections.functions() as u64,
        0, // PaddingBytesBeforeCounters (continuous mode only).
        sections.counters() as u64,
        pad8(sections.counters.len()) as u64,
        0, // NumBitmapBytes: rustc emits no MC/DC bitmaps.
        0, // PaddingBytesAfterBitmapBytes.
        sections.names.len() as u64,
        sections.counters_begin.wrapping_sub(sections.data_begin),
        0, // BitmapDelta.
        sections.names_begin,
        0, // NumVTables.
        0, // VNamesSize.
        VALUE_KIND_LAST,
    ]
}

/// Serialise one raw profile. Returns the bytes written.
pub fn write_profraw<W: Write>(out: &mut W, sections: &Sections) -> io::Result<usize> {
    sections.validate()?;
    for word in header(sections) {
        out.write_all(&word.to_le_bytes())?;
    }
    out.write_all(sections.data)?;
    out.write_all(sections.counters)?;
    out.write_all(&[0u8; 8][..pad8(sections.counters.len())])?;
    out.write_all(sections.names)?;
    out.write_all(&[0u8; 8][..pad8(sections.names.len())])?;
    Ok(sections.profraw_len())
}

/// Header fields a reader needs, parsed back from a raw profile. Test support
/// only: the plugin never reads profiles, the host tools do (in Python).
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub version: u64,
    pub binary_ids_size: u64,
    pub functions: u64,
    pub counters: u64,
    pub names_size: u64,
    pub counters_delta: u64,
    pub names_delta: u64,
}

/// Parse the header of a raw profile written by this module or by compiler-rt.
/// `None` when the magic is wrong or the bytes are too short.
#[cfg(test)]
pub fn read_header(bytes: &[u8]) -> Option<Header> {
    if bytes.len() < HEADER_WORDS * 8 {
        return None;
    }
    let word = |index: usize| {
        let mut buffer = [0u8; 8];
        buffer.copy_from_slice(&bytes[index * 8..index * 8 + 8]);
        u64::from_le_bytes(buffer)
    };
    if word(0) != RAW_MAGIC {
        return None;
    }
    Some(Header {
        version: word(1),
        binary_ids_size: word(2),
        functions: word(3),
        counters: word(5),
        names_size: word(9),
        counters_delta: word(10),
        names_delta: word(12),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let data: Vec<u8> = (0..DATA_RECORD_BYTES * 3).map(|n| n as u8).collect();
        let counters: Vec<u8> = (0..COUNTER_BYTES * 7).map(|n| (n * 3) as u8).collect();
        let names = b"\x37\x00x\xda-names-blob-that-is-not-a-multiple".to_vec();
        (data, counters, names)
    }

    #[test]
    fn header_matches_the_reference_layout() {
        // Values read from a compiler-rt profile of the same LLVM (22.1.0) on
        // the host: counters at 0x5d148 and data at 0x5d170, so CountersDelta
        // wrapped to -0x28; version 10; ValueKindLast 2.
        let (data, counters, names) = sample();
        let sections = Sections {
            data: &data,
            counters: &counters,
            names: &names,
            data_begin: 0x5d170,
            counters_begin: 0x5d148,
            names_begin: 0xb71a,
        };
        let words = header(&sections);
        assert_eq!(words[0], 0xff6c70726f667281);
        assert_eq!(words[1], 10);
        assert_eq!(words[2], 0);
        assert_eq!(words[3], 3);
        assert_eq!(words[5], 7);
        assert_eq!(words[9], names.len() as u64);
        assert_eq!(words[10], (-0x28i64) as u64);
        assert_eq!(words[11], 0);
        assert_eq!(words[12], 0xb71a);
        assert_eq!(words[15], 2);
    }

    #[test]
    fn profile_is_header_data_counters_names_padded() {
        let (data, counters, names) = sample();
        let sections = Sections {
            data: &data,
            counters: &counters,
            names: &names,
            data_begin: 0x1000,
            counters_begin: 0x2000,
            names_begin: 0x3000,
        };
        let mut out = Vec::new();
        let written = write_profraw(&mut out, &sections).unwrap();
        assert_eq!(written, out.len());
        assert_eq!(written, sections.profraw_len());
        assert_eq!(written % 8, 0);
        let header = read_header(&out).unwrap();
        assert_eq!(
            header,
            Header {
                version: 10,
                binary_ids_size: 0,
                functions: 3,
                counters: 7,
                names_size: names.len() as u64,
                counters_delta: 0x1000,
                names_delta: 0x3000,
            }
        );
        let mut offset = HEADER_WORDS * 8;
        assert_eq!(&out[offset..offset + data.len()], &data[..]);
        offset += data.len();
        assert_eq!(&out[offset..offset + counters.len()], &counters[..]);
        offset += counters.len();
        assert_eq!(&out[offset..offset + names.len()], &names[..]);
        offset += names.len();
        assert!(out[offset..].iter().all(|byte| *byte == 0));
        assert_eq!(out.len() - offset, pad8(names.len()));
    }

    #[test]
    fn misaligned_sections_are_rejected() {
        let (data, counters, names) = sample();
        let mut out = Vec::new();
        let bad_data = Sections {
            data: &data[..DATA_RECORD_BYTES + 1],
            counters: &counters,
            names: &names,
            data_begin: 0,
            counters_begin: 0,
            names_begin: 0,
        };
        assert_eq!(
            write_profraw(&mut out, &bad_data).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        let bad_counters = Sections {
            data: &data,
            counters: &counters[..COUNTER_BYTES + 3],
            names: &names,
            data_begin: 0,
            counters_begin: 0,
            names_begin: 0,
        };
        assert_eq!(
            write_profraw(&mut out, &bad_counters).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        let empty = Sections {
            data: &[],
            counters: &counters,
            names: &names,
            data_begin: 0,
            counters_begin: 0,
            names_begin: 0,
        };
        assert!(write_profraw(&mut out, &empty).is_err());
        assert!(out.is_empty());
    }

    #[test]
    fn read_header_rejects_foreign_bytes() {
        assert!(read_header(&[0u8; 16]).is_none());
        let mut wrong = vec![0u8; HEADER_WORDS * 8];
        wrong[..8].copy_from_slice(&(RAW_MAGIC ^ 1).to_le_bytes());
        assert!(read_header(&wrong).is_none());
        assert_eq!(pad8(0), 0);
        assert_eq!(pad8(1), 7);
        assert_eq!(pad8(8), 0);
        assert_eq!(pad8(55), 1);
    }
}
