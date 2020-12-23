//! Contains utility functions and types for CSV type inference.

#[macro_use]
extern crate serde;

use anyhow::Result;
use csv::StringRecord;
use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;

use bzip2::write::{BzDecoder, BzEncoder};
use bzip2::Compression;
use pbkdf2::pbkdf2_simple;

/// Represents what is returned from Analysis function
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    /// HashMap of the datatypes of columns
    pub types: HashMap<String, ColumnType>,
    /// First 5 rows and dataset headers
    pub header: String,
}

/// Represents the types that a CSV column could have.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ColumnType {
    /// String-like data, such as University.
    Categorical(HashMap<String, String>),
    /// Numerical data, such as Age.
    Numerical(f64, f64),
}

impl ColumnType {
    pub fn anonymise(&self, value: String) -> String {
        match self {
            ColumnType::Categorical(mapping) => mapping.get(&value).unwrap().to_string(),
            ColumnType::Numerical(min, max) => ColumnType::normalise(f64::from_str(&value).unwrap(), *min, *max).to_string(),
        }
    }

    pub fn normalise(value: f64, min: f64, max: f64) -> f64 {
        if max - min == 0.0 {
            0.0
        } else {
            (value - min) / (max - min)
        }
    }

    pub fn obfuscate(value: String) -> String {
        pbkdf2_simple(&value, 10).unwrap()
    }

    pub fn is_categorical(&self) -> bool {
        match self {
            ColumnType::Categorical(_) => true,
            _ => false,
        }
    }

    pub fn is_numerical(&self) -> bool {
        match self {
            ColumnType::Numerical(_, _) => true,
            _ => false
        }
    }
}

impl From<Vec<String>> for ColumnType {
    fn from(values: Vec<String>) -> ColumnType {
        if values.iter().all(|v| f64::from_str(v).is_ok()) {
            let numerical: Vec<f64> = values.iter().map(|v| f64::from_str(v).unwrap()).collect();
            ColumnType::Numerical(*numerical.iter().min_by(|x, y| x.partial_cmp(&y).unwrap()).unwrap(), *numerical.iter().max_by(|x, y| x.partial_cmp(&y).unwrap()).unwrap())
        } else {
            ColumnType::Categorical(values.iter().zip(values.iter().map(|v| ColumnType::obfuscate(v.to_string()))).map(|(v, o)| (v.to_string(), o)).collect())
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
/// one of [`ColumnType`]. The dataset is expected to be valid CSV data, with headers as the first
/// row.
///
/// # Examples
///
/// ```
/// # use std::collections::HashMap;
/// # use utils::{ColumnType, infer_dataset_types};
/// let dataset = r#"
/// education,age
/// Warwick,22
/// Coventry,24
/// "#;
/// let mut reader = csv::Reader::from_reader(std::io::Cursor::new(dataset));
/// let types = infer_dataset_types(&mut reader).unwrap();
///
/// assert!(types.get(&"education".to_string()).unwrap().is_categorical());
/// assert!(types.get(&"age".to_string()).unwrap().is_numerical());
/// ```
pub fn infer_dataset_types<R: std::io::Read>(
    reader: &mut csv::Reader<R>,
) -> csv::Result<HashMap<String, ColumnType>> {
    // Get the headers
    let headers = reader.headers()?.to_owned();

    // Ignore rows that fail to parse
    let records: Vec<StringRecord> = reader.records().filter_map(Result::ok).collect();

    // Insert name and unknown type for each header
    Ok(headers
        .into_iter()
        .enumerate()
        .map(|(i, h)| (h.to_string(), ColumnType::from(column_values(&records, i))))
        .collect())
}

pub fn column_values(records: &Vec<StringRecord>, col: usize) -> Vec<String> {
    records.iter().map(|r| r.iter().enumerate().nth(col).unwrap().1.to_string()).collect()
}

/// Infers the training and prediction data based on whether the last column is empty.
///
/// This method will include the header of the data in both results, as this allows them to be used
/// more easily later on. It does not allocate new strings either, meaning it should function well
/// even with large datasets.
pub fn infer_train_and_predict(data: &str) -> (Vec<&str>, Vec<&str>) {
    let mut lines = data.split('\n');
    let header = lines.next().unwrap();

    // Include the header in both
    let mut train = vec![header];
    let mut predict = vec![header];

    // Iterate the rest of the records
    for record in lines {
        if record.split(',').last().unwrap().is_empty() {
            predict.push(record);
        } else {
            train.push(record);
        }
    }

    (train, predict)
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
/// ```no_run
/// # use utils::compress_data;
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
/// # use utils::{decompress_data, compress_data};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorical_cannot_be_overwritten() {
        let dataset = "age\n21\nTwenty\n20";
        let mut reader = csv::Reader::from_reader(dataset.as_bytes());
        let types = infer_dataset_types(&mut reader).unwrap();

        assert!(types.get(&"age".to_string()).unwrap().is_categorical());
    }

    #[test]
    fn train_and_predict_data_can_be_inferred() {
        let dataset = "age,location\n20,Coventry\n20,\n21,Leamington";
        let (data, predict) = infer_train_and_predict(dataset);

        assert_eq!(data, vec!["age,location", "20,Coventry", "21,Leamington"]);
        assert_eq!(predict, vec!["age,location", "20,"])
    }

    #[test]
    fn compression_full_stack() {
        let data = "Hello World!";
        let comp_data: Vec<u8> = compress_data(data).unwrap();
        let decomp_vec = decompress_data(&comp_data).unwrap();
        let decomp_data = std::str::from_utf8(&decomp_vec).unwrap();
        assert_eq!(data, decomp_data);
    }

    #[test]
    fn vectors_can_be_compressed() {
        let dataset = "age,location\n20,Coventry\n20,\n21,Leamington";
        let (data, predict) = infer_train_and_predict(dataset);

        let comp = compress_vec(&data).unwrap();
        let decomp = decompress_data(&comp).unwrap();

        assert_eq!(
            std::str::from_utf8(&decomp).unwrap(),
            "age,location\n20,Coventry\n21,Leamington"
        );

        let comp = compress_vec(&predict).unwrap();
        let decomp = decompress_data(&comp).unwrap();

        assert_eq!(std::str::from_utf8(&decomp).unwrap(), "age,location\n20,");
    }
}
