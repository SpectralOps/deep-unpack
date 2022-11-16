use std::path::Path;

fn main() {
    let extract_file = Path::new("tests").join("mocks").join("multiple");
    let destination_folder = Path::new("tmp").join("extract").join("multiple");
    let result = deep_unpack::extract_to_folder(&extract_file, &destination_folder, 5);

    println!("{:#?}", result);
}
