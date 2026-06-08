use http_wire::response::FullResponse;
use http_wire::{WireEncode, WireDecode};
use http::{Request, Response};
use http_body_util::Full;
use bytes::Bytes;
use http_wire::Header;
use sha1::digest::typenum::U;
use crate::protocol::connection::{Connection};
use crate::protocol::peerinfo::PeerInfo;
use tokio::net::{lookup_host};
use url::Url;
use httparse::EMPTY_HEADER;
use super::bencode::Bencode;
use anyhow::anyhow;

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

pub struct TrackerResponse {
    pub complete: u32,
    pub incomplete: u32,
    pub peers: Vec<PeerInfo>, // peer, completed
    pub interval: u32,
    pub min_interval: u32,
    pub trackerid: u64,
}

impl TrackerResponse {
    pub fn new() -> Self {
        TrackerResponse { complete: 0, incomplete: 0, peers: Vec::new(), interval: 0, min_interval: 0, trackerid: 0 }
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

        let mut headers: [Header; 16] = [EMPTY_HEADER; 16];
        let resp = connection.receive().await?;
        println!("{}", std::str::from_utf8(resp.as_slice())?);
        let (response, total_len) = FullResponse::decode(resp.as_slice(), &mut headers)?;
        let mut bencoder = Bencode::new();
        bencoder.parse(Vec::from(response.body)).await?;
        let dict = bencoder.value.as_dict().ok_or("Excpected dictionary in response from tracker")?;

        let mut tracker_response = TrackerResponse::new();

        if dict.contains_key("failure reason")  {
            return Err(anyhow!("failure reason: {}", std::str::from_utf8(dict["warning reason"].as_string().unwrap_or(Vec::new()).as_slice())?).into());
        } else if dict.contains_key("warning reason") {
            eprintln!("warning reason: {}", std::str::from_utf8(dict["warning reason"].as_string().unwrap_or(Vec::new()).as_slice())?);
        }

        tracker_response.complete = dict["complete"].as_int().ok_or("Expected int for coplete")? as u32;
        tracker_response.incomplete = dict["incomplete"].as_int().ok_or("Expected int for incomplete")? as u32;
        tracker_response.interval = dict["trackerid"].as_int().ok_or("Expected int for trackerid")? as u32;
        tracker_response.interval = dict["interval"].as_int().ok_or("Expected int for interval")? as u32;
        tracker_response.min_interval = dict["min_interval"].as_int().unwrap_or(tracker_response.interval as i64) as u32;

        // peers parsing
        if let Some(peers_dict) = dict["peers"].as_dict() {
            for (_, peer) in peers_dict {
                tracker_response.peers.push(PeerInfo::from_bencode(&peer)?);
            }
        } else if let Some(peers) = dict["peers"].as_string() {
            // TODO: Add compact mode
        }

        connection.close().await?;

        Ok(())
    }
}