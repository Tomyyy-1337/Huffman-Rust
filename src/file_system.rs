use std::fs;

use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use serde::{Serialize, Deserialize};

use crate::{huffman, lz77};

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

#[derive(Serialize, Deserialize, Debug)]
pub enum FileData {
    Huffman {
        data: huffman::Huffman,
    },
    LZ77_24Huffman {
        data: huffman::Huffman,
        chunk_sizes: Vec<u32>,
    },
    LZ77_24 {
        data: lz77::LZ77,
    },
    LZ77_16Huffman {
        data: huffman::Huffman,
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
        let data_len = data.len();
        
        let lz16 = lz77::LZ77::encode(&data, 2);
        let mut compressions = vec![
            FileData::LZ77_16Huffman { 
                data: huffman::Huffman::encrypt(&lz16.data), 
                chunk_sizes: lz16.chunk_sizes.clone() 
            },
            FileData::LZ77_16 { data: lz16 },
            FileData::Huffman { data: huffman::Huffman::encrypt(&data) },
        ];
        if data_len > u16::MAX as usize {
            let lz77_24 = lz77::LZ77::encode(&data, 3);
            compressions.push(FileData::LZ77_24Huffman {
                data: huffman::Huffman::encrypt(&lz77_24.data),
                chunk_sizes: lz77_24.chunk_sizes.clone(),
            });
            compressions.push(FileData::LZ77_24 { data: lz77_24 });
        }
        compressions.push(FileData::Binary { data });

        compressions.into_iter()
            .min_by_key(|c| c.size())
            .unwrap()
    }

    fn size (&self) -> usize {
        match self {
            FileData::LZ77_24Huffman { data, chunk_sizes } => data.serialize().len() + chunk_sizes.len() * 4,
            FileData::LZ77_16Huffman { data, chunk_sizes } => data.serialize().len() + chunk_sizes.len() * 4,
            FileData::LZ77_24 { data } => data.data.len() + data.chunk_sizes.len() * 4,
            FileData::LZ77_16 { data } => data.data.len() + data.chunk_sizes.len() * 4,
            FileData::Huffman { data } => data.serialize().len(),
            FileData::Binary { data } => data.len(),
        }
    }

    pub fn decode(&self) -> Vec<u8> {
        match self {
            FileData::LZ77_24Huffman { data, chunk_sizes } => lz77::LZ77 {
                data: data.decrypt(),
                chunk_sizes: chunk_sizes.clone(),
            }.decode(3),
            FileData::LZ77_16Huffman { data, chunk_sizes } => lz77::LZ77 {
                data: data.decrypt(),
                chunk_sizes: chunk_sizes.clone(),
            }.decode(2),
            FileData::LZ77_24 { data } => data.decode(3),
            FileData::LZ77_16 { data } => data.decode(2),
            FileData::Huffman { data } => data.decrypt(),
            FileData::Binary { data } => data.clone(),
        }
    }
}