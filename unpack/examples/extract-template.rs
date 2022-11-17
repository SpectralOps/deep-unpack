use std::path::Path;

fn main() {
    let extract_file = Path::new("tests").join("mocks").join("multiple");
    let destination_folder = Path::new("tmp").join("extract").join("extract-template");
    let result = deep_unpack::DeepWalk::new()
        .folder(format!("{}", extract_file.display()))
        .unpack_folder(format!("{}", destination_folder.display()))
        .unpack_level(4 as u32)
        .extract_template("_PREFIX_${FILENAME}$_SUFFIX_")
        .extract()
        .unwrap();

    println!("{:#?}", result);
}
