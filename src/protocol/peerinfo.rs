use std::collections::HashMap;

use super::bencode::Entry;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    id: Option<Vec<u8>>,  // 20 bytes
    ip: String,
    port: u16,
}

impl PeerInfo {
    pub fn new(id: Option<Vec<u8>>, ip: String, port: u16) -> PeerInfo {
        Self {
            id: id,
            ip: ip,
            port: port,
        }
    }

    pub fn set_ip(&mut self, ip: String) {
        self.ip = ip;
    }

    pub fn set_id(&mut self, id: Vec<u8>) {
        self.id = Some(id);
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn from_bencode(entry: &Entry) -> Result<Self, Box<dyn std::error::Error>> {
        let dict = entry.as_dict().ok_or("Expected a dictionary for peer entry")?;
        let id = dict.get("peer id").unwrap_or(&Entry::String(Vec::new())).as_string();
        let mut ip: String  = String::new();
        dict.get("ip").ok_or("Expected 'ip'")?.as_string().ok_or("Expected 'ip' to be a string")?.iter().for_each(|b| ip.push(*b as char));
        let port = dict.get("port").ok_or("Expected 'port'")?.as_int().ok_or("Expected 'port' to be an integer")? as u16;
        Ok(Self { id, ip, port })
    }

    pub fn to_bencode(&self) -> Entry {
        let mut dict = HashMap::new();
        if let Some(id) = &self.id {dict.insert("peer id".to_string(), Entry::String(id.to_vec())); }
        dict.insert("ip".to_string(), Entry::String(self.ip.as_bytes().to_vec()));
        dict.insert("port".to_string(), Entry::Integer(self.port as i64));
        Entry::Dict(dict)
    }

    pub fn get_id(&self) -> &Option<Vec<u8>> {
        &self.id
    }

    pub fn get_ip(&self) -> &String {
        &self.ip
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}