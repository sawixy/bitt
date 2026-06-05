pub struct TorrentFile {
    pub announce: String,
    pub anounce_list: Vec<String>,
    pub comment: String,
    pub created_by: String,
    pub creation_date: i64,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
}

impl TorrentFile {
    pub fn from_bencode(bencode: &Bencode) -> Option<Self> {
        let announce = bencode.get("announce")?.as_string()?;
        let anounce_list = bencode.get("announce-list")?.as_list()?;
        let comment = bencode.get("comment")?.as_string()?;
        let created_by = bencode.get("created by")?.as_string()?;
        let creation_date = bencode.get("creation date")?.as_integer()?;
        let piece_length = bencode.get("info")?.as_dict()?.get("piece length")?.as_integer()?;
        let pieces = bencode.get("info")?.as_dict()?.get("pieces")?.as_string()?;

        Some(Self {
            announce: String::from_utf8(announce).ok()?,
            anounce_list: anounce_list.iter().filter_map(|entry| entry.as_string().and_then(|s| String::from_utf8(s).ok())).collect(),
            comment: String::from_utf8(comment).ok()?,
            created_by: String::from_utf8(created_by).ok()?,
            creation_date,
            piece_length,
            pieces,
        })
    }

    pub fn into_bencode(&self) -> Bencode {
        let mut bencode = Bencode::new();
        let mut info_dict = HashMap::new();

        info_dict.insert("piece length".to_string(), Entry::Integer(self.piece_length));
        info_dict.insert("pieces".to_string(), Entry::String(self.pieces.clone()));

        bencode.value = Entry::Dict(vec![
            ("announce".to_string(), Entry::String(self.announce.clone())),
            ("announce-list".to_string(), Entry::List(self.anounce_list.iter().map(|s| Entry::String(s.as_bytes().to_vec())).collect())),
            ("comment".to_string(), Entry::String(self.comment.clone())),
            ("created by".to_string(), Entry::String(self.created_by.clone())),
            ("creation date".to_string(), Entry::Integer(self.creation_date)),
            ("info".to_string(), Entry::Dict(info_dict)),
        ].into_iter().collect());

        bencode
    }
}