use std::fs;

use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use serde::{Serialize, Deserialize};

use crate::{huffman::{self, HuffmanTree}, lz77};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Archive {
    File{
        name: String,
        content: FileData,
    },
    Directory{
        name: String,
        children: Vec<Archive>,
    },
    Root {
        name: String,
        children: Vec<Archive>,
        tree: huffman::HuffmanTree,
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
            Archive::Root { name, .. } => name.clone(),
        }
    }

    fn count_chars(path: &str) -> [u64; 256] {
        let full_path = fs::canonicalize(path).unwrap();
        if full_path.is_file() {
            let file = fs::read(path).unwrap();
            if file.len() > 5000 {
                return [0; 256];
            }
            let mut char_counts = [0; 256];
            for &c in &file {
                char_counts[c as usize] += 1;
            }
            return char_counts;
        }

        fs::read_dir(path).unwrap()
            .map(|entry| entry.unwrap().path().file_name().unwrap().to_str().unwrap().to_string())
            .filter(|child_path| !child_path.ends_with(".tmy"))
            .map(|child_path| Self::count_chars(&(path.to_string() + "/" + &child_path)))
            .fold([0; 256], |mut acc, counts| {
                for i in 0..256 {
                    acc[i] += counts[i];
                }
                acc
            })
        
    }

    fn contains_fixed_huffman(&self) -> bool {
        match self {
            Archive::File { content, .. } => matches!(content, FileData::FixedHuffman { .. }),
            Archive::Directory { children, .. } | Archive::Root { children, .. } => children.iter().any(|child| child.contains_fixed_huffman()),
        }
    }

    pub fn read_directory(path: &str) -> Self {
        let char_counts = Self::count_chars(path);
        let mut tree = huffman::HuffmanTree::from_counts(char_counts);
        let r = Self::read_directory_rec(path, &tree);
        match &r {
            Archive::Directory { name, children } => {
                if !r.contains_fixed_huffman() {
                    tree = HuffmanTree {
                        children: vec![],
                        character: None,
                    };
                    println!("No fixed huffman found");
                }
                Archive::Root { name: name.to_string(), children: children.clone(), tree }
            },
            _ => panic!("Root must be a directory"),        
        }

    }

    pub fn read_directory_rec(path: &str, tree: &HuffmanTree) -> Self {
        let full_path = fs::canonicalize(path).unwrap();
        let dir_name = full_path.file_name().unwrap().to_str().unwrap();
        if full_path.is_file() {
            return Self::File {
                name: dir_name.to_string(),
                content: FileData::read_and_encode(path, tree),
            }
        }

        let children = fs::read_dir(path).unwrap()
            .map(|entry| entry.unwrap().path().file_name().unwrap().to_str().unwrap().to_string())
            .filter(|child_path| !child_path.ends_with(".tmy"))
            .par_bridge()
            .map(|child_path| Self::read_directory_rec(&(path.to_string() + "/" + &child_path), tree))
            .collect::<Vec<_>>();

        Self::Directory {
            name: dir_name.to_string(),
            children,
        }
    }

    pub fn write_directory(&self, path: &str) {
        match self {
            Archive::Root { tree, .. } => {
                self.write_directory_rec(path, tree);            },
            _ => panic!("Root must be a directory"),
        }
    }

    pub fn write_directory_rec(&self, path: &str, tree: &HuffmanTree) {
        match self {
            Archive::File { name, content } if fs::metadata(format!("{}/{}", path, name)).is_err() => {
                let decoded = content.decode(tree);
                fs::write(path.to_string() + "/" + name, &decoded).unwrap();
            },
            Archive::Directory { name, children } | Archive::Root { name, children, .. } if fs::metadata(format!("{}/{}", path, name)).is_err() => {
                fs::create_dir(path.to_string() + "/" + name).unwrap_or(());
                children.par_iter().for_each(|child| child.write_directory_rec(&(path.to_string() + "/" + name), tree));
            },
            Archive::File { name, .. } | Archive::Directory { name, .. } | Archive::Root { name, .. } => println!("{} existiert bereits", name),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum FileData {
    FixedHuffman {
        data: huffman::HuffmanNoTree,
    },
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
    pub fn read_and_encode(path: &str, tree: &huffman::HuffmanTree) -> Self {
        let data = std::fs::read(path).unwrap();
        
        let num_bits = (data.len() + 1).ilog2();
        let num_bits = num_bits.max(2).min(24) as u8;

        let lz77: lz77::LZ77 = lz77::LZ77::encode(&data, num_bits);
        let mut compressions = vec![
            FileData::LZ77Huffman { 
                data: huffman::Huffman::encrypt(&lz77.serialize()), 
                bits: num_bits,
            },
            FileData::LZ77 { data: lz77, bits: num_bits },
            FileData::Huffman { data: huffman::Huffman::encrypt(&data) },
        ];

        if data.len() < 5000 {
            let fixed_huffman = huffman::HuffmanNoTree::encrypt(&data, tree);
            compressions.push(FileData::FixedHuffman { data: fixed_huffman });
        }
        compressions.push(FileData::Binary { data });

        let best_format = compressions.into_iter()
            .min_by_key(|c| c.size())
            .unwrap();

        // match &best_format {
        //     FileData::LZ77Huffman { bits, .. } => println!("Lz77Huffman with {} bits (Path: {})", bits, path),
        //     FileData::LZ77 { bits, .. } => println!("Lz77 with {} bits (Path: {})", bits, path),
        //     FileData::Huffman { .. } => println!("Huffman (Path: {})", path),
        //     FileData::Binary { .. } => println!("Binary (Path: {})", path),
        //     FileData::FixedHuffman { .. } => println!("FixedHuffman (Path: {})", path),
        // }

        best_format
    }

    fn size (&self) -> usize {
        match self {
            FileData::LZ77Huffman { data, .. } => data.serialize().len(),
            FileData::LZ77 { data, .. } => data.serialize().len(),
            FileData::Huffman { data } => data.serialize().len(),
            FileData::Binary { data } => data.len(),
            FileData::FixedHuffman { data } => data.serialize().len(),
        }
    }

    pub fn decode(&self, tree: &huffman::HuffmanTree) -> Vec<u8> {
        match self {
            FileData::LZ77Huffman { data, bits } => lz77::LZ77::deserialize(&data.decrypt()).decode(*bits),
            FileData::LZ77 { data, bits } => data.clone().decode(*bits),
            FileData::Huffman { data } => data.decrypt(),
            FileData::Binary { data } => data.clone(),
            FileData::FixedHuffman { data } => data.decrypt(tree),
        }
    }
}