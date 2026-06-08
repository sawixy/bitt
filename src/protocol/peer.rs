use crate::protocol::connection::{Connection, TcpConnection};
use crate::protocol::tracker::{TrackerEvent, TrackerRequest};
use super::tracker::Tracker;
use super::peerinfo::PeerInfo;

use super::file::TorrentFile;

pub struct Peer<C: Connection> {
    file: TorrentFile,
    connections: Vec<C>,
}

impl<C: Connection> Peer<C> {
    pub fn new(file: TorrentFile) -> Self {
        Self { file, connections: Vec::new() }
    }

    pub fn get_file(&self) -> &TorrentFile {
        &self.file
    }

    pub async fn announce(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.file.get_trackers().is_empty() {
            // TODO: Make V2, Magnet support
            return Err("No trackers found in torrent file".into());
        }

        let mut conn: TcpConnection = TcpConnection::new(String::from("127.0.0.1"), 0000);
        let tracker = Tracker::new(self.file.get_trackers()[0].clone());

        let mut tracker_req = TrackerRequest::new();
        tracker_req.info_hash = vec![65; 20];
        tracker_req.peer_id = vec![67; 20];
        tracker_req.port = 6781;
        tracker_req.downloaded = 0;
        tracker_req.uploaded = 0;
        tracker_req.left = 155590;
        tracker_req.numwant = 100;
        tracker_req.event = Some(TrackerEvent::Started);
        tracker_req.compact = false;

        tracker.send_request(&mut conn, tracker_req).await?;

        Ok(())
    }
}