//! Contains utility functions and types for CSV type inference.

#[macro_use]
extern crate serde;

use anyhow::Result;
use csv::{Reader, StringRecord};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crypto::generate_string;

pub mod analysis;
pub mod anon;
pub mod compress;

/// Represents the types that a CSV column could have.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ColumnType {
    /// String-like data, such as University.
    Categorical(HashMap<String, String>),
    /// Numerical data, such as Age.
    Numerical(f64, f64),
}

pub type ColumnValues = (String, Vec<String>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub pseudonym: String,
    pub column_type: ColumnType,
}

pub type Columns = HashMap<String, Column>;

impl Column {
    pub fn anonymise(&self, value: String) -> String {
        if value.len() == 0 {
            value
        } else {
            match &self.column_type {
                ColumnType::Categorical(mapping) => mapping.get(&value).unwrap().to_string(),
                ColumnType::Numerical(min, max) => {
                    Column::normalise(f64::from_str(&value).unwrap(), *min, *max).to_string()
                }
            }
        }
    }

    pub fn deanonymise(&self, value: String) -> String {
        if value.len() == 0 {
            value
        } else {
            match &self.column_type {
                ColumnType::Categorical(mapping) => mapping
                    .iter()
                    .filter(|(_, v)| **v == value)
                    .next()
                    .unwrap()
                    .0
                    .to_string(),
                ColumnType::Numerical(min, max) => {
                    Column::denormalise(f64::from_str(&value).unwrap(), *min, *max).to_string()
                }
            }
        }
    }

    pub fn normalise(value: f64, min: f64, max: f64) -> f64 {
        if max - min == 0.0 {
            0.0
        } else {
            (value - min) / (max - min)
        }
    }

    pub fn denormalise(value: f64, min: f64, max: f64) -> f64 {
        if max - min == 0.0 {
            0.0
        } else {
            value * (max - min) + min
        }
    }

    pub fn obfuscate(value: String) -> String {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        generate_string(64).hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn is_categorical(&self) -> bool {
        match self.column_type {
            ColumnType::Categorical(_) => true,
            _ => false,
        }
    }

    pub fn is_numerical(&self) -> bool {
        match self.column_type {
            ColumnType::Numerical(_, _) => true,
            _ => false,
        }
    }
}

impl From<ColumnValues> for Column {
    fn from((name, values): ColumnValues) -> Column {
        if values.iter().all(|v| f64::from_str(v).is_ok()) {
            let numerical: Vec<f64> = values.iter().map(|v| f64::from_str(v).unwrap()).collect();
            let column_type = ColumnType::Numerical(
                *numerical
                    .iter()
                    .min_by(|x, y| x.partial_cmp(&y).unwrap())
                    .unwrap(),
                *numerical
                    .iter()
                    .max_by(|x, y| x.partial_cmp(&y).unwrap())
                    .unwrap(),
            );
            Column {
                name: name,
                pseudonym: generate_string(16),
                column_type: column_type,
            }
        } else {
            let column_type = ColumnType::Categorical(
                values
                    .iter()
                    .zip(values.iter().map(|v| Column::obfuscate(v.to_string())))
                    .map(|(v, o)| (v.to_string(), o))
                    .collect(),
            );
            Column {
                name: name,
                pseudonym: generate_string(16),
                column_type: column_type,
            }
        }
    }
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
/// # use csv::Reader;
/// # use utils::{ColumnType, infer_columns};
/// let dataset = r#"
/// education,age
/// Warwick,22
/// Coventry,24
/// "#;
/// let mut reader = Reader::from_reader(std::io::Cursor::new(dataset));
/// let types = infer_columns(&mut reader).unwrap();
///
/// assert!(types.get(&"education".to_string()).unwrap().is_categorical());
/// assert!(types.get(&"age".to_string()).unwrap().is_numerical());
/// ```
pub fn infer_columns<R: std::io::Read>(reader: &mut Reader<R>) -> csv::Result<Columns> {
    // Get the headers
    let headers = reader.headers()?.to_owned();

    // Ignore rows that fail to parse
    let records: Vec<StringRecord> = reader.records().filter_map(Result::ok).collect();

    // Insert name and unknown type for each header
    Ok(headers
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            (
                h.to_string(),
                Column::from(column_values(h.to_string(), &records, i)),
            )
        })
        .collect())
}

pub fn column_values(name: String, records: &Vec<StringRecord>, col: usize) -> ColumnValues {
    (
        name,
        records
            .iter()
            .map(|r| r.iter().enumerate().nth(col).unwrap().1.to_string())
            .collect(),
    )
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
pub fn parse_header<R: std::io::Read>(reader: &mut Reader<R>) -> String {
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
pub fn parse_body<R: std::io::Read>(reader: &mut Reader<R>, n: usize) -> String {
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
