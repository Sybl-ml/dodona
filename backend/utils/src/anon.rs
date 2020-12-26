//! Defines anonymisation functionality for project data

use crate::{infer_columns, Columns};
use csv::{Reader, StringRecord, Writer};

/// Given a `dataset` represented in a `String`-encoded CSV format, identifies the columns
/// of features within the dataset, anonymises the data using range normalisation for
/// numerical data and pseudonymisation for categorical data, anonymises header names with
/// random pseudonyms and returns an anonymised `String`-encoded CSV dataset and the
/// `Columns` object needed to de-anonymise data from the same domain.
pub fn anonymise_dataset(dataset: String) -> Option<(String, Columns)> {
    // identify the types and range (numerical) or unique values (categorical) of each column
    let mut reader = Reader::from_reader(dataset.as_bytes());
    let types: Columns = infer_columns(&mut reader).ok()?;

    // read each record of the data and anonymise each value based on its column
    let mut reader = Reader::from_reader(dataset.as_bytes());
    let headers = reader.headers().ok()?.to_owned();
    let mut writer = Writer::from_writer(vec![]);
    let records: Vec<StringRecord> = reader.records().filter_map(Result::ok).collect();
    let anonymised = records
        .iter()
        .map(|r| {
            r.iter()
                .zip(&headers)
                .filter_map(|(v, c)| types.get(&c.to_string()).and_then(|l| l.anonymise(v.to_string())))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // encode column names based on random pseudonyms
    writer
        .write_record(
            headers
                .iter()
                .map(|c| &types.get(&c.to_string()).unwrap().pseudonym),
        )
        .unwrap();

    // write the anonymised records to the output CSV
    anonymised.iter().for_each(|v| {
        writer.write_record(v).unwrap();
    });

    // return the anonymised dataset as a `String` with the `Columns` used for anonymisation
    Some((
        writer.into_inner().ok().and_then(|l| String::from_utf8(l).ok())?,
        types,
    ))
}

/// Given an anonymised `dataset` represented in a `String`-encoded CSV format and the
/// `columns` used to anonymise data, deanonymises the data in the `dataset` based on the values
/// included in `columns`, decodes the pseudonymised headers and returns the deanonymised
/// dataset in a `String`-encoded CSV format
pub fn deanonymise_dataset(dataset: String, columns: Columns) -> Option<String> {
    // identify the pseudonyms used to anonymise the data
    let mut reader = Reader::from_reader(dataset.as_bytes());
    let pseudonyms = reader.headers().ok()?.to_owned();
    let mut writer = Writer::from_writer(vec![]);
    let records: Vec<StringRecord> = reader.records().filter_map(Result::ok).collect();

    // translate the pseudonyms to their original names for each column
    let headers: Vec<_> = pseudonyms
        .iter()
        .map(|p| {
            &columns
                .values()
                .filter(|c| c.pseudonym == p)
                .next()
                .unwrap()
                .name
        })
        .collect();

    // deanonymise each record based on `columns`
    let deanonymised = records
        .iter()
        .map(|r| {
            r.iter()
                .zip(headers.iter())
                .filter_map(|(v, c)| {
                    columns
                        .get(&c.to_string())
                        .and_then(|r| r.deanonymise(v.to_string()))
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // write the deanonymised records to the output CSV
    writer.write_record(headers.iter()).unwrap();
    deanonymised.iter().for_each(|v| {
        writer.write_record(v).unwrap();
    });

    // return the deanonymised dataset as a `String`
    writer.into_inner().ok().and_then(|l| String::from_utf8(l).ok())
}
