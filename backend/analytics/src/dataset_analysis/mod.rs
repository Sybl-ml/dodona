use std::sync::Arc;
use std::str::FromStr;
use std::collections::HashMap;

use csv::Reader;
use anyhow::{anyhow, Result};
use mongodb::{
    bson::{de::from_document, doc, oid::ObjectId, ser::to_document},
    Database,
};

use models::dataset_analysis::{ColumnAnalysis, NumericalAnalysis, CategoricalAnalysis};
use models::dataset_details::DatasetDetails;
use models::dataset_analysis::DatasetAnalysis;
use models::datasets::Dataset;
use utils::compress::decompress_data;


/// Prepare Data for analysis
///
///
///
pub async fn prepare_dataset(database: &Arc<Database>, project_id: &ObjectId) -> Result<()>{
    let datasets = database.collection("datasets");
    let dataset_details = database.collection("dataset_details");
    let dataset_analysis = database.collection("dataset_analysis");

    // Obtain the dataset
    let document = datasets
        .find_one(doc! { "project_id": &project_id }, None)
        .await?
        .ok_or(anyhow!("Dataset doesn't exist"))?;

    let dataset: Dataset = from_document(document)?;
    let comp_train = dataset.dataset.expect("missing training dataset").bytes;
    let decomp_train = decompress_data(&comp_train)?;
    let train = crypto::clean(std::str::from_utf8(&decomp_train)?);

    // Obtain the column details
    let document = dataset_details
        .find_one(doc! { "project_id": &project_id }, None)
        .await?
        .ok_or(anyhow!("Dataset Details doesn't exist"))?;

    let dataset_detail: DatasetDetails = from_document(document)?;
    let column_types = dataset_detail.column_types;

    let mut column_data = Vec::new();
    for (name, column) in column_types {
        if column.is_categorical() {
            column_data.push((name, "C".to_string()));
        } else {
            column_data.push((name, "N".to_string()));
        }
    }

    let analysis = analyse_project(&train, column_data);
    dbg!(&analysis);

    let analysis = DatasetAnalysis::new(project_id.clone(), analysis);
    
    let document = to_document(&analysis)?;
    dataset_analysis.insert_one(document, None).await?;
    
    Ok(())
}

/// Basic Dataset Analysis
///
/// Converts dataset string to a reader and performs statistical analysis
pub fn analyse_project(
    dataset: &str,
    column_data: Vec<(String, String)>,
) -> HashMap<String, ColumnAnalysis> {
    let mut reader = Reader::from_reader(std::io::Cursor::new(dataset));

    let headers = reader
        .headers()
        .unwrap()
        .deserialize::<Vec<String>>(None)
        .expect("Couldn't deserialize header data");

    let mut tracker: HashMap<String, ColumnAnalysis> = column_data
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
        .collect();

    let mut dataset_length = 0;
    for result in reader.records() {
        let row = result.expect("Failed to read row");
        dataset_length += 1;

        for (elem, header) in row.iter().zip(headers.iter()) {
            match tracker.get_mut(header).expect("Failed to access header data") {
                ColumnAnalysis::Categorical(content) => {
                    *content.values.entry(elem.to_string()).or_insert(0) += 1;
                }
                ColumnAnalysis::Numerical(content) => {
                    content.min = content.min.min(f64::from_str(elem).expect("Failed to convert to float"));
                    content.max = content.max.max(f64::from_str(elem).expect("Failed to convert to float"));
                    content.sum = content.sum + f64::from_str(elem).expect("Failed to convert to float");
                }
            };
        }
    }

    column_data.iter().for_each(|(header, _)| {
        match tracker.get_mut(header).expect("Failed to access header data") {
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
mod tests;