mod huffman;

use std::thread::sleep;

use huffman::Huffman;

fn main() {
    let start = std::time::Instant::now();

    let contents = std::fs::read_to_string("input.txt").unwrap();

    let read_time = std::time::Instant::now();
    
    let encrypted = Huffman::encrypt(&contents);

    let encrypt_time = std::time::Instant::now();

    std::fs::write("encrypted.txt", encrypted.clone()).unwrap();

    let write_time = std::time::Instant::now();

    let contents = std::fs::read_to_string("encrypted.txt").unwrap();

    let read_encrypted_time = std::time::Instant::now();

    let result = Huffman::decrypt(contents);

    let decrypt_time = std::time::Instant::now();

    println!("Read: {:?}\nEncrypt: {:?}\nWrite: {:?}\nRead Encrypted: {:?}\nDecrypt: {:?}", 
        read_time.duration_since(start),
        encrypt_time.duration_since(read_time),
        write_time.duration_since(encrypt_time),
        read_encrypted_time.duration_since(write_time),
        decrypt_time.duration_since(read_encrypted_time),
    );

    println!("Decrypted: {}", result.chars().take(200).collect::<String>());

}

