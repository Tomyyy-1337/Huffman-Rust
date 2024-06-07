mod huffman;
mod file_system;

use std::fs;

use huffman::{HuffmanResult, HuffmanTree};

fn main() {

    let start = std::time::Instant::now();

    let contents = fs::read_to_string("bibel.txt").unwrap();

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

    // let test = (0..1000000000).map(|i| (i % 26 + 65) as u8 as char).collect::<String>();
    // fs::write("test.txt", &test).unwrap();


}

