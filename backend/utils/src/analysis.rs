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
