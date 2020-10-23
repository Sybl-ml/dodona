//! Contains utility functions and types for CSV type inference.

use std::collections::HashMap;
use std::str::FromStr;

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
///
/// let types = infer_dataset_types(dataset).unwrap();
///
/// let mut expected = HashMap::new();
/// expected.insert(String::from("education"), DatasetType::Categorical);
/// expected.insert(String::from("age"), DatasetType::Numerical);
///
/// assert_eq!(types, expected);
/// ```
pub fn infer_dataset_types(dataset: &str) -> csv::Result<HashMap<String, DatasetType>> {
    // Get the headers
    let mut reader = csv::Reader::from_reader(dataset.as_bytes());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorical_cannot_be_overwritten() {
        let dataset = "age\n21\nTwenty\n20";
        let types = infer_dataset_types(dataset).unwrap();

        let mut expected = HashMap::new();
        expected.insert(String::from("age"), DatasetType::Categorical);

        assert_eq!(types, expected);
    }
}
