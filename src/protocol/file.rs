use std::collections::HashMap;
use tokio::fs;

#[derive(Clone, Debug)]
pub enum Entry {
    String(String),
    Integer(i64),
    List(Vec<Entry>),
    Dict(HashMap<String, Entry>)
}

impl Entry {
    pub fn as_string(&self) -> Option<String> {
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
        println!("{}", std::str::from_utf8(&block[*pos..])?);
        if block[*pos] == b'i' {
            println!("Integer");
            *pos += 1;
            let mut len: usize = 0;
            while block[*pos+len] != b'e' {
                len += 1;
            }

            let str = std::str::from_utf8(&block[*pos..*pos+len]).unwrap();
            *pos += len+1;
            
            return Ok(Entry::Integer(str.parse()?));
        } else if block[*pos] == b'l' {
            println!("List");
            *pos += 1;
            let mut list: Vec<Entry> = Vec::new();
            loop {
                if block[*pos] != b'e' {
                    list.push(Self::parse(block, pos)?)
                } else {
                    break;
                }
            }
            return Ok(Entry::List(list));
        } else if block[*pos] == b'd' {
            println!("Dictionary");
            let mut dict: HashMap<String, Entry> = HashMap::new();
            *pos += 1;
            loop {
                if block[*pos] != b'e' {
                    let key = Self::parse(block, pos)?.as_string();
                    let value = Self::parse(block, pos)?;
                    let key = match key {
                        Some(s) => s,
                        None => continue,
                    };

                    dict.insert(key, value); 
                } else {
                    break;
                }
            }
            return Ok(Entry::Dict(dict));
        } else {
            println!("String");
            let mut len: usize = 0;
            while block[*pos+len] != b':' {
                len += 1;
            }
            let str = std::str::from_utf8(&block[*pos..*pos+len]).unwrap();
            *pos += len + 1;
            let len: usize= str.parse()?;
            *pos += len;
            return Ok(Entry::String(String::from(std::str::from_utf8(&block[*pos-len..*pos])?)));
        }
    }

    pub async fn load(&mut self, path: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = fs::read(path).await?;
        content.retain(|&b| b != b'\n' && b != b'\r');

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