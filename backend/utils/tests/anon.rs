use utils::anon::{anonymise_dataset, deanonymise_dataset};
use utils::infer_columns;

#[test]
fn columns_can_be_anonymised() {
    let dataset = "age,location\n20,Coventry\n20,\n21,Leamington";
    let mut reader = csv::Reader::from_reader(dataset.as_bytes());
    let types = infer_columns(&mut reader).unwrap();
    let age = types.get(&"age".to_string()).unwrap();
    let loc = types.get(&"location".to_string()).unwrap();
    assert_eq!(age.anonymise("20.0".to_string()).unwrap(), "0".to_string());
    assert_eq!(
        age.anonymise("20.5".to_string()).unwrap(),
        "0.5".to_string()
    );
    assert_eq!(age.anonymise("21.0".to_string()).unwrap(), "1".to_string());
    assert_ne!(
        loc.anonymise("Coventry".to_string()).unwrap(),
        "Coventry".to_string()
    );
    assert_ne!(
        loc.anonymise("Leamington".to_string()).unwrap(),
        "Leamington".to_string()
    );
}

#[test]
fn headers_can_be_anonymised() {
    let dataset = "age,location\n20,Coventry\n20,\n21,Leamington".to_string();
    let anonymised = anonymise_dataset(dataset).unwrap().0;
    assert!(!anonymised.contains("age") && !anonymised.contains("location"));
}

#[test]
fn datasets_can_be_anonymised() {
    let dataset = "age,location\n20,Coventry\n20,\n21,Leamington".to_string();
    assert_ne!(anonymise_dataset(dataset.clone()).unwrap().0, dataset);
}

#[test]
fn datasets_can_be_deanonymised() {
    let dataset = "age,location\n20,Coventry\n20,\n21,Leamington\n".to_string();
    let (anonymised, columns) = anonymise_dataset(dataset.clone()).unwrap();
    assert_eq!(deanonymise_dataset(anonymised, columns).unwrap(), dataset);
}
