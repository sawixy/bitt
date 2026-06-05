use super::bencode::{Entry, Bencode};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum TorrentFileType {
    Magnet, // V1 without trackers (DHT, PEX, LPD)
    V1,
    V2,
    V1V2,
}

#[derive(Debug)]
pub struct TorrentFile {
    pub file_type: TorrentFileType,
    pub files: Vec<(String, u64)>,    // (path, size)
    pub trackers: Vec<String>,        // For V1 and V1V2
    pub info: HashMap<String, Entry>, // The "info" dictionary for V1 and V2
    pub piece_length: u64,
    pub pieces: Vec<[u8; 20]>,        // 20 bytes for each piece (V1)
    pub pieces_v2: Option<Vec<Vec<u8>>>, // For V2 (Merkle tree or piece hashes)
    pub info_hash: Vec<u8>,           // 20 bytes for V1 (SHA-1)
    pub info_hash_v2: Option<Vec<u8>>, // 32 bytes for V2 (SHA-256)
    pub name: String,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    pub creation_date: Option<u64>,
    pub url_list: Option<Vec<String>>, // Web seeds (HTTP/HTTPS mirrors)
}

impl TorrentFile {
    pub fn from_bencode(bencode: &Bencode) -> Result<Self, Box<dyn std::error::Error>> {
        let dict = bencode.value.as_dict().ok_or("Top-level bencode is not a dictionary")?;

        // Parse optional fields
        let comment = dict.get("comment")
            .and_then(|c| c.as_string())
            .map(|c| String::from_utf8_lossy(&c).to_string());
            
        let created_by = dict.get("created by")
            .and_then(|c| c.as_string())
            .map(|c| String::from_utf8_lossy(&c).to_string());
            
        let creation_date = dict.get("creation date")
            .and_then(|c| c.as_int())
            .map(|c| c as u64);
        
        // Parse url-list (web seeds)
        let url_list = if let Some(url_list_entry) = dict.get("url-list") {
            match url_list_entry {
                Entry::List(list) => {
                    let urls: Vec<String> = list.iter()
                        .filter_map(|e| e.as_string())
                        .map(|s| String::from_utf8_lossy(&s).to_string())
                        .collect();
                    if urls.is_empty() { None } else { Some(urls) }
                }
                Entry::String(s) => Some(vec![String::from_utf8_lossy(s).to_string()]),
                _ => None,
            }
        } else {
            None
        };

        // Parse trackers (announce + announce-list)
        let mut trackers = Vec::new();
        
        if let Some(announce) = dict.get("announce") {
            if let Some(announce_str) = announce.as_string() {
                trackers.push(String::from_utf8_lossy(&announce_str).to_string());
            }
        }
        
        if let Some(announce_list) = dict.get("announce-list") {
            if let Some(list_of_lists) = announce_list.as_list() {
                for tier in list_of_lists {
                    if let Some(tier_list) = tier.as_list() {
                        for tracker in tier_list {
                            if let Some(tracker_str) = tracker.as_string() {
                                trackers.push(String::from_utf8_lossy(&tracker_str).to_string());
                            }
                        }
                    }
                }
            }
        }
        
        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        trackers.retain(|t| seen.insert(t.clone()));

        // Get info dictionary
        let info_dict = dict.get("info").ok_or("Missing 'info' dictionary")?;
        let info = info_dict.as_dict().ok_or("'info' is not a dictionary")?;
        
        // Check for V2
        let is_v2 = info.contains_key("meta version") || info.contains_key("file tree");
        let is_v1 = info.contains_key("pieces");
        
        let file_type = match (is_v1, is_v2, trackers.is_empty()) {
            (true, true, _) => TorrentFileType::V1V2,
            (true, false, true) => TorrentFileType::Magnet,
            (true, false, false) => TorrentFileType::V1,
            (false, true, _) => TorrentFileType::V2,
            _ => return Err(Box::from("Unknown or invalid torrent file format")),
        };
        
        // Parse piece length
        let piece_length = info.get("piece length")
            .ok_or("Missing 'piece length'")?
            .as_int()
            .ok_or("'piece length' is not an integer")? as u64;
        
        // Parse name
        let name = info.get("name")
            .ok_or("Missing 'name' in info dictionary")?
            .as_string()
            .ok_or("'name' is not a string")?;
        let name = String::from_utf8_lossy(&name).to_string();
        
        // Parse V1 pieces (if present)
        let pieces = if is_v1 {
            let pieces_str = info.get("pieces")
                .ok_or("Missing 'pieces' for V1")?
                .as_string()
                .ok_or("'pieces' is not a string")?;
            if pieces_str.len() % 20 != 0 {
                return Err(Box::from("'pieces' length is not a multiple of 20"));
            }
            pieces_str.chunks(20).map(|chunk| {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(chunk);
                arr
            }).collect()
        } else {
            Vec::new()
        };
        
        // Parse V2 pieces (if present)
        let pieces_v2 = if is_v2 {
            let mut v2_pieces = Vec::new();
            if let Some(pieces_root) = info.get("pieces root") {
                if let Some(root_hash) = pieces_root.as_string() {
                    v2_pieces.push(root_hash.to_vec());
                }
            }
            if let Some(file_tree) = info.get("file tree") {
                // For full V2 support, we'd need to recursively parse the file tree
                // This is a simplified version
                v2_pieces.push(vec![0]); // Placeholder
            }
            Some(v2_pieces)
        } else {
            None
        };
        
        // Parse files
        let files = if let Some(files_list) = info.get("files") {
            // Multi-file torrent
            let files_list = files_list.as_list().ok_or("'files' is not a list")?;
            let mut result = Vec::new();
            
            for file_entry in files_list {
                let file_dict = file_entry.as_dict().ok_or("File entry is not a dictionary")?;
                let length = file_dict.get("length")
                    .ok_or("Missing 'length' in file entry")?
                    .as_int()
                    .ok_or("'length' is not an integer")? as u64;
                
                let path_list = file_dict.get("path")
                    .ok_or("Missing 'path' in file entry")?
                    .as_list()
                    .ok_or("'path' is not a list")?;
                
                let mut path_parts = Vec::new();
                for part in path_list {
                    let part_str = part.as_string().ok_or("Path component is not a string")?;
                    path_parts.push(String::from_utf8_lossy(&part_str).to_string());
                }
                let path = path_parts.join("/");
                result.push((path, length));
            }
            result
        } else {
            // Single-file torrent
            let length = info.get("length")
                .ok_or("Missing 'length' in info dictionary for single-file torrent")?
                .as_int()
                .ok_or("'length' is not an integer")? as u64;
            vec![(name.clone(), length)]
        };
        
        // Calculate info hash (V1 - SHA-1 of encoded info dict)
        let info_hash = if is_v1 {
            // Create a new Bencode object with just the info dict
            let info_bencode = Bencode {
                value: Entry::Dict(info.clone()),
            };
            let encoded_info = info_bencode.format(); // Use the format() method from your API
            
            use sha1::{Sha1, Digest};
            let mut hasher = Sha1::new();
            hasher.update(&encoded_info);
            hasher.finalize().to_vec()
        } else {
            Vec::new()
        };
        
        // Calculate info hash V2 (SHA-256)
        let info_hash_v2 = if is_v2 {
            let info_bencode = Bencode {
                value: Entry::Dict(info.clone()),
            };
            let encoded_info = info_bencode.format();
            
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&encoded_info);
            Some(hasher.finalize().to_vec())
        } else {
            None
        };
        
        Ok(TorrentFile {
            file_type,
            files,
            trackers,
            info: info.clone(),
            piece_length,
            pieces,
            pieces_v2,
            info_hash,
            info_hash_v2,
            name,
            comment,
            created_by,
            creation_date,
            url_list,
        })
    }
    
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|(_, size)| size).sum()
    }
    
    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }
    
    pub fn is_magnet_only(&self) -> bool {
        matches!(self.file_type, TorrentFileType::Magnet)
    }
    
    pub fn supports_v1(&self) -> bool {
        matches!(self.file_type, TorrentFileType::V1 | TorrentFileType::V1V2 | TorrentFileType::Magnet)
    }
    
    pub fn supports_v2(&self) -> bool {
        matches!(self.file_type, TorrentFileType::V2 | TorrentFileType::V1V2)
    }
}