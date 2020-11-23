//! Contains the builder functions used to generate message for DCL-DCN protcol

/// Builder for the hearbeat message
pub fn heartbeat_msg() -> String {
    String::from("{'alive': '1'}\0")
}

/// Builder for the job config message
pub fn job_config_msg() -> String {
    String::from("{'job_config': 'yes'}\0")
}

/// Builder for the dataset and predict message
pub fn dataset_msg(dataset: String, predict: String) -> String {
    String::from(format!(
        "{{'dataset': '{}', 'predict': '{}' }}\0",
        dataset, predict
    ))
}
