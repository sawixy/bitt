use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use url::Url;

use super::bencode::{Bencode, Entry};
use crate::protocol::peerinfo::PeerInfo;

#[derive(Debug, Clone, Copy)]
pub enum TrackerEvent {
    Started,
    Stopped,
    Completed,
}

#[derive(Debug)]
pub struct TrackerRequest {
    pub info_hash: Vec<u8>,
    pub peer_id: Vec<u8>,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub compact: bool,
    pub event: Option<TrackerEvent>,
    pub numwant: u32,
    pub key: Option<u32>,
    pub tracker_id: Option<String>,
}

#[derive(Debug)]
pub struct TrackerResponse {
    pub complete: u32,
    pub incomplete: u32,
    pub interval: u32,
    pub min_interval: u32,
    pub tracker_id: Option<String>,
    pub peers: Vec<PeerInfo>,
}

pub struct Tracker {
    client: Client,
    url: Url,
}

impl Tracker {
    pub fn new(url: String) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            url: Url::parse(&url)?,
        })
    }

    pub async fn send_request(&self, req: &TrackerRequest) -> Result<TrackerResponse, Box<dyn std::error::Error>> {
        if req.info_hash.len() != 20 {
            return Err(anyhow!("info_hash must be 20 bytes").into());
        }

        if req.peer_id.len() != 20 {
            return Err(anyhow!(format!("peer_id must be 20 bytes, but it {}", req.peer_id.len())).into());
        }

        let query_string = format!(
            "info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}&numwant={}{}{}{}",
            percent_encode(&req.info_hash),
            percent_encode(&req.peer_id),
            req.port,
            req.uploaded,
            req.downloaded,
            req.left,
            if req.compact { "1" } else { "0" },
            req.numwant,
            if let Some(ref id) = req.tracker_id {
                format!("&trackerid={}", percent_encode(id.as_bytes()))
            } else {
                String::new()
            },
            if let Some(key) = req.key {
                format!("&key={}", key)
            } else {
                String::new()
            },
            if let Some(event) = req.event {
                format!(
                    "&event={}",
                    match event {
                        TrackerEvent::Started => "started",
                        TrackerEvent::Stopped => "stopped",
                        TrackerEvent::Completed => "completed",
                    }
                )
            } else {
                String::new()
            },
        );

        let url_string = format!("{}?{}", self.url.as_str().trim_end_matches('/'), query_string);

        let body = self.client
            .get(&url_string)
            .send()
            .await?
            .error_for_status()?  
            .bytes()
            .await?;
        let mut decoder = Bencode::new();
        decoder.parse(body.to_vec()).await?;

        let dict = decoder.value
            .as_dict()
            .ok_or_else(|| anyhow!("tracker response is not a dictionary"))?;

        if let Some(reason) = dict.get("failure reason") {
            if let Some(bytes) = reason.as_string() {
                return Err(anyhow!(
                    "tracker failure: {}",
                    String::from_utf8_lossy(&bytes)
                ).into());
            }
        }

        if let Some(warning) = dict.get("warning message") {
            if let Some(bytes) = warning.as_string() {
                eprintln!(
                    "tracker warning: {}",
                    String::from_utf8_lossy(&bytes)
                );
            }
        }

        Ok(TrackerResponse {
            complete: get_u32(&dict, "complete", 0),
            incomplete: get_u32(&dict, "incomplete", 0),
            interval: get_u32(&dict, "interval", 1800),
            min_interval: get_u32(
                &dict,
                "min interval",
                get_u32(&dict, "interval", 1800),
            ),
            tracker_id: dict
                .get("tracker id")
                .and_then(|v| v.as_string())
                .map(|v| String::from_utf8_lossy(&v).into()),

            peers: parse_peers(&dict)?,
        })
    }
}

fn get_u32(dict: &std::collections::HashMap<String, Entry>, key: &str, default: u32) -> u32 {
    dict.get(key)
        .and_then(|v| v.as_int())
        .unwrap_or(default as i64) as u32
}

fn parse_peers(dict: &std::collections::HashMap<String, Entry>) -> Result<Vec<PeerInfo>, Box<dyn std::error::Error>> {
    let mut peers = Vec::new();

    // compact mode
    if let Some(value) = dict.get("peers") {
        if let Some(bytes) = value.as_string() {
            if bytes.len() % 6 == 0 {
                for chunk in bytes.chunks_exact(6) {
                    let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);
                    let port = ((chunk[4] as u16) << 8) | chunk[5] as u16;
                    peers.push(PeerInfo::new(None, ip, port));
                }
                return Ok(peers);
            }
        }
    }

    // compact mode 6
    if let Some(value6) = dict.get("peers6") {
        if let Some(bytes) = value6.as_string() {
            for chunk in bytes.chunks_exact(18) {
                let ip = format!(
                    "{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                    chunk[0], chunk[1], chunk[2], chunk[3],
                    chunk[4], chunk[5], chunk[6], chunk[7],
                    chunk[8], chunk[9], chunk[10], chunk[11],
                    chunk[12], chunk[13], chunk[14], chunk[15],
                );
                let port = ((chunk[16] as u16) << 8) | chunk[17] as u16;
                peers.push(PeerInfo::new(None, ip, port));
            }
            return Ok(peers);
        }
    }

    // non-compact mode
    if let Some(value) = dict.get("peers") {
        if let Some(list) = value.as_list() {
            for peer in list {
                peers.push(PeerInfo::from_bencode(&peer)?);
            }
        }
    }

    Ok(peers)
}

fn percent_encode(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("%{:02x}", b))
        .collect()
}