//! Defines anonymisation functionality for project data

use csv::{Reader, Writer, StringRecord};
use crate::{Columns, infer_columns};

pub fn anonymise_dataset(dataset: String) -> (String, Columns) {
    let mut reader = Reader::from_reader(dataset.as_bytes());
    let types: Columns = infer_columns(&mut reader).unwrap();
    let mut reader = Reader::from_reader(dataset.as_bytes());
    let headers = reader.headers().unwrap().to_owned();
    let mut writer = Writer::from_writer(vec![]);
    let records: Vec<StringRecord> = reader.records().filter_map(Result::ok).collect();
    let anonymised = records
        .iter()
        .map(|r| {
            r.iter()
                .zip(&headers)
                .map(|(v, c)| types.get(&c.to_string()).unwrap().anonymise(v.to_string()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    writer
        .write_record(
            headers
                .iter()
                .map(|c| &types.get(&c.to_string()).unwrap().pseudonym),
        )
        .unwrap();
    anonymised.iter().for_each(|v| {
        writer.write_record(v).unwrap();
    });
    (
        String::from_utf8(writer.into_inner().unwrap()).unwrap(),
        types,
    )
}

pub fn deanonymise_dataset(dataset: String, columns: Columns) -> String {
    let mut reader = Reader::from_reader(dataset.as_bytes());
    let pseudonyms = reader.headers().unwrap().to_owned();
    let mut writer = Writer::from_writer(vec![]);
    let records: Vec<StringRecord> = reader.records().filter_map(Result::ok).collect();
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
    let deanonymised = records
        .iter()
        .map(|r| {
            r.iter()
                .zip(headers.iter())
                .map(|(v, c)| {
                    columns
                        .get(&c.to_string())
                        .unwrap()
                        .deanonymise(v.to_string())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    writer.write_record(headers.iter()).unwrap();
    deanonymised.iter().for_each(|v| {
        writer.write_record(v).unwrap();
    });
    String::from_utf8(writer.into_inner().unwrap()).unwrap()
}