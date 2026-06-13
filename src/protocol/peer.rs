use anyhow::anyhow;

use super::connection::Connection;
use super::file::TorrentFile;
use super::peerinfo::PeerInfo;

use std::any;
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
    Unknown,
}

pub struct PeerMessage {
    pub msg_type: PeerMessageType,
    pub payload: Vec<u8>
}

impl PeerMessage {
    pub fn new() -> Self {
        Self {
            msg_type: PeerMessageType::Unchoke,
            payload: Vec::new(),
        }
    }
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
    pub async fn send_message(&mut self, msg: PeerMessage) -> Result<(), Box<dyn std::error::Error>> {
        let mut raw: Vec<u8> = Vec::new();
        raw.push(match msg.msg_type {
            PeerMessageType::Choke => 0,
            PeerMessageType::Unchoke => 1,
            PeerMessageType::Interested => 2,
            PeerMessageType::NotInterested => 3,
            PeerMessageType::Have => 4,
            PeerMessageType::Bitfield => 5,
            PeerMessageType::Request => 6,
            PeerMessageType::Piece => 7,
            PeerMessageType::Cancel => 8,
            PeerMessageType::Unknown => return Err(anyhow!("Message type is unknown").into()),
        });

        raw.push(msg.payload.len() as u8);
        raw.extend(msg.payload.iter());

        self.conn.send(raw.as_slice()).await?;

        Ok(())
    }

    pub async fn recv_message(&mut self) -> Result<PeerMessage, Box<dyn std::error::Error>> {
        let mut msg = PeerMessage::new();

        let raw = self.conn.receive().await?;
        msg.msg_type = match raw[0] {
            0 => PeerMessageType::Choke,
            1 => PeerMessageType::Unchoke,
            2 => PeerMessageType::Interested,
            3 => PeerMessageType::NotInterested,
            4 => PeerMessageType::Have,
            5 => PeerMessageType::Bitfield,
            6 => PeerMessageType::Request,
            7 => PeerMessageType::Piece,
            8 => PeerMessageType::Cancel,
            _ => PeerMessageType::Unknown,
        };

        let mut data: Vec<u8> = Vec::new();
        data.extend_from_slice(raw[2..(raw[1] as usize)].iter().as_slice());

        Ok(msg)
    }
}