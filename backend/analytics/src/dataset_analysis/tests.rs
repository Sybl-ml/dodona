use std::collections::HashMap;

use crate::dataset_analysis::analyse_project;
use models::dataset_analysis::{CategoricalAnalysis, ColumnAnalysis, NumericalAnalysis};

#[test]
fn analysis_data_is_generated_correctly_and_not_empty() {
    let data = "\
city,country,popcount
Boston,US,3
Concord,US,2
Boston,UK,1
";

    let column_data = vec![
        ("city".to_string(), 'C'),
        ("country".to_string(), 'C'),
        ("popcount".to_string(), 'N'),
    ];

    // Run analysis
    let analysis_data = analyse_project(data, &column_data);

    // Generate Expected Output
    let mut country = CategoricalAnalysis::default();
    country.values.insert("US".to_string(), 2);
    country.values.insert("UK".to_string(), 1);

    let mut city = CategoricalAnalysis::default();
    city.values.insert("Concord".to_string(), 1);
    city.values.insert("Boston".to_string(), 2);

    let mut popcount = NumericalAnalysis::default();
    popcount.min = 1.0;
    popcount.max = 3.0;
    popcount.sum = 6.0;
    popcount.avg = 2.0;
    popcount.values.insert("1".to_string(), 1);
    popcount.values.insert("1.5".to_string(), 0);
    popcount.values.insert("2".to_string(), 1);
    popcount.values.insert("2.5".to_string(), 0);
    popcount.values.insert("3".to_string(), 1);

    let mut expected_data: HashMap<String, ColumnAnalysis> = HashMap::new();
    expected_data.insert("city".to_string(), ColumnAnalysis::Categorical(city));
    expected_data.insert("country".to_string(), ColumnAnalysis::Categorical(country));
    expected_data.insert("popcount".to_string(), ColumnAnalysis::Numerical(popcount));

    // Generated structs should match
    assert_eq!(expected_data, analysis_data);
}
