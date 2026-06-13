use anyhow::anyhow;

use super::connection::Connection;
use super::file::TorrentFile;
use super::peerinfo::PeerInfo;

use std::sync::Arc;

pub enum PeerMessageType {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
    Cancel,
}

pub struct PeerMessage {
    pub msg_type: PeerMessageType,
    pub length: u8,
    pub payload: Vec<u8>
}

pub struct Peer<C: Connection> {
    info: PeerInfo,
    peer_info: PeerInfo,
    conn: C,
    choking: bool,
    interested: bool,
    peer_choking: bool,
    peer_interested: bool,
    file: Arc<TorrentFile>,
}

impl<C: Connection> Peer<C> {
    pub fn new(conn: C, info: PeerInfo, peer_info: PeerInfo, file: Arc<TorrentFile>) -> Self {
        Self {
            conn,
            choking: true,
            interested: false,
            peer_choking: true,
            peer_interested: false,
            info: info,
            peer_info: peer_info,
            file: file,
        }
    }
    
    pub fn choking(&self) -> bool {
        self.choking
    }
    
    pub fn interested(&self) -> bool {
        self.interested
    }
    
    pub fn peer_choking(&self) -> bool {
        self.peer_choking
    }
    
    pub fn peer_interested(&self) -> bool {
        self.peer_interested
    }
    
    pub fn set_choking(&mut self, choking: bool) {
        self.choking = choking;
    }
    
    pub fn set_interested(&mut self, interested: bool) {
        self.interested = interested;
    }
    
    pub fn set_peer_choking(&mut self, choking: bool) {
        self.peer_choking = choking;
    }
    
    pub fn set_peer_interested(&mut self, interested: bool) {
        self.peer_interested = interested;
    }
    
    pub fn get_conn(&mut self) -> &mut C {
        &mut self.conn
    }

    pub async fn send_handshake(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut handshake: Vec<u8> = Vec::new();
        handshake.push(19);
        
        handshake.extend_from_slice(b"BitTorrent Protocol");

        // reserved
        handshake.extend_from_slice(&[0u8; 8]);

        // info_hash
        self.file.get_info_hash().iter().for_each(|&b| handshake.push(b));
        
        // peerinfo
        if let Some(id) = self.info.get_id() {
            id.iter().for_each(|&b| handshake.push(b));
        }

        self.conn.send(handshake.as_slice()).await?;

        Ok(())
    }

    pub async fn recv_handshake(&mut self, file: TorrentFile) -> Result<(), Box<dyn std::error::Error>> {
        let raw: Vec<u8> = self.conn.receive().await?;
        if raw[0] != 19 || &raw[1..20] != b"BitTorrent Protocol" {
            return Err(anyhow!("Invalid protocol").into());
        }

        let info_hash = &raw[21..41];
        if info_hash != self.file.get_info_hash().as_slice() {
            return Err(anyhow!("Info hash doesnt match").into())
        }

        // peer id skipped (i dont care about it)

        Ok(())
    }
}