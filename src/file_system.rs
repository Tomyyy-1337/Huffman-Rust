use crate::huffman::{HuffmanResult, HuffmanTree};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileSystem {
    root: Element,
}

impl FileSystem {
    pub fn compress(name: &str) {
        let filesystem =FileSystem{
            root: Element::Directory {
                name: "unpacked".to_string(),
                children: vec![Element::read_from_path("./")],
            },
        };
        let serialized = bincode::serialize(&filesystem).unwrap();
        std::fs::write(name, serialized).unwrap();
    }

    pub fn decompress(name: &str) {
        let serialized = std::fs::read(name).unwrap();
        let filesystem: FileSystem = bincode::deserialize(&serialized).unwrap();
            
    }

}

#[derive(Serialize, Deserialize, Debug)]
enum Element {
    File{
        name: String,
        content: File,
    },
    Directory{
        name: String,
        children: Vec<Element>,
    },
}

impl Element {
    fn read_from_path(path: &str) -> Self {
        let metadata = std::fs::metadata(path).unwrap();
        if metadata.is_file() {
            return Element::File{
                name: path.to_string(),
                content: File::read_from_path(path),
            };
        }
        let children = std::fs::read_dir(path).unwrap()
            .map(|entry| Element::read_from_path(&entry.unwrap().path().to_string_lossy()))
            .collect();
        Element::Directory{
            name: path.to_string(),
            children,
        }
    }



    
}

#[derive(Serialize, Deserialize, Debug)]
enum File {
    Text{
        compressed_text: HuffmanResult,
    },
    Binary {
        data: Vec<u8>,
    },
}

impl File {
    fn read_from_path(path: &str) -> Self {
        if path.ends_with(".txt") || path.ends_with(".rs") || path.ends_with(".toml") || path.ends_with(".meta") {
            let contents = std::fs::read_to_string(path).unwrap();
            return File::Text{
                compressed_text: HuffmanTree::encrypt(&contents)
            };
        } 
        File::Binary{
            data: std::fs::read(path).unwrap()
        }
    }

    fn decompress(self) -> Vec<u8> {
        match self {
            File::Text{compressed_text} => HuffmanTree::decrypt(&compressed_text).into_bytes(),
            File::Binary{data} => data,
        }
    }

}