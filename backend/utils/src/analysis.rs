//! Defines data set analysis utilities for project data

use crate::{infer_columns, parse_body, Column};
use csv::{Position, Reader};
use std::collections::HashMap;

// use mongodb::bson::oid::ObjectId;
use std::str::FromStr;
/// Represents what is returned from Analysis function
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    /// HashMap of the datatypes of columns
    pub types: HashMap<String, Column>,
    /// First 5 rows and dataset headers
    pub header: String,
}

/// Analyses given dataset and extracts important information
///
/// Declares a reader for the dataset and gets the column types
/// and the first 5 rows and headers of the dataset. These are then
/// combined into a struct which returns the data together.
pub fn analyse(dataset: &str) -> Analysis {
    let mut reader = Reader::from_reader(std::io::Cursor::new(dataset));
    let types = infer_columns(&mut reader).unwrap();
    reader.seek(Position::new()).unwrap();
    let header: String = parse_body(&mut reader, 6);
    log::info!("Header: {}", header);
    Analysis { types, header }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetAnalysis {
    pub columns: HashMap<String, ColumnAnalysis>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ColumnAnalysis {
    Categorical(CategoricalAnalysis),
    Numerical(NumericalAnalysis),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CategoricalAnalysis {
    /// All the values in the column
    values: HashMap<String, i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumericalAnalysis {
    max: f64,
    min: f64,
    sum: f64,
    avg: f64,
}

impl Default for NumericalAnalysis {
    fn default() -> Self {
        Self {
            max: f64::MIN,
            min: f64::MAX,
            sum: 0.0,
            avg: 0.0,
        }
    }
}

/// Basic Dataset Analysis
///
/// Converts dataset string to a reader and performs statistical analysis
pub fn analyse_project(dataset: &str, column_data: Vec<(String, String)>) -> DatasetAnalysis {
    let mut reader = Reader::from_reader(std::io::Cursor::new(dataset));

    let headers = reader
        .headers()
        .unwrap()
        .deserialize::<Vec<String>>(None)
        .unwrap();

    let mut tracker: DatasetAnalysis = DatasetAnalysis {
        columns: column_data
            .iter()
            .map(|(header, data_type)| {
                (
                    header.clone(),
                    match data_type.as_str() {
                        "N" => ColumnAnalysis::Numerical(NumericalAnalysis::default()),
                        _ => ColumnAnalysis::Categorical(CategoricalAnalysis::default()),
                    },
                )
            })
            .collect(),
    };
    let mut dataset_length = 0;
    for result in reader.records() {
        let row = result.unwrap();
        dataset_length += 1;

        for (elem, header) in row.iter().zip(headers.iter()) {
            match tracker.columns.get_mut(header).unwrap() {
                ColumnAnalysis::Categorical(content) => {
                    *content.values.entry(elem.to_string()).or_insert(0) += 1;
                }
                ColumnAnalysis::Numerical(content) => {
                    content.min = content.min.min(f64::from_str(elem).unwrap());
                    content.max = content.max.max(f64::from_str(elem).unwrap());
                    content.sum = content.sum + f64::from_str(elem).unwrap();
                }
            };
        }
    }

    column_data.iter().for_each(|(header, _)| {
        match tracker.columns.get_mut(header).unwrap() {
            ColumnAnalysis::Numerical(content) => {
                content.avg = content.sum as f64 / dataset_length as f64;
            }
            _ => {}
        };
    });

    dbg!(&tracker);
    return tracker;
}

#[cfg(test)]
mod tests {
    use crate::analysis::analyse_project;
    #[test]
    fn test() {
        let data = "\
city,country,popcount
Boston,United States,4628910
Concord,United States,42695
Boston,United Kingdom,23432
";
        let column_data = vec![
            ("city".to_string(), "C".to_string()),
            ("country".to_string(), "C".to_string()),
            ("popcount".to_string(), "N".to_string()),
        ];

        let anaylsis_data = analyse_project(data, column_data);
        dbg!(data);
        dbg!(anaylsis_data);
        assert_eq!(1, 2)
    }
}
