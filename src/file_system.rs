use std::fs;

use rayon::{iter::{IntoParallelIterator, ParallelIterator}, vec};
use serde::{Serialize, Deserialize};

use crate::{huffman_byte, lz77};

#[derive(Serialize, Deserialize, Debug)]
pub enum Archive {
    File{
        name: String,
        content: FileData,
    },
    Directory{
        name: String,
        children: Vec<Archive>,
    },
}

impl Archive {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn deserialize(input: &[u8]) -> Self {
        bincode::deserialize(input).unwrap()
    }

    pub fn get_name(&self) -> String {
        match self {
            Archive::File { name, .. } => name.clone(),
            Archive::Directory { name, .. } => name.clone(),
        }
    }

    pub fn read_directory(path: &str) -> Self {
        let full_path = std::fs::canonicalize(path).unwrap();
        let dir_name = full_path.file_name().unwrap().to_str().unwrap();
        if full_path.is_file() {
            return Self::File {
                name: dir_name.to_string(),
                content: FileData::read_and_encode(path),
            }
        }

        let children_paths = fs::read_dir(path).unwrap()
            .map(|entry| entry.unwrap().path().file_name().unwrap().to_str().unwrap().to_string())
            .collect::<Vec<_>>();

        let children = children_paths.into_par_iter()
            .map(|child_path| Self::read_directory(&(path.to_string() + "/" + &child_path)))
            .collect::<Vec<_>>();

        Self::Directory {
            name: dir_name.to_string(),
            children,
        }

    }

    pub fn write_directory(&self, path: &str) {
        match self {
            Archive::File { name, content } => {
                let decoded = content.decode();
                fs::write(path.to_string() + "/" + name, &decoded).unwrap();
            },
            Archive::Directory { name, children } => {
                fs::create_dir(path.to_string() + "/" + name).unwrap_or(());
                children.iter().for_each(|child| child.write_directory(&(path.to_string() + "/" + name)));
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FileData {
    LZ77_24Huffman {
        data: huffman_byte::Huffman,
        chunk_sizes: Vec<u32>,
    },
    LZ77_16 {
        data: lz77::LZ77,
    },
    Binary {
        data: Vec<u8>,
    },
}

impl FileData {
    pub fn read_and_encode(path: &str) -> Self {
        let data = std::fs::read(path).unwrap();
        let compressed = if data.len() < u16::MAX as usize {
            FileData::LZ77_16 { data: lz77::LZ77::encode(&data, 2) }
        } else {
            let lz77 = lz77::LZ77::encode(&data, 3);
            let huffman = huffman_byte::Huffman::encrypt(&lz77.data);
            FileData::LZ77_24Huffman { data: huffman, chunk_sizes: lz77.chunk_sizes }
        };
        if compressed.size() < data.len() {
            compressed
        } else {
            FileData::Binary { data }
        }
        
    }

    fn size (&self) -> usize {
        match self {
            FileData::LZ77_24Huffman { data, chunk_sizes } => data.serialize().len() + chunk_sizes.len() * 4,
            FileData::LZ77_16 { data } => data.data.len() + data.chunk_sizes.len() * 4,
            FileData::Binary { data } => data.len(),
        }
    }

    pub fn decode(&self) -> Vec<u8> {
        match self {
            FileData::LZ77_24Huffman { data, chunk_sizes } => {
                lz77::LZ77 {
                    data: data.decrypt(),
                    chunk_sizes: chunk_sizes.clone(),
                }.decode(3)
            },
            FileData::LZ77_16 { data } => data.decode(2),
            FileData::Binary { data } => data.clone(),
        }
    }
    
}