use super::connection::Connection;

pub trait Request {
    fn to_bytes(&self) -> Vec<u8>;
}

pub enum PeerRequestType {
    Choke,          // no payload
    Unchoke,        // no payload
    Interested,     // no payload
    NotInterested,  // no payload
    Have, 
    Bitfield,
    Request,
    Piece,
    Cancel,
}

pub enum Event {
    Started,
    Completed,
    Stopped,
}

pub struct TrackerRequest {
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    event: Option<Event>,
}

pub struct PeerRequest {
    
}