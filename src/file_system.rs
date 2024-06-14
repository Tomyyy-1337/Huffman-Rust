use std::fs;

use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use serde::{Serialize, Deserialize};

use crate::{huffman, lz77};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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
        let full_path = fs::canonicalize(path).unwrap();
        let dir_name = full_path.file_name().unwrap().to_str().unwrap();
        if full_path.is_file() {
            return Self::File {
                name: dir_name.to_string(),
                content: FileData::read_and_encode(path),
            }
        }

        let children = fs::read_dir(path).unwrap()
            .map(|entry| entry.unwrap().path().file_name().unwrap().to_str().unwrap().to_string())
            .filter(|child_path| !child_path.ends_with(".tmy"))
            .par_bridge()
            .map(|child_path| Self::read_directory(&(path.to_string() + "/" + &child_path)))
            .collect::<Vec<_>>();

        Self::Directory {
            name: dir_name.to_string(),
            children,
        }
    }

    pub fn write_directory(&self, path: &str) {
        match self {
            Archive::File { name, content } if fs::metadata(format!("{}/{}", path, name)).is_err() => {
                let decoded = content.decode();
                fs::write(path.to_string() + "/" + name, &decoded).unwrap();
            },
            Archive::Directory { name, children } if fs::metadata(format!("{}/{}", path, name)).is_err() => {
                fs::create_dir(path.to_string() + "/" + name).unwrap_or(());
                children.par_iter().for_each(|child| child.write_directory(&(path.to_string() + "/" + name)));
            },
            Archive::File { name, .. } | Archive::Directory { name, .. } => println!("{} existiert bereits", name),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum FileData {
    Huffman {
        data: huffman::Huffman,
    },
    LZ77Huffman {
        data: huffman::Huffman,
        bits: u8,
    },
    LZ77 {
        data: lz77::LZ77,
        bits: u8,
    },
    Binary {
        data: Vec<u8>,
    },
}

impl FileData {
    pub fn read_and_encode(path: &str) -> Self {
        let data = std::fs::read(path).unwrap();
        
        let num_bits = (data.len() + 1).ilog2();

        let num_bits = num_bits.max(2).min(24) as u8;

        let lz77: lz77::LZ77 = lz77::LZ77::encode(&data, num_bits);
        let compressions = vec![
            FileData::LZ77Huffman { 
                data: huffman::Huffman::encrypt(&lz77.serialize()), 
                bits: num_bits,
            },
            FileData::LZ77 { data: lz77, bits: num_bits },
            FileData::Huffman { data: huffman::Huffman::encrypt(&data) },
            FileData::Binary { data },
        ];

        compressions.into_iter()
            .min_by_key(|c| c.size())
            .unwrap()
    }

    fn size (&self) -> usize {
        match self {
            FileData::LZ77Huffman { data, .. } => data.serialize().len(),
            FileData::LZ77 { data, .. } => data.serialize().len(),
            FileData::Huffman { data } => data.serialize().len(),
            FileData::Binary { data } => data.len(),
        }
    }

    pub fn decode(&self) -> Vec<u8> {
        match self {
            FileData::LZ77Huffman { data, bits } => lz77::LZ77::deserialize(&data.decrypt()).decode(*bits),
            FileData::LZ77 { data, bits } => data.clone().decode(*bits),
            FileData::Huffman { data } => data.decrypt(),
            FileData::Binary { data } => data.clone(),
        }
    }
}