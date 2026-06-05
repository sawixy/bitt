mod protocol;

use protocol::file::TorrentFile;
use protocol::bencode::{Bencode, Entry};
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut f = Bencode::new();
    f.parse(tokio::fs::read("archlinux-2026.06.01-x86_64.iso.torrent").await?).await?;
    println!("{:?}", f);
    let file = TorrentFile::from_bencode(&f)?;
    println!("{:#?}", file);
    Ok(())
}