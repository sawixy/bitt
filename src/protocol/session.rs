use crate::protocol::bitfield::BitField;
use crate::protocol::connection::{Connection, TcpConnection};
use crate::protocol::peer::{self, PeerMessage, PeerMessageType};
use crate::protocol::tracker::{TrackerEvent, TrackerRequest, TrackerResponse};
use super::tracker::Tracker;
use super::peerinfo::PeerInfo;
use super::peer::Peer;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::file::TorrentFile;
use super::storage::Storage;

const TRIES: usize = 10;

pub struct Session<S: Storage> {
    file: Arc<TorrentFile>,
    peers: Arc<RwLock<Vec<Peer<TcpConnection>>>>,
    info: PeerInfo,
    storage: S,
}

impl<S> Session<S> where S: Storage {
    pub fn new(file: TorrentFile, storage: S) -> Self {
        let peer_id = Vec::new();
        let ip = String::new();
        Self { file: Arc::new(file), peers: Arc::new(RwLock::new(Vec::new())), info: PeerInfo::new(Some(peer_id), ip, 6881), storage: storage }
    }

    pub fn get_file(&self) -> &TorrentFile {
        &self.file
    }

    pub async fn add_peer(&self, peer: Peer<TcpConnection>) {
        self.peers.write().await.push(peer);
    }

    pub async fn remove_peer(&self, peerinfo: PeerInfo) {
        self.peers.write().await.retain(|p| p.get_peerinfo().get_id() == peerinfo.get_id() &&
                                                                  p.get_peerinfo().get_ip() == peerinfo.get_ip() && 
                                                                  p.get_peerinfo().get_port() == peerinfo.get_port())
    }

    pub async fn download(&self, storage: S) -> Result<(), Box<dyn std::error::Error>> {
        

        Ok(())
    } 
}