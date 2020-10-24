//! Contains utility functions and types for CSV type inference.

use bzip2::write::{BzDecoder, BzEncoder};
use bzip2::Compression;
use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;

/// Represents what is returned from Analysis function
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    /// HashMap of the datatypes of columns
    pub types: HashMap<String, DatasetType>,
    /// First 5 rows and dataset headers
    pub header: String,
}

/// Represents the types that a CSV column could have.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DatasetType {
    /// String-like data, such as University.
    Categorical,
    /// Numerical data, such as Age.
    Numerical,
}

impl DatasetType {
    /// Infers the type of a string based on whether it is numerical or not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dodona::utils::DatasetType;
    /// assert_eq!(DatasetType::infer("Warwick"), DatasetType::Categorical);
    /// assert_eq!(DatasetType::infer("22"), DatasetType::Numerical);
    /// ```
    pub fn infer(value: &str) -> Self {
        match f64::from_str(value) {
            Ok(_) => Self::Numerical,
            Err(_) => Self::Categorical,
        }
    }
}

/// Analyses given dataset and extracts important information
///
/// Declares a reader for the dataset and gets the column types
/// and the first 5 rows and headers of the dataset. These are then
/// combined into a struct which returns the data together.
pub fn analyse(dataset: &str) -> Analysis {
    let mut reader = csv::Reader::from_reader(std::io::Cursor::new(dataset));
    let types = infer_dataset_types(&mut reader).unwrap();
    reader.seek(csv::Position::new()).unwrap();
    let header: String = parse_body(&mut reader, 6);
    log::info!("Header: {}", header);
    Analysis { types, header }
}

/// Infers the types of each column given a dataset.
///
/// Iterates through the rows of a dataset and decides the type of data in each column, which is
/// one of [`DatasetType`]. The dataset is expected to be valid CSV data, with headers as the first
/// row.
///
/// # Examples
///
/// ```
/// # use std::collections::HashMap;
/// # use dodona::utils::{DatasetType, infer_dataset_types};
/// let dataset = r#"
/// education,age
/// Warwick,22
/// Coventry,24
/// "#;
/// let mut reader = csv::Reader::from_reader(std::io::Cursor::new(dataset));
/// let types = infer_dataset_types(&mut reader).unwrap();
///
/// let mut expected = HashMap::new();
/// expected.insert(String::from("education"), DatasetType::Categorical);
/// expected.insert(String::from("age"), DatasetType::Numerical);
///
/// assert_eq!(types, expected);
/// ```
pub fn infer_dataset_types<R: std::io::Read>(
    reader: &mut csv::Reader<R>,
) -> csv::Result<HashMap<String, DatasetType>> {
    // Get the headers
    let headers = reader.headers()?;

    // Insert name and unknown type for each header
    let mut types: HashMap<_, _> = headers
        .into_iter()
        .enumerate()
        .map(|(i, h)| (i, (h.to_string(), DatasetType::Numerical)))
        .collect();

    // Ignore rows that fail to parse
    let records = reader.records().filter_map(Result::ok);

    // Update the types based on each row
    for row in records {
        for (i, v) in row.into_iter().enumerate() {
            let inferred = DatasetType::infer(v);
            let current = types.get_mut(&i).unwrap();

            // Only overwrite if we are changing from Numerical
            if let DatasetType::Numerical = current.1 {
                current.1 = inferred;
            }
        }
    }

    // Pivot `types` to go from k => (n, t) to n => t
    Ok(types.into_iter().map(|(_, v)| (v.0, v.1)).collect())
}

/// Returns a string containing CSV headers
///
/// When a CSV dataset is being used, a user may want to form a string which
/// contains the headers of the dataset. When passed a `csv::Reader`, this
/// function will join together the header into a single `String` and it will
/// return it.
pub fn parse_header<R: std::io::Read>(reader: &mut csv::Reader<R>) -> String {
    reader
        .headers()
        .unwrap()
        .deserialize::<Vec<String>>(None)
        .unwrap()
        .join(",")
}

/// Returns a string containing CSV first n rows
///
/// When a CSV dataset is being used, a user may want to form a string which
/// contains the first n rows of the dataset. When passed a `csv::Reader`, this
/// function will join together the header into a single `String` and it will
/// return it.
pub fn parse_body<R: std::io::Read>(reader: &mut csv::Reader<R>, n: usize) -> String {
    reader
        .records()
        .take(n)
        .map(|record| {
            record
                .unwrap()
                .deserialize::<Vec<String>>(None)
                .unwrap()
                .join(",")
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Compresses data and returns result about compression process
///
/// Takes in a dataset as a string slice and will convert it into a byte representation
/// of the string. Then it will be compressed using BZip2 using an io stream. This write
/// stream is then finished and the Result is returned.
///
/// # Examples
///
/// ```
/// use dodona::utils::compress_data;
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
    let mut write_compress = BzEncoder::new(vec![], Compression::best());
    write_compress.write(data.as_bytes()).unwrap();
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
/// ```
/// use dodona::utils::{decompress_data, compress_data};
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
    write_decompress.write(data).unwrap();
    write_decompress.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorical_cannot_be_overwritten() {
        let dataset = "age\n21\nTwenty\n20";
        let mut reader = csv::Reader::from_reader(dataset.as_bytes());
        let types = infer_dataset_types(&mut reader).unwrap();

        let mut expected = HashMap::new();
        expected.insert(String::from("age"), DatasetType::Categorical);

        assert_eq!(types, expected);
    }

    #[test]
    fn compression_full_stack() {
        let data = "Hello World!";
        let comp_data: Vec<u8> = compress_data(data).unwrap();
        let decomp_vec = decompress_data(&comp_data).unwrap();
        let decomp_data = std::str::from_utf8(&decomp_vec).unwrap();
        assert_eq!(data, decomp_data);
    }
}
