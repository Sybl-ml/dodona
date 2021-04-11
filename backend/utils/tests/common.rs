use utils::{chunk_calculate, infer_columns, infer_train_and_predict};

pub static CHUNK_SIZE: usize = 100;

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

    assert_ne!(types_salted, types_resalted);
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

#[test]
fn correct_chunks_calculated_chunk_one() {
    let min_row = 0;
    let max_row = 10;

    let (chunk_vec, lower_chunk) = chunk_calculate(min_row, max_row, CHUNK_SIZE);
    assert_eq!(0, lower_chunk);
    assert_eq!(chunk_vec, vec![0, 1]);
}

#[test]
fn correct_chunks_calculated_chunk_two() {
    let min_row = 101;
    let max_row = 125;

    let (chunk_vec, lower_chunk) = chunk_calculate(min_row, max_row, CHUNK_SIZE);
    assert_eq!(1, lower_chunk);
    assert_eq!(chunk_vec, vec![0, 1, 2]);
}

#[test]
fn correct_chunks_calculated_multi_chunk() {
    let min_row = 90;
    let max_row = 101;

    let (chunk_vec, lower_chunk) = chunk_calculate(min_row, max_row, CHUNK_SIZE);
    assert_eq!(0, lower_chunk);
    assert_eq!(chunk_vec, vec![0, 1]);
}
