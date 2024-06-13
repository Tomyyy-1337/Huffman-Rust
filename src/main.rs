// mod huffman;
mod huffman_byte;
mod file_system;
mod lz77;
mod ukkonen;

use std::fs;

use bincode::de::{self, read};
use huffman_byte::Huffman;
use lz77::LZ77;
use rayon::vec;
use serde::{Deserialize, Serialize};

fn main() {
    let test_input = fs::read("input.txt").unwrap();
    println!("Input size: {}", test_input.len());

    let mut start = std::time::Instant::now();
    let encoded = LZ77::encode(&test_input);
    println!("Encoding took {:?}", start.elapsed());   
    println!("Encoded size: {}", encoded.data.iter().map(|x| x.len()).sum::<usize>());
    start = std::time::Instant::now();
    // println!("Encoded size: {}", encoded.len());
    let decoded = encoded.decode();
    println!("Decoding took {:?}", start.elapsed());

    assert!(test_input == decoded, "Decoded content is not the same as the original content")





    // let input = fs::read("input.txt").unwrap();

    // let mut start = std::time::Instant::now();

    // let lz77 = lz77::LZ77::encode(&input);

    // println!("LZ77 encoding took {:?}", start.elapsed());
    // start = std::time::Instant::now();

    // let huffman = Huffman::encrypt(&lz77.data);

    // println!("Huffman encoding took {:?}", start.elapsed());
    // start = std::time::Instant::now();

    // let serialized = huffman.serialize();

    // println!("Serialization took {:?}", start.elapsed());
    // start = std::time::Instant::now();

    // fs::write("output.bin", &serialized).unwrap();

    // println!("Writing to file took {:?}", start.elapsed());
    // start = std::time::Instant::now();

    // let serialized = fs::read("output.bin").unwrap();

    // println!("Reading from file took {:?}", start.elapsed());
    // start = std::time::Instant::now();


    // let huffman = Huffman::deserialize(&serialized);

    // println!("Deserialization took {:?}", start.elapsed());
    // start = std::time::Instant::now();

    // let lz77_2 = LZ77::from_data(huffman.decrypt());

    // println!("Huffman decoding took {:?}", start.elapsed());
    // start = std::time::Instant::now();

    // let decoded = lz77_2.decode();

    // println!("LZ77 decoding took {:?}", start.elapsed());

    // fs::write("output.txt", &decoded).unwrap();

    // assert!(input == decoded, "Decoded content is not the same as the original content");

    
    // fs::write("lz77_output.bin", lz77.serialize());

}
