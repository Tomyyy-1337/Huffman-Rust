mod huffman_byte;
mod file_system;
mod lz77;
// mod ukkonen;

use std::fs;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if let Some(path) = args.get(1) {
        println!("Decoding archive {}", path);
        let data = fs::read(path).unwrap();
        let archive = file_system::Archive::deserialize(&data);
        archive.write_directory("./");
        println!("Decoding complete")
    } else {
        println!("Encoding current directory");
        let archive = file_system::Archive::read_directory("./");
        let serialized = archive.serialize();
        fs::write(format!("./{}.tmy", archive.get_name()), &serialized).unwrap();
        println!("Encoding complete, output file: {}.tmy", archive.get_name());
    }


    // println!("=========Directory==========");
    // let start = std::time::Instant::now();
    // let archive = file_system::Archive::read_directory("./");
    // println!("Encoding directory took {:?}", start.elapsed());

    // let serialized = archive.serialize();
    // fs::write("output_archive.bin", &serialized).unwrap();
    // let start = std::time::Instant::now();
    // let archive = file_system::Archive::deserialize(&serialized);
    // println!("Decoding directory took {:?}", start.elapsed());

    // archive.write_directory("./");


    // println!("=========Single=File==========");

    // let path = "bible.txt";

    // let test_input = fs::read(path).unwrap();

    // let start = std::time::Instant::now();
    // let file = file_system::FileData::read_and_encode(path);
    // println!("Encoding took {:?}", start.elapsed());

    // let serialized = file.serialize();
    // fs::write("output_file.bin", &serialized).unwrap();
    // let file = file_system::FileData::deserialize(&serialized);
    
    // let start = std::time::Instant::now();
    // let decoded = file.decode();
    // println!("Decoding took {:?}", start.elapsed());
    // println!("Compression factor: {}", file.serialize().len() as f64 / test_input.len() as f64);
    // fs::write("output_file.txt", &decoded).unwrap();

    // assert!(test_input == decoded, "Decoded file is not the same as the original file")

}
