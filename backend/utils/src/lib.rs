//! Contains utility functions and types for CSV type inference.

#[macro_use]
extern crate serde;

use anyhow::Result;
use csv::{Reader, StringRecord, Writer};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crypto::generate_string;
use fern::colors::{Color, ColoredLevelConfig};

pub mod analysis;
pub mod anon;
pub mod compress;
pub mod finance;

/// Represents the types that a CSV column could have.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ColumnType {
    /// Categorical data, with a mapping from original values to pseudonymised values
    Categorical(HashMap<String, String>),
    /// Numerical data, with a minimum and maximum value
    Numerical(f64, f64),
}

/// Represents the name of a column and its associated `String` values in a dataset
pub type ColumnValues = (String, Vec<String>);

/// Represents a `Column` in a dataset, including its name, its random pseudonym, and
/// the ColumnType used to anonymise any data in the column
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Column {
    // The name of the column
    pub name: String,
    // The randomly generated pseudonym of the column
    pub pseudonym: String,
    // The type of the column, as well as any values needed to anonymise column values
    pub column_type: ColumnType,
}

/// Represents the `Columns` of a dataset, mapping from their original name to a
/// corresponding `Column` object
pub type Columns = HashMap<String, Column>;

impl Column {
    /// Creates a new column which is guaranteed to be categorical
    /// Used in classification problems where the prediction column
    /// could be accidentally inferred as a numerical column
    pub fn categorical(name: &str, dataset: &str) -> Column {
        let mut reader = Reader::from_reader(dataset.as_bytes());
        let headers = reader.headers().unwrap().to_owned();
        let records: Vec<StringRecord> = reader.records().filter_map(Result::ok).collect();
        let values = column_values(
            name.to_string(),
            &records,
            headers.iter().position(|h| h == name).unwrap(),
        )
        .1;
        Column {
            name: name.to_string(),
            pseudonym: generate_string(16),
            column_type: ColumnType::Categorical(
                values
                    .iter()
                    .filter(|v| !v.is_empty())
                    .zip(values.iter().map(|v| Column::obfuscate(v)))
                    .map(|(v, o)| (v.to_string(), o))
                    .collect(),
            ),
        }
    }

    /// Given a `value` for this `Column`, anonymise it based on this `Column`'s `column_type`
    ///
    /// Once a `Column` has been constructed, it is simple to anonymise data from the same domain
    /// as the column. For example, if the `Column` is categorical then the associated pseudonym
    /// for `value` is returned. Alternatively, if the `Column` is numerical then the `value`
    /// is normalised to the range `[0, 1]` based on the minimum and maximum values of `Column`
    ///
    /// Returns None if the `value` is not in the domain of the `Column` (e.g. it is non-numerical
    /// data in a numerical column or is an unrecognised value in a categorical column)
    pub fn anonymise(&self, value: String) -> Option<String> {
        if value.is_empty() {
            Some(value)
        } else {
            match &self.column_type {
                // if this `Column` holds categorical data, find the pseudonym for `value`
                ColumnType::Categorical(mapping) => Some(mapping.get(&value)?.to_string()),
                // if this `Column` holds numerical data, normalise `value` to the standard range
                ColumnType::Numerical(min, max) => {
                    Some(Column::normalise(f64::from_str(&value).ok()?, *min, *max).to_string())
                }
            }
        }
    }

    /// Given an anonymised `value`, deanonymise it based on this `Column`'s `column_type`
    ///
    /// Deanonymises a `value` by finding the original value based on its anonymised value. For
    /// example, if the `Column` is categorical then the original value associated with the
    /// pseudonym `value` is returned. Alternatively, if the `Column` is numerical, then the scaled
    /// number `value` is denormalised back to its true range and returned
    ///
    /// Returns None if no match was found for the pseudonym `value` in the `Column`
    pub fn deanonymise(&self, value: String) -> Option<String> {
        if value.is_empty() {
            Some(value)
        } else {
            match &self.column_type {
                // if this `Column` holds categorical data, find the original name for this `value`
                ColumnType::Categorical(mapping) => {
                    Some(mapping.iter().find(|(_, v)| **v == value)?.0.to_string())
                }
                // if this `Column` holds numerical data, denormalise `value` to its true range
                ColumnType::Numerical(min, max) => {
                    Some(Column::denormalise(f64::from_str(&value).ok()?, *min, *max).to_string())
                }
            }
        }
    }

    /// Given a `value` and a range defined by `min` and `max`, normalises `value` to the
    /// range `[0, 1]` and returns the scaled value
    pub fn normalise(value: f64, min: f64, max: f64) -> f64 {
        if max - min == 0.0 {
            0.0
        } else {
            (value - min) / (max - min)
        }
    }

    /// Given a `value` in the range `[0, 1]` and a range defined by `min` and `max`, normalises
    /// `value` to its original range and returns its true value
    pub fn denormalise(value: f64, min: f64, max: f64) -> f64 {
        if max - min == 0.0 {
            0.0
        } else {
            value * (max - min) + min
        }
    }

    // Given a `value`, create a random pseudonym for this value
    pub fn obfuscate(value: &str) -> String {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        generate_string(64).hash(&mut hasher);
        hasher.finish().to_string()
    }

    // Returns true if and only if this `Column` represents categorical data
    pub fn is_categorical(&self) -> bool {
        matches!(self.column_type, ColumnType::Categorical(_))
    }

    // Returns true if and only if this `Column` represents numerical data
    pub fn is_numerical(&self) -> bool {
        matches!(self.column_type, ColumnType::Numerical(_, _))
    }
}

