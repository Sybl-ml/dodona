use utils::compress::{compress_data, compress_vec, decompress_data};
use utils::infer_train_and_predict;

#[test]
fn compression_full_stack() {
    let data = "Hello World!";
    let comp_data: Vec<u8> = compress_data(data).unwrap();
    let decomp_vec = decompress_data(&comp_data).unwrap();
    let decomp_data = std::str::from_utf8(&decomp_vec).unwrap();
    assert_eq!(data, decomp_data);
}

#[test]
fn vectors_can_be_compressed() {
    let dataset = "age,location\n20,Coventry\n20,\n21,Leamington";
    let (data, predict) = infer_train_and_predict(dataset);

    let comp = compress_vec(&data).unwrap();
    let decomp = decompress_data(&comp).unwrap();

    assert_eq!(
        std::str::from_utf8(&decomp).unwrap(),
        "age,location\n20,Coventry\n21,Leamington"
    );

    let comp = compress_vec(&predict).unwrap();
    let decomp = decompress_data(&comp).unwrap();

    assert_eq!(std::str::from_utf8(&decomp).unwrap(), "age,location\n20,");
}
