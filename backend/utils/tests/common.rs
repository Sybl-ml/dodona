use utils::{infer_columns, infer_train_and_predict};

#[test]
fn categorical_data_can_be_inferred() {
    let dataset = "age\n21\nTwenty\n20";
    let mut reader = csv::Reader::from_reader(dataset.as_bytes());
    let types = infer_columns(&mut reader).unwrap();

    assert!(types.get(&"age".to_string()).unwrap().is_categorical());
}

#[test]
fn categorical_data_is_salted() {
    let dataset = "age\n21\nTwenty\n20";
    let mut reader = csv::Reader::from_reader(dataset.as_bytes());
    let types_salted = infer_columns(&mut reader).unwrap();
    let mut reader = csv::Reader::from_reader(dataset.as_bytes());
    let types_resalted = infer_columns(&mut reader).unwrap();

    assert_ne!(types_salted, types_resalted,);
}

#[test]
fn numerical_data_can_be_inferred() {
    let dataset = "age\n21\n21.564\n20";
    let mut reader = csv::Reader::from_reader(dataset.as_bytes());
    let types = infer_columns(&mut reader).unwrap();

    assert!(types.get(&"age".to_string()).unwrap().is_numerical());
}

#[test]
fn data_types_can_be_inferred() {
    let dataset = "age,location\n20,Coventry\n20,\n21,Leamington";
    let mut reader = csv::Reader::from_reader(dataset.as_bytes());
    let types = infer_columns(&mut reader).unwrap();
    assert!(types.get(&"age".to_string()).unwrap().is_numerical());
    assert!(types.get(&"location".to_string()).unwrap().is_categorical());
}

#[test]
fn train_and_predict_data_can_be_inferred() {
    let dataset = "age,location\n20,Coventry\n20,\n21,Leamington";
    let (data, predict) = infer_train_and_predict(dataset);

    assert_eq!(data, vec!["age,location", "20,Coventry", "21,Leamington"]);
    assert_eq!(predict, vec!["age,location", "20,"])
}
