use egui::accesskit::Checked::True;
use http_wire::{WireEncode, WireDecode};
use http::{Request, Response};
use http_body_util::Full;
use bytes::Bytes;
use sha1::digest::typenum::U;
use crate::protocol::connection::{Connection};
use crate::protocol::peerinfo::PeerInfo;
use tokio::net::{lookup_host};
use url::Url;

pub struct Tracker {
    url: String,
}

pub struct TrackerRequest {
    pub info_hash: Vec<u8>, // 20 bytes
    pub peer_id: Vec<u8>,   // 20 bytes
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub compact: bool,
    pub event: Option<TrackerEvent>,
    pub ip: String,
    pub numwant: u64,
    pub key: u64,
    pub trackerid: u64,
}

impl TrackerRequest {
    pub fn new() -> Self {
        TrackerRequest { info_hash: Vec::new(), peer_id: Vec::new(), port: 0, uploaded: 0, downloaded: 0, left: 0, compact: false, event: None, ip: String::new(), numwant: 0, key: 0, trackerid: 0 }
    }
}

pub enum TrackerEvent {
    Started,
    Stopped,
    Completed
}

impl Tracker {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub fn get_url(&self) -> &String {
        &self.url
    }

    pub async fn send_request<C: Connection>(&self, connection: &mut C, req: TrackerRequest) -> Result<(), Box<dyn std::error::Error>> {
        let parsed = Url::parse(&self.url)?;
        let port = parsed.port();
        let host_with_port = format!("{}:{}", parsed.domain().ok_or("Expected domain in URL")?, port.unwrap_or(443));
        let sock_addr = lookup_host(host_with_port).await?.next().ok_or("Failed to resolve domain")?;
        
        connection.set_ip(sock_addr.ip().to_string());
        connection.set_port(match port {
            Some(p) => p,
            None => 433,
        });
        connection.open().await?;

        let mut info_hash = String::new();
        req.info_hash.iter().for_each(|b| info_hash.push(*b as char));

        let mut peer_id = String::new();
        req.peer_id.iter().for_each(|b| peer_id.push(*b as char));

        let request = format!(
            "info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}&ip={}&numwant={}&key={}&trackerid={}{}",
            info_hash,
            peer_id,
            req.port,
            req.uploaded,
            req.downloaded,
            req.left,
            match req.compact { true => 1, false => 0 },
            req.ip,
            req.numwant,
            req.key,
            req.trackerid,
            match req.event {
                Some(event) => match event {
                    TrackerEvent::Started => "&event=started",
                    TrackerEvent::Stopped => "&event=stopped",
                    TrackerEvent::Completed => "&event=completed",
                },
                None => ""
            }
        );


        let req = Request::builder()
            .method("GET")
            .uri(format!("/announce?{}", request))
            .body(Full::new(Bytes::from("")))?;

        connection.send(&req.encode()?).await?;

        let mut headers = [http_wire::Header; 16];
        let resp = connection.receive().await?.as_slice();

        connection.close().await?;

        Ok(())
    }
}