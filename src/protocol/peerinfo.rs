use std::collections::HashMap;

use super::bencode::Entry;

pub struct PeerInfo {
    id: Vec<u8>,  // 20 bytes
    ip: String,
    port: u16,
}

impl PeerInfo {
    pub fn new() -> PeerInfo {
        Self {
            id: Vec::new(),
            ip: String::new(),
            port: 0,
        }
    }

    pub fn from_bencode(entry: &Entry) -> Result<Self, Box<dyn std::error::Error>> {
        let dict = entry.as_dict().ok_or("Expected a dictionary for peer entry")?;
        let id = dict["peer id"].as_string().ok_or("Expected 'peer id' to be a string")?;
        let mut ip: String  = String::new();
        let ip_bytes = dict["ip"].as_string().ok_or("Expected 'ip' to be a string")?.iter().map(|b| ip.push(*b as char));
        let port = dict["port"].as_int().ok_or("Expected 'port' to be an integer")? as u16;
        Ok(Self { id, ip, port })
    }

    pub fn to_bencode(&self) -> Entry {
        let mut dict = HashMap::new();
        dict.insert("peer id".to_string(), Entry::String(self.id.clone()));
        dict.insert("ip".to_string(), Entry::String(self.ip.as_bytes().to_vec()));
        dict.insert("port".to_string(), Entry::Integer(self.port as i64));
        Entry::Dict(dict)
    }

    pub fn get_id(&self) -> &[u8] {
        &self.id
    }

    pub fn get_ip(&self) -> &String {
        &self.ip
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}