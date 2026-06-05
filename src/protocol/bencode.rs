use std::collections::HashMap;

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
pub struct Bencode {
    value: Entry,
}

impl Bencode {
    pub fn new() -> Self {
        Self {
            value: Entry::Dict(HashMap::new())
        }
    }

    fn parse_block(block: &[u8], pos: &mut usize) -> Result<Entry, Box<dyn std::error::Error>> {
        if *pos >= block.len() {
            return Err(Box::from("Unexpected end of data"));
        }
        
        if block[*pos] == b'i' {
            *pos += 1;
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
                list.push(Self::parse_block(block, pos)?);
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
                let key = Self::parse_block(block, pos)?.as_string();
                let value = Self::parse_block(block, pos)?;
                
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

    pub async fn parse(&mut self, content: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let mut pos: usize = 0;

        self.value = Self::parse_block(content.as_slice(), &mut pos)?;

        Ok(())
    }

    fn format_block(entry: Entry) -> Vec<u8> {
        match entry {
            Entry::Integer(int) => {
                let mut res = vec![b'i'];
                res.extend_from_slice(int.to_string().as_bytes());
                res.push(b'e');
                return res;
            },
            Entry::String(str) => {
                let mut res = Vec::new();
                res.extend_from_slice(str.len().to_string().as_bytes());
                res.push(b':');
                res.extend_from_slice(&str);
                return res;
            },
            Entry::List(list) => {
                let mut res = vec![b'l'];
                for item in list {
                    Self::format_block(item).iter().for_each(|b| { res.push(*b); });
                }
                res.push(b'e');
                return res;
            },
            Entry::Dict(dict) => {
                let mut res = vec![b'd'];
                for (key, value) in dict {
                    Self::format_block(Entry::String(key.into_bytes())).iter().for_each(|b| { res.push(*b); });
                    Self::format_block(value).iter().for_each(|b| { res.push(*b); });
                }
                res.push(b'e');
                return res;
            },
            _ => Vec::new(),
        }
    }

    pub fn format(&self) -> Vec<u8> {
        Self::format_block(self.value.clone())
    }

    pub fn get(&self, key: &str) -> Option<Entry> {
        let dict = self.value.as_dict()?;
        match dict.get(key) {
            Some(entry) => Some(entry.clone()),
            _ => None,
        }
    }
}