pub trait Response {
    fn handle<C: super::connection::Connection>(&self, conn: C) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct TrackerResponse {
    interval: u32,
    peers: Vec<super::peer::Peer>,
}