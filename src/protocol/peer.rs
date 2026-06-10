use super::connection::Connection;
use super::file::TorrentFile;
use super::peerinfo::PeerInfo;

pub struct Peer<C: Connection> {
    info: PeerInfo,
    peer_info: PeerInfo,
    conn: C,
    choking: bool,
    interested: bool,
    peer_choking: bool,
    peer_interested: bool,
}

impl<C: Connection> Peer<C> {
    pub fn new(conn: C, info: PeerInfo, peer_info: PeerInfo) -> Self {
        Self {
            conn,
            choking: true,
            interested: false,
            peer_choking: true,
            peer_interested: false,
            info: info,
            peer_info: peer_info,
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

    pub async fn send_handshake(&mut self, file: TorrentFile) -> Result<(), Box<dyn std::error::Error>> {
        let mut handshake: Vec<u8> = Vec::new();
        handshake.push(19);
        
        for ch in b"BitTorrent protocol" {
            handshake.push(*ch);
        }

        // reserved
        handshake.push(0);
        handshake.push(0);
        handshake.push(0);
        handshake.push(0);
        handshake.push(0);
        handshake.push(0);
        handshake.push(0);
        handshake.push(0);

        // info_hash
        file.get_info_hash().iter().for_each(|&b| handshake.push(b));
        
        // peerinfo
        if let Some(id) = self.info.get_id() {
            id.iter().for_each(|&b| handshake.push(b));
        }

        self.conn.send(handshake.as_slice()).await?;

        Ok(())
    }
}