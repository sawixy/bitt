use anyhow::anyhow;
use eframe::egui::Key::A;

use super::connection::Connection;
use super::file::TorrentFile;
use super::peerinfo::PeerInfo;

use std::sync::Arc;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
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
    KeepAlive,
}

#[derive(Clone)]
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


#[derive(Clone, Debug)]
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
            conn: conn,
            choking: true,
            interested: false,
            peer_choking: true,
            peer_interested: false,
            info: info,
            peer_info: peer_info,
            file: file,
        }
    }

    pub fn get_info(&self) -> PeerInfo {
        self.info.clone()
    }

    pub fn get_peerinfo(&self) -> PeerInfo {
        self.peer_info.clone()
    }
    
    pub fn get_choking(&self) -> bool {
        self.choking
    }
    
    pub fn get_interested(&self) -> bool {
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
        
        handshake.extend_from_slice(b"BitTorrent protocol");

        // reserved
        handshake.extend_from_slice(&[0u8; 8]);

        // info_hash
        self.file.get_info_hash().iter().for_each(|&b| handshake.push(b));
        
        // peerinfo
        if let Some(id) = self.info.get_id() {
            id.iter().for_each(|&b| handshake.push(b));
        } else {
            handshake.extend_from_slice(b"-qB4670-M5MQ6hKK8_5V");
        }

        self.conn.send(handshake.as_slice()).await?;

        Ok(())
    }

    pub async fn recv_handshake(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let raw = self.conn.read_exact(68).await?;
        println!("{:?}", raw);
        if raw[0] != 19 || &raw[1..20] != b"BitTorrent protocol" {
            return Err(anyhow!("Invalid protocol").into());
        }

        let info_hash = raw[28..48].to_vec();
        let my_info_hash = self.file.get_info_hash();
        for i in 0..20usize {
            if info_hash[i] != my_info_hash[i] {
                return Err(anyhow!("Info hash doesnt match").into())
            }
        }

        // peer id skipped (i dont care about it, or am i?)

        Ok(())
    }
    pub async fn send_message(&mut self, msg: PeerMessage) -> Result<(), Box<dyn std::error::Error>> {
        let mut raw: Vec<u8> = Vec::new();
        
        // Message ID (1 byte)
        let message_id: u8 = match msg.msg_type {
            PeerMessageType::Choke => 0,
            PeerMessageType::Unchoke => 1,
            PeerMessageType::Interested => 2,
            PeerMessageType::NotInterested => 3,
            PeerMessageType::Have => 4,
            PeerMessageType::Bitfield => 5,
            PeerMessageType::Request => 6,
            PeerMessageType::Piece => 7,
            PeerMessageType::Cancel => 8,
            PeerMessageType::KeepAlive => return Err(anyhow!("Message type is KeepAlive").into()),
        };
        
        // Length prefix: 4 bytes big-endian = 1 (message id) + payload length
        let length = 1u32 + msg.payload.len() as u32;
        raw.extend_from_slice(&length.to_be_bytes());
        
        // Message ID
        raw.push(message_id);
        
        // Payload
        raw.extend_from_slice(&msg.payload);

        self.conn.send(raw.as_slice()).await?;
        Ok(())
    }

    pub async fn recv_message(&mut self) -> Result<PeerMessage, Box<dyn std::error::Error>> {
        let length_bytes = self.conn.read_exact(4).await?;
        let length = u32::from_be_bytes([length_bytes[0], length_bytes[1], length_bytes[2], length_bytes[3]]) as usize;
        println!("Length: {}", length);
        
        if length == 0 {
            return Ok(PeerMessage {
                msg_type: PeerMessageType::KeepAlive,
                payload: Vec::new(),
            });
        }
        
        let body = self.conn.read_exact(length).await?;

        println!("Body: {:?}", body);

        let msg_type = match body[0] {
            0 => PeerMessageType::Choke,
            1 => PeerMessageType::Unchoke,
            2 => PeerMessageType::Interested,
            3 => PeerMessageType::NotInterested,
            4 => PeerMessageType::Have,
            5 => PeerMessageType::Bitfield,
            6 => PeerMessageType::Request,
            7 => PeerMessageType::Piece,
            8 => PeerMessageType::Cancel,
            _ => return Err(anyhow!("Unknown message type: {}", body[0]).into()),
        };
        
        let payload = body[1..].to_vec();
        
        Ok(PeerMessage { msg_type, payload })
    }
}