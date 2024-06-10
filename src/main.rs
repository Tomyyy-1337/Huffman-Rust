// mod huffman;
mod huffman_byte;
mod file_system;
mod lz77;

use std::fs;

use huffman_byte::Huffman;
use lz77::LZ77;
use serde::{Deserialize, Serialize};

fn main() {
    let input = fs::read("lcet10.txt").unwrap();

    let lz77 = lz77::LZ77::encode(&input);

    let huffman = Huffman::encrypt(&lz77.data);

    let serialized = huffman.serialize();

    fs::write("output.bin", &serialized).unwrap();

    let serialized = fs::read("output.bin").unwrap();

    let huffman = Huffman::deserialize(&serialized);

    let lz77_2 = LZ77::from_data(huffman.decrypt());

    let decoded = lz77_2.decode();

    fs::write("output.txt", &decoded).unwrap();

    assert!(input == decoded, "Decoded content is not the same as the original content");

    
    fs::write("lz77_output.bin", lz77.serialize());

}
