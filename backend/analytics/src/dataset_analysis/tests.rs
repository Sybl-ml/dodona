
use crate::dataset_analysis::analyse_project;

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
