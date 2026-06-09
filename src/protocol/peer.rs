use super::connection::Connection;

#[derive(Debug)]
pub enum PeerState {
    Choked,
    Unchoked,
    Interested,
    NotInterested,
}

pub struct Peer<C: Connection> {
    conn: C,
    choking: bool,
    interested: bool,
    peer_choking: bool,
    peer_interested: bool,
}

impl<C: Connection> Peer<C> {
    pub fn new(conn: C) -> Self {
        Self {
            conn,
            choking: true,
            interested: false,
            peer_choking: true,
            peer_interested: false,
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
}