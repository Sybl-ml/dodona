//! Defines compression and decompression utilities for project data

use anyhow::Result;
use bzip2::write::{BzDecoder, BzEncoder};
use bzip2::Compression;
use std::io::Write;

/// Compresses data and returns result about compression process
///
/// Takes in a dataset as a string slice and will convert it into a byte representation
/// of the string. Then it will be compressed using BZip2 using an io stream. This write
/// stream is then finished and the Result is returned.
///
/// # Examples
///
/// ```no_run
/// # use utils::compress::compress_data;
/// let dataset = r#"
/// education,age
/// Warwick,22
/// Coventry,24
/// "#;
///
/// match compress_data(dataset) {
///     Ok(compressed) => {
///         log::info!("Compressed data: {:?}", &compressed);
///     }
///     Err(_) => log::error!("Compression failed"),
/// }
/// ```
pub fn compress_data(data: &str) -> Result<Vec<u8>, std::io::Error> {
    compress_bytes(data.as_bytes())
}

/// Compresses a vector of raw bytes.
pub fn compress_bytes(bytes: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut write_compress = BzEncoder::new(vec![], Compression::best());
    write_compress.write(bytes).unwrap();
    write_compress.finish()
}

/// Compresses a vector of byte arrays into a single compression stream.
pub fn compress_vec(data: &[&str]) -> Result<Vec<u8>, std::io::Error> {
    let mut write_compress = BzEncoder::new(vec![], Compression::best());

    for (i, e) in data.iter().enumerate() {
        write_compress.write(e.as_bytes()).unwrap();

        // Write newlines in for decompression
        if i != data.len() - 1 {
            write_compress.write(&[b'\n']).unwrap();
        }
    }

    write_compress.finish()
}

/// Decompresses data and returns a result about the compression process
///
/// Takes in compressed data as an array slice and writes it to the decompresssion
/// stream. Here the data is decompressed and the write stream is finished. A result
/// is then returned displaying the status of the decompression.
///
/// # Examples
///
/// ```no_run
/// # use utils::compress::{decompress_data, compress_data};
/// let dataset = r#"
/// education,age
/// Warwick,22
/// Coventry,24
/// "#;
///
/// let compressed = compress_data(dataset).unwrap();
///
/// match decompress_data(&compressed) {
///     Ok(decompressed) => {
///         log::info!("Decompressed data: {:?}", &decompressed);
///     }
///     Err(_) => log::error!("Decompression failed"),
/// }
/// ```
pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut write_decompress = BzDecoder::new(vec![]);
    write_decompress.write_all(data).unwrap();
    write_decompress.finish()
}
