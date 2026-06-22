mod protocol;
mod client;
use crate::client::app::render;
use crate::protocol::connection::TcpConnection;
use protocol::session::Session;
use protocol::file::TorrentFile;
use protocol::bencode::Bencode;
use protocol::storage::FileStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let torrent_data = tokio::fs::read("test.torrent").await?;
    let mut bencoder = Bencode::new();
    bencoder.parse(torrent_data.to_vec()).await?;
    let file = TorrentFile::from_bencode(&mut bencoder)?;
    
    let storage = FileStorage::create(
        "downloaded_file",
        file.total_size(),
        file.piece_length as u64,
        file.piece_count()
    ).await?;
    
    let mut session = Session::new(file, storage);
    Ok(())
}