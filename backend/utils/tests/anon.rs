use utils::anon::{anonymise_dataset, deanonymise_dataset, infer_dataset_columns};
use utils::generate_ids;
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
    let columns = infer_dataset_columns(&dataset).unwrap();
    let anonymised = anonymise_dataset(&dataset, &columns).unwrap();
    assert!(!anonymised.contains("age") && !anonymised.contains("location"));
}

#[test]
fn datasets_can_be_anonymised() {
    let dataset = "age,location\n20,Coventry\n20,\n21,Leamington".to_string();
    let columns = infer_dataset_columns(&dataset).unwrap();
    assert_ne!(anonymise_dataset(&dataset, &columns).unwrap(), dataset);
}

#[test]
fn datasets_can_be_deanonymised() {
    let dataset = "age,location\n20,Coventry\n20,\n21,Leamington\n".to_string();
    let columns = infer_dataset_columns(&dataset).unwrap();
    let anonymised = anonymise_dataset(&dataset, &columns).unwrap();
    assert_eq!(deanonymise_dataset(&anonymised, &columns).unwrap(), dataset);
}

#[test]
fn n_record_ids_are_generated() {
    let dataset: String = String::from(
        "12.2,Francis Lane,1896\n12.2,Thomas Curtis,1896\n11.8,Tom Burke,1896\n11.4,Arthur Duffey,1900\n10.0,Charlie Greene,1968\n10.0,Jim Hines,1968\n9.9,Jim Hines,1968\n9.92,Carl Lewis,1988\n9.84,Donovan Bailey,1996\n9.69,Usain Bolt,2008\n9.63,Usain Bolt,2012 ",);
    let (new_dataset, ids) = generate_ids(dataset);
    let new_dataset = new_dataset.split("\n").collect::<Vec<_>>();
    assert_eq!(ids.len(), new_dataset.len());
    for item in new_dataset.iter().zip(ids.iter()) {
        let (row, id) = item;
        let broken_row = row.split(",").collect::<Vec<_>>();
        assert_eq!(broken_row[0], id);
    }
}
