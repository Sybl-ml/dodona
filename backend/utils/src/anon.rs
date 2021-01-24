//! Defines anonymisation functionality for project data

use crate::{infer_columns, Columns};
use csv::{Reader, StringRecord, Writer};

pub fn infer_dataset_columns(dataset: &String) -> Option<Columns> {
    // identify the types and range (numerical) or unique values (categorical) of each column
    let mut reader = Reader::from_reader(dataset.as_bytes());
    infer_columns(&mut reader).ok()
}

/// Given a `dataset` represented in a `String`-encoded CSV format, identifies the columns
/// of features within the dataset, anonymises the data using range normalisation for
/// numerical data and pseudonymisation for categorical data, anonymises header names with
/// random pseudonyms and returns an anonymised `String`-encoded CSV dataset and the
/// `Columns` object needed to de-anonymise data from the same domain.
///
/// Returns `None` if headers cannot be parsed or the output CSV cannot be `String`-encoded.
/// Ignores and removes any records which cannot be anonymised
pub fn anonymise_dataset(dataset: &String, columns: &Columns) -> Option<String> {
    // read each record of the data and anonymise each value based on its column
    let mut reader = Reader::from_reader(dataset.as_bytes());
    let headers = reader.headers().ok()?.to_owned();
    let mut writer = Writer::from_writer(vec![]);
    let records: Vec<StringRecord> = reader.records().filter_map(Result::ok).collect();
    let anonymised = records
        .iter()
        .filter_map(|r| anonymise_row(r, &columns, &headers))
        .collect::<Vec<_>>();

    // encode column names based on random pseudonyms
    writer
        .write_record(
            headers
                .iter()
                .map(|c| &columns.get(&c.to_string()).unwrap().pseudonym),
        )
        .unwrap();

    // write the anonymised records to the output CSV
    anonymised.iter().for_each(|v| {
        writer.write_record(v).unwrap();
    });

    // return the anonymised dataset as a `String`
    Some(
        writer
            .into_inner()
            .ok()
            .and_then(|l| String::from_utf8(l).ok())?,
    )
}

pub fn anonymise_row(
    row: &StringRecord,
    types: &Columns,
    headers: &StringRecord,
) -> Option<Vec<String>> {
    row.iter()
        .zip(headers)
        .map(|(v, c)| {
            types
                .get(&c.to_string())
                .and_then(|l| l.anonymise(v.to_string()))
        })
        .collect()
}

/// Given an anonymised `dataset` represented in a `String`-encoded CSV format and the
/// `columns` used to anonymise data, deanonymises the data in the `dataset` based on the values
/// included in `columns`, decodes the pseudonymised headers and returns the deanonymised
/// dataset in a `String`-encoded CSV format
///
/// Returns `None` if headers cannot be parsed or the output CSV cannot be `String`-encoded.
/// Ignores and removes any records which cannot be deanonymised
pub fn deanonymise_dataset(dataset: &String, columns: &Columns) -> Option<String> {
    // identify the pseudonyms used to anonymise the data
    let mut reader = Reader::from_reader(dataset.as_bytes());
    let pseudonyms = reader.headers().ok()?.to_owned();
    let mut writer = Writer::from_writer(vec![]);
    let records: Vec<StringRecord> = reader.records().filter_map(Result::ok).collect();

    // translate the pseudonyms to their original names for each column
    let headers: StringRecord = pseudonyms
        .iter()
        .map(|p| match p {
            "record_id" => p,
            _ => columns
                .values()
                .filter(|c| c.pseudonym == p)
                .next()
                .unwrap()
                .name
                .as_str(),
        })
        .collect::<Vec<_>>()
        .into();

    // deanonymise each record based on `columns`
    let deanonymised = records
        .iter()
        .filter_map(|r| deanonymise_row(r, &columns, &headers))
        .collect::<Vec<_>>();

    // write the deanonymised records to the output CSV
    writer.write_record(&headers).unwrap();
    deanonymised.iter().for_each(|v| {
        writer.write_record(v).unwrap();
    });

    // return the deanonymised dataset as a `String`
    writer
        .into_inner()
        .ok()
        .and_then(|l| String::from_utf8(l).ok())
}

pub fn deanonymise_row(
    row: &StringRecord,
    types: &Columns,
    headers: &StringRecord,
) -> Option<Vec<String>> {
    row.iter()
        .zip(headers)
        .map(|(v, c)| match c {
            "record_id" => Some(v.to_string()),
            _ => types
                .get(&c.to_string())
                .and_then(|l| l.deanonymise(v.to_string()))
        })
        .collect()
}
