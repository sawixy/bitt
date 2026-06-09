mod protocol;
mod client;
use crate::client::app::render;
use crate::protocol::connection::TcpConnection;
use protocol::session::Session;
use protocol::file::TorrentFile;
use protocol::bencode::Bencode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = tokio::fs::read("archlinux-2026.06.01-x86_64.iso.torrent").await?;
    let mut bencoder = Bencode::new();
    bencoder.parse(content).await?;
    let mut client: Session<TcpConnection> = Session::new(TorrentFile::from_bencode(&bencoder)?);
    client.announce().await?;
    render().await?;

    Ok(())
}


