use crate::protocol::bitfield::BitField;
use crate::protocol::connection::{Connection, TcpConnection};
use crate::protocol::peer::{PeerMessage, PeerMessageType};
use crate::protocol::tracker::TrackerRequest;
use super::peerinfo::PeerInfo;
use super::peer::Peer;
use std::sync::Arc;
use anyhow::anyhow;
use tokio::sync::{RwLock, Mutex};
use super::file::TorrentFile;
use super::storage::Storage;
use super::tracker::Tracker;

const TRIES: usize = 10;
const PORT: u16 = 2222;

struct PeerState {
    peer: Peer<TcpConnection>,
    bits: BitField,
    busy: bool,
}

impl PeerState {
    fn new(peer: Peer<TcpConnection>, bits: BitField, busy: bool) -> Self {
        Self {
            peer: peer,
            bits: bits,
            busy: busy,
        }
    }
}

pub struct Session<S: Storage> {
    file: Arc<TorrentFile>,
    peers: Arc<RwLock<Vec<Arc<Mutex<PeerState>>>>>,
    info: PeerInfo,
    storage: Arc<Mutex<S>>, 
}

impl<S> Session<S> where S: Storage + Send + Sync + 'static {
    pub fn new(file: TorrentFile, storage: S) -> Self {
        let peer_id = b"-ZB0001-0123456789ab".to_vec();
        let ip = String::new();
        Self { 
            file: Arc::new(file), 
            peers: Arc::new(RwLock::new(Vec::new())), 
            info: PeerInfo::new(Some(peer_id), ip, PORT), 
            storage: Arc::new(Mutex::new(storage)) 
        }
    }

    pub fn get_file(&self) -> &TorrentFile {
        &self.file
    }

    async fn connect(&self, peer: &mut Peer<TcpConnection>) -> Result<BitField, Box<dyn std::error::Error>> {
        println!("Sending handshake");
        peer.send_handshake().await?;
        println!("Receiving handshake");
        peer.recv_handshake().await?;

        // sending bitfield
        println!("Sending bitfield");
        let mut bitfield = PeerMessage::new();
        bitfield.msg_type = PeerMessageType::Bitfield;
        bitfield.payload = self.storage.lock().await.get_bitfield().to_vec();
        peer.send_message(bitfield).await?;

        // receiving bitfield
        let msg = peer.recv_message().await?;
        println!("Type: {:?}", msg.msg_type);
        let mut bf = BitField::from_vec(msg.payload);

        // sending unchoke and interested
        peer.set_choking(false);
        let mut unchoke = PeerMessage::new();
        unchoke.msg_type = PeerMessageType::Unchoke;

        peer.set_interested(true);
        let mut interested = PeerMessage::new();
        interested.msg_type = PeerMessageType::Interested;

        peer.send_message(unchoke).await?;

        let mut choking: Option<bool> = None;
        let mut interesting: Option<bool> = None;

        for _ in 0..TRIES {
            if let Some(_) = choking {
                break;
            }
            let msg = peer.recv_message().await?;
            println!("Type: {:?}", msg.msg_type);
            match msg.msg_type {
                PeerMessageType::Bitfield => bf = BitField::from_vec(msg.payload),
                PeerMessageType::Have => bf.set_bit(u32::from_be_bytes(msg.payload[..4].try_into()?) as usize, 1),
                PeerMessageType::Interested => interesting = Some(true),
                PeerMessageType::NotInterested => interesting = Some(false),
                PeerMessageType::Choke => choking = Some(true),
                PeerMessageType::Unchoke => {
                    choking = Some(false);

                    peer.send_message(interested.clone()).await?;
                },
                _ => continue,
            };
        }

        if let Some(boo) = choking {
            peer.set_peer_choking(boo);
        } else {
            return Err(anyhow!("Peer didnt send chocking").into())
        }
        Ok(bf)
    }

    pub async fn discover(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.file.supports_v1() {
            return Err(anyhow!("File type not supported").into());
        }

        let tracker_url = self.file.get_trackers()[0].clone();
        let tracker = Tracker::new(tracker_url)?;
        let request = TrackerRequest {
            info_hash: self.file.get_info_hash(),
            peer_id: match self.info.get_id() {
                    Some(id) => id.to_vec(),
                    None => Vec::with_capacity(20),
                },
            port: PORT,
            uploaded: 0,
            downloaded: 0,
            left: 10,
            compact: true,
            numwant: 500,
            event: None,
            key: None,
            tracker_id: None,
        };

        let response = tracker.send_request(&request).await?;

        for p in response.peers {
            let mut connection = TcpConnection::new(p.get_ip().clone(), p.get_port());

            match connection.open().await {
                Ok(()) => {
                    let peer = Peer::new(
                        connection,
                        self.info.clone(),
                        p.clone(),
                        self.file.clone(),
                    );
                    if let Err(e) = self.add_peer(peer).await {
                        eprintln!("Handshake failed with {:?}: {}", p, e);
                    }
                }
                Err(e) => {
                    eprintln!("Connection failed to {:?}: {}", p, e);
                }
            }
        }

        Ok(())
    }

    pub async fn add_peer(&self, mut peer: Peer<TcpConnection>) -> Result<(), Box<dyn std::error::Error>> {
        println!("Connected {:?}", peer.get_peerinfo());
        let bitfield = self.connect(&mut peer).await?;
        self.peers.write().await.push(Arc::new(Mutex::new(PeerState::new(peer, bitfield, false))));

        Ok(())
    }

    pub async fn remove_peer(&self, peerinfo: PeerInfo) {
        let mut peers = self.peers.write().await;
        let mut i = 0;
        while i < peers.len() {
            let lock = peers[i].lock().await;
            let same = lock.peer.get_peerinfo().get_id() == peerinfo.get_id()
                && lock.peer.get_peerinfo().get_ip() == peerinfo.get_ip()
                && lock.peer.get_peerinfo().get_port() == peerinfo.get_port();
            drop(lock);
            if same {
                peers.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        for piece in 0..self.file.piece_count() {
            println!("Requesting {}th piece", piece);
            for state in self.peers.write().await.iter_mut() {
                let mut lock = state.lock().await;
                if lock.bits.get_bit(piece) == 0 || lock.busy {
                    continue;
                }

                let mut request = PeerMessage::new();
                request.msg_type = PeerMessageType::Request;
                request.payload = piece.to_be_bytes().to_vec();
                lock.peer.send_message(request).await?;
                
                // Mark as busy before dropping the lock and spawning the task
                lock.busy = true; 
                drop(lock);

                let state_clone = Arc::clone(state);
                let storage_clone = Arc::clone(&self.storage);
            
                tokio::spawn(async move {
                    let mut lock = state_clone.lock().await;
                    
                    for _ in 0..TRIES {
                        // Use match instead of ? since this block returns ()
                        let msg = match lock.peer.recv_message().await {
                            Ok(m) => m,
                            Err(_) => return, 
                        };

                        match msg.msg_type {
                            PeerMessageType::Have => {
                                if let Ok(bytes) = msg.payload[..4].try_into() {
                                    lock.bits.set_bit(u32::from_be_bytes(bytes) as usize, 1);
                                }
                            }
                            PeerMessageType::Interested => lock.peer.set_peer_interested(true),
                            PeerMessageType::NotInterested => lock.peer.set_peer_interested(false),
                            PeerMessageType::Choke => lock.peer.set_peer_choking(true),
                            PeerMessageType::Unchoke => lock.peer.set_peer_choking(false),
                            PeerMessageType::Piece => {
                                println!("got {}th piece", piece);
                                let index = match msg.payload[0..4].try_into() {
                                    Ok(b) => u32::from_be_bytes(b) as usize,
                                    Err(_) => return,
                                };
                                let _begin = match msg.payload[4..8].try_into() {
                                    Ok(b) => u32::from_be_bytes(b) as usize,
                                    Err(_) => return,
                                };
                                let block = msg.payload[8..].to_vec();
                                
                                storage_clone.lock().await.set_piece(index, block).await;
                                lock.busy = false;
                                break;
                            }
                            _ => continue,
                        }
                    }
                }).await?; 
            }
        }

        Ok(())
    }
}