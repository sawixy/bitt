mod protocol;

use protocol::file::TorrentFile;
use protocol::connection::{Connection, TcpConnection};
use protocol::bencode::{Bencode, Entry};
use http_wire::WireEncode;
use http::Request;
use http_body_util::Full;
use bytes::Bytes;

mod client;
use crate::client::app::render;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>

    render().await?;


    let mut f = Bencode::new();
    f.parse(tokio::fs::read("archlinux-2026.06.01-x86_64.iso.torrent").await?).await?;
    let file = TorrentFile::from_bencode(&f)?;
    println!("{:?}", file);

    let mut conn = TcpConnection::new(file.get_trackers()[0].clone(), 6881);
    conn.open().await?;
    let req = Request::builder()
        .method("GET")
        .uri(format!("/announce?peer_id={}&info_hash={}&port={}&left={}&downloaded={}&uploaded={}&compact=0", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbbbbbb", 6881, 0, 0, 0))
        .body(Full::new(Bytes::from("")))?;

    conn.send(&req.encode()?).await?;
    println!("{}", std::str::from_utf8(conn.receive().await?.as_slice())?);

    Ok(())
}


