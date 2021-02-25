use std::collections::HashMap;

use crate::dataset_analysis::analyse_project;
use models::dataset_analysis::{ColumnAnalysis, NumericalAnalysis, CategoricalAnalysis};

#[test]
fn analysis_data_is_generated_correctly_and_not_empty() {
    let data = "\
city,country,popcount
Boston,US,4628910
Concord,US,42695
Boston,UK,23432
";

    let column_data = vec![
        ("city".to_string(), "C".to_string()),
        ("country".to_string(), "C".to_string()),
        ("popcount".to_string(), "N".to_string()),
    ];

    // Run analysis
    let anaylsis_data = analyse_project(data, column_data.clone());

    // Generate Expected Output
    let mut country = CategoricalAnalysis::default();
    country.values.insert("US".to_string(), 2);
    country.values.insert("UK".to_string(), 1);
    
    let mut city = CategoricalAnalysis::default();
    city.values.insert("Concord".to_string(), 1);
    city.values.insert("Boston".to_string(), 2);

    let mut popcount = NumericalAnalysis::default();
    popcount.min = 23432.0;
    popcount.max = 4628910.0;
    popcount.sum = 4695037.0;
    popcount.avg = 1565012.3333333333;

    let mut expected_data: HashMap<String, ColumnAnalysis> = HashMap::new();
    expected_data.insert("city".to_string(), ColumnAnalysis::Categorical(city));
    expected_data.insert("country".to_string(), ColumnAnalysis::Categorical(country));
    expected_data.insert("popcount".to_string(), ColumnAnalysis::Numerical(popcount));

    assert_eq!(expected_data, anaylsis_data);
}
