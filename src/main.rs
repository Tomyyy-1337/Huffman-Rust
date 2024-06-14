mod huffman;
mod file_system;
mod lz77;
pub mod bitbuffer;
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
        println!("Encoding complete\noutput file: {}.tmy", archive.get_name());
    }
}
