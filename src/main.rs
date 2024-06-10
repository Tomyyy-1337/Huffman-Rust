mod huffman;
mod file_system;
mod lz77;

use std::fs;

use huffman::{HuffmanResult, HuffmanTree};
use serde::{Deserialize, Serialize};

fn main() {

    let start = std::time::Instant::now();

    let contents = "Hallo Welt! Dies ist ein Test. Ich hoffe, dassfasa1234567,8,91234567890123123123123123";

    let encrypted = HuffmanTree::encrypt(&contents);

    fs::write("encrypted.tmy", encrypted.serialize()).unwrap();

    let entcrypt_time = std::time::Instant::now();

    let encrypted_file_contents = fs::read("encrypted.tmy").unwrap();

    let encrypted = HuffmanResult::deserialize(&encrypted_file_contents)
;
    let decrypted = HuffmanTree::decrypt(&encrypted);
    fs::write("decrypted.txt", &decrypted).unwrap();

    let decrypt_time = std::time::Instant::now();

    println!("Decrypted: {:?}", decrypted.chars().into_iter().take(100).collect::<String>());
    assert!(decrypted == contents, "Decrypted content is not the same as the original content");
    println!("Encryption time: {:?}", entcrypt_time.duration_since(start));
    println!("Decryption time: {:?}", decrypt_time.duration_since(entcrypt_time));


    let input = fs::read("input.txt").unwrap();
    let lz77 = lz77::LZ77::encode(&input);
    fs::write("compressed.tmy", lz77.serialize()).unwrap();
    let compressed = fs::read("compressed.tmy").unwrap();
    let deserialized = lz77::LZ77::deserialize(&compressed);
    let decoded = deserialized.decode();
    // println!("{:?}", decoded);
    assert!(input == decoded, "Decoded content is not the same as the original content");

}

