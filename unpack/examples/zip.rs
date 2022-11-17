use std::path::Path;

fn main() {
    let extract_file = Path::new("tests").join("mocks").join("zip");
    let destination_folder = Path::new("tmp").join("extract").join("zip");

    let result = deep_unpack::DeepWalk::new()
        .folder(format!("{}", extract_file.display()))
        .unpack_folder(format!("{}", destination_folder.display()))
        .extract()
        .unwrap();

    println!("{:#?}", result);
}
