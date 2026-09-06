//! Read-file format detection. KMC must be told whether its input is FASTA or FASTQ, and it
//! reads only plain or gzip-compressed files.

use flate2::read::MultiGzDecoder;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadFormat {
    Fasta,
    Fastq,
}

/// Detects the format shared by all `paths`. Fails if a file is unreadable, not FASTA/FASTQ,
/// compressed with something other than gzip, or if the files do not all have the same format.
pub fn detect_shared_format(paths: &[impl AsRef<Path>]) -> Result<ReadFormat, String> {
    let mut shared: Option<ReadFormat> = None;
    for path in paths {
        let path = path.as_ref();
        let format = detect_format(path).map_err(|e| format!("{}: {e}", path.display()))?;
        match shared {
            None => shared = Some(format),
            Some(first) if first != format => {
                return Err(format!(
                    "all input files must have the same format for on-disk k-mer counting, \
                     but {} is {:?} while an earlier file is {:?}",
                    path.display(),
                    format,
                    first
                ))
            }
            Some(_) => {}
        }
    }
    shared.ok_or_else(|| "no input files".to_string())
}

/// Detects FASTA or FASTQ from the first record marker, looking through gzip if needed.
pub fn detect_format(path: &Path) -> Result<ReadFormat, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut magic = [0u8; 3];
    let n = file.read(&mut magic).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;

    let raw: Box<dyn Read> = match &magic[..n] {
        [0x1f, 0x8b, ..] => Box::new(MultiGzDecoder::new(file)),
        [b'B', b'Z', b'h'] => {
            return Err(
                "bzip2-compressed reads are not supported by the on-disk k-mer counter; \
                        use gzip or uncompressed files"
                    .to_string(),
            )
        }
        _ => Box::new(file),
    };

    let mut reader = BufReader::new(raw);
    let mut byte = [0u8; 1];
    loop {
        if reader.read(&mut byte).map_err(|e| e.to_string())? == 0 {
            return Err("file is empty".to_string());
        }
        match byte[0] {
            b'>' => return Ok(ReadFormat::Fasta),
            b'@' => return Ok(ReadFormat::Fastq),
            c if c.is_ascii_whitespace() => continue,
            other => {
                return Err(format!(
                    "not a FASTA or FASTQ file (starts with {:?})",
                    other as char
                ))
            }
        }
    }
}
