use std::collections::HashMap;
use tokio::fs;

#[derive(Clone, Debug)]
pub enum Entry {
    String(Vec<u8>),
    Integer(i64),
    List(Vec<Entry>),
    Dict(HashMap<String, Entry>)
}

impl Entry {
    pub fn as_string(&self) -> Option<Vec<u8>> {
        match self {
            Entry::String(str) => Option::from(str.clone()),
            _ => None,
        }
    }
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Entry::Integer(int) => Option::from(*int),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<Vec<Entry>> {
        match self {
            Entry::List(list) => Option::from(list.clone()),
            _ => None,
        }
    }
    pub fn as_dict(&self) -> Option<HashMap<String, Entry>> {
        match self {
            Entry::Dict(dict) => Option::from(dict.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct File {
    value: Entry,
}

impl File {
    pub fn new() -> Self {
        Self {
            value: Entry::Dict(HashMap::new())
        }
    }

    fn parse(block: &[u8], pos: &mut usize) -> Result<Entry, Box<dyn std::error::Error>> {
        if *pos >= block.len() {
            return Err(Box::from("Unexpected end of data"));
        }
        
        if block[*pos] == b'i' {
            *pos += 1;
            let start = *pos;
            let mut len: usize = 0;
            
            while *pos + len < block.len() && block[*pos + len] != b'e' {
                if block[*pos + len] < b'0' || block[*pos + len] > b'9' {
                    return Err(Box::from("Invalid integer"));
                }
                len += 1;
            }
            
            if *pos + len >= block.len() {
                return Err(Box::from("Unexpected end of data in integer"));
            }
            
            let mut num = 0;
            for i in 0..len {
                num = num * 10 + (block[*pos + i] - b'0') as i64;
            }
            *pos += len + 1;
            
            Ok(Entry::Integer(num))
            
        } else if block[*pos] == b'l' {
            *pos += 1;
            let mut list: Vec<Entry> = Vec::new();
            
            while *pos < block.len() && block[*pos] != b'e' {
                list.push(Self::parse(block, pos)?);
            }
            
            if *pos < block.len() {
                *pos += 1;
            } else {
                return Err(Box::from("Unexpected end of data in list"));
            }
            
            Ok(Entry::List(list))
            
        } else if block[*pos] == b'd' {
            let mut dict: HashMap<String, Entry> = HashMap::new();
            *pos += 1;
            
            while *pos < block.len() && block[*pos] != b'e' {
                let key = Self::parse(block, pos)?.as_string();
                let value = Self::parse(block, pos)?;
                
                if let Some(s) = key {
                    let key_str = String::from_utf8(s)?;
                    dict.insert(key_str, value);
                }
            }
            
            if *pos < block.len() {
                *pos += 1;
            } else {
                return Err(Box::from("Unexpected end of data in dictionary"));
            }
            
            Ok(Entry::Dict(dict))
            
        } else if block[*pos] >= b'0' && block[*pos] <= b'9' {
            let start = *pos;
            let mut len = 0;
            
            while *pos + len < block.len() && block[*pos + len] != b':' {
                if block[*pos + len] < b'0' || block[*pos + len] > b'9' {
                    return Err(Box::from("Invalid string length"));
                }
                len += 1;
            }
            
            if *pos + len >= block.len() {
                return Err(Box::from("Unexpected end of data, missing ':'"));
            }
            
            let mut strlen: usize = 0;
            for i in 0..len {
                strlen = strlen * 10 + (block[start + i] - b'0') as usize;
            }
            
            *pos += len + 1;
            
            if *pos + strlen > block.len() {
                return Err(Box::from("String length exceeds data boundaries"));
            }
            
            let result = Vec::from(&block[*pos..*pos + strlen]);
            *pos += strlen;
            
            Ok(Entry::String(result))
            
        } else {
            Err(Box::from(format!("Unexpected character: {}", block[*pos] as char)))
        }
    }

    pub async fn load(&mut self, path: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = fs::read(path).await?;

        let mut pos: usize = 0;

        self.value = Self::parse(content.as_slice(), &mut pos)?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Entry> {
        let dict = self.value.as_dict()?;
        match dict.get(key) {
            Some(entry) => Some(entry.clone()),
            _ => None,
        }
    }
}