impl From<ColumnValues> for Column {
    // Creates a new `Column` object based on its `name` and `values`
    fn from((name, values): ColumnValues) -> Column {
        // check if all values in the column are numerical
        if let Ok(numerical) = values
            .iter()
            .filter(|v| !v.is_empty())
            .map(|v| f64::from_str(v))
            .collect::<Result<Vec<_>, _>>()
        {
            let column_type = ColumnType::Numerical(
                // identify the minimum value of the column
                *numerical
                    .iter()
                    .min_by(|x, y| x.partial_cmp(&y).unwrap())
                    .unwrap(),
                // identify the maximum value of the column
                *numerical
                    .iter()
                    .max_by(|x, y| x.partial_cmp(&y).unwrap())
                    .unwrap(),
            );
            // return a `Column` with a `name`, a random `pseudonym` and numerical `column_type`
            Column {
                name,
                pseudonym: generate_string(16),
                column_type,
            }
        } else {
            let column_type = ColumnType::Categorical(
                values
                    .iter()
                    .filter(|v| !v.is_empty())
                    // obfuscate each value in the column with a random pseudonym
                    .zip(values.iter().map(|v| Column::obfuscate(v)))
                    .map(|(v, o)| (v.to_string(), o))
                    // when collected into a `HashMap`, conflicting pseudonyms for
                    // the same unique value are automatically resolved
                    .collect(),
            );
            // return a `Column` with a `name`, a random `pseudonym` and categorical `column_type`
            Column {
                name,
                pseudonym: generate_string(16),
                column_type,
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

/// Given a column's `name`, its index `col` the `records` from a CSV, return the `ColumnValues`
/// associated with this column in `records`
pub fn column_values(name: String, records: &[StringRecord], col: usize) -> ColumnValues {
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
    let mut lines = data.lines();
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

/// Returns a vector of ids used during ML
///
/// When a CSV is sent to a client, they should be given
/// the ids of the records so that they can be matched up upon
/// being returned.
pub fn generate_ids(dataset: &str) -> (String, Vec<String>) {
    // Break dataset
    let mut record_ids = Vec::new();
    let mut reader = Reader::from_reader(dataset.as_bytes());
    let mut headers = reader.headers().ok().unwrap().to_owned();
    let mut writer = Writer::from_writer(vec![]);
    let records: Vec<StringRecord> = reader.records().filter_map(Result::ok).collect();

    let mut new_header = vec!["record_id"];

    for field in headers.iter() {
        new_header.push(field);
    }

    headers = StringRecord::from(new_header);

    let with_ids = records
        .iter()
        .map(|line| {
            let record_id = generate_string(8);
            record_ids.push(record_id.clone());
            let mut new_line = vec![record_id];
            for field in line.iter() {
                new_line.push(String::from(field));
            }
            StringRecord::from(new_line)
        })
        .collect::<Vec<_>>();

    // Write headers
    writer.write_record(headers.iter()).unwrap();

    // write the new rows with ids to csv
    with_ids.iter().for_each(|v| {
        writer.write_record(v).unwrap();
    });

    let new_csv = writer
        .into_inner()
        .ok()
        .and_then(|l| String::from_utf8(l).ok())
        .unwrap();

    (new_csv, record_ids)
}

/// Sets up the logging for the application.
///
/// Initialises a new instance of a [`fern`] logger, which displays the time and some coloured
/// output based on the level of the message. It also suppresses output from libraries unless they
/// are warnings or errors, and enables all log levels for the current binary.
pub fn setup_logger(lvl_for: &'static str) {
    let levels = vec![(lvl_for, log::LevelFilter::Trace)];

    setup_logger_with_filters(levels)
}

pub fn setup_logger_with_filters<Conditions, Name>(conditions: Conditions)
where
    Conditions: IntoIterator<Item = (Name, log::LevelFilter)>,
    Name: Into<String>,
{
    let colours_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::BrightBlack);

    let mut dispatch = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{colours_line}[{date}][{target}][{level}]\x1B[0m {message}",
                colours_line = format_args!(
                    "\x1B[{}m",
                    colours_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = record.level(),
                message = message,
            ));
        })
        .level(log::LevelFilter::Warn);

    for (module_name, level) in conditions {
        dispatch = dispatch.level_for(module_name.into(), level);
    }

    dispatch
        .chain(std::io::stdout())
        .apply()
        .expect("Failed to initialise the logger");
}

/// Function to workout what chunks the row slice belongs to
///
/// Returns the selection of chunks to get data from, as well as the index of
/// the lower chunk
pub fn calculate_chunk_indices(
    min_row: usize,
    max_row: usize,
    chunk_size: usize,
) -> (Vec<i32>, i32) {
    let lower_chunk = (min_row / chunk_size) as i32;
    let upper_chunk = (max_row / chunk_size) as i32;

    // Equal size
    if lower_chunk == upper_chunk {
        // If one chunk is the first chunk
        // Need to bring in extra chunk because first chunk has 9999 elements instead
        if lower_chunk == 0 {
            (vec![upper_chunk, (upper_chunk + 1)], lower_chunk)
        } else {
            (vec![0, upper_chunk, (upper_chunk + 1)], lower_chunk)
        }
    }
    // Different sizes
    else {
        // If one chunk is the first chunk
        if lower_chunk == 0 {
            (vec![lower_chunk, upper_chunk], lower_chunk)
        } else {
            (vec![0, lower_chunk, upper_chunk], lower_chunk)
        }
    }
}
