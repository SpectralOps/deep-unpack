use std::path::Path;

fn main() {
    let extract_file = Path::new("tests").join("mocks").join("zip");
    let destination_folder = Path::new("tmp").join("extract").join("zip");
    let result = deep_unpack::extract_to_folder(&extract_file, &destination_folder, 1);

    println!("{:#?}", result);
}
