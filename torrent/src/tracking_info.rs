use reqwest::blocking::Client;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub struct TrackerRequests {
    pub info_hash: String,
    pub peer_id: String,
    pub port: usize,
    pub uploaded: usize,
    pub downloaded: usize,
    pub left: usize,
    pub compact: bool,
    pub no_peer_id: String, // Temporary
    pub event: String, // started, stopped, completed
    pub ip: Option<String>, // Temporary
    pub numwant: Option<usize>,
    pub key: Option<String>,
    pub trackerid: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub struct TrackerResponses {
    pub failure_reason: String,
    pub interval: usize,
    pub trackerid: String,
    pub complete: usize,
    pub incomplete: usize,
    pub warning_message: String,
    pub min_interval: usize,
}

#[allow(dead_code)]
impl TrackerRequests {
    const PIECE_HASH_SIZE: usize = 20;

    pub fn new(
        info_hash: String,
        peer_id: String,
        port: usize,
        uploaded: usize,
        downloaded: usize,
        left: usize,
        compact: bool,
        no_peer_id: String,
        event: String,
        ip: Option<&str>,
        numwant: Option<usize>,
        key: Option<&str>,
        trackerid: Option<&str>
    ) -> Option<Self> {
        Some(TrackerRequests {
            info_hash,
            peer_id,
            port,
            uploaded,
            downloaded,
            left,
            compact,
            no_peer_id,
            event,
            ip: match ip {
                Some(x) => Some(x.to_string()),
                None => None,
                
            },
            numwant: match numwant {
                Some(x) => Some(x),
                None => None,
            },
            key: match key {
                Some(x) => Some(x.to_string()),
                None => None,
            },
            trackerid: match trackerid {
                Some(x) => Some(x.to_string()),
                None => None,
            },
        })
    }

    pub fn build_request_url(&self, base_url: &str) -> Result<String, &str> {
        let mut url = format!("{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}&no_peer_id={}&event={}",
            base_url,
            self.info_hash,
            self.peer_id,
            self.port,
            self.uploaded,
            self.downloaded,
            self.left,
            self.compact,
            self.no_peer_id,
            self.event
        );

        if let Some(ip) = &self.ip {
            url.push_str(&format!("&ip={}", ip));
        }

        if let Some(numwant) = &self.numwant {
            url.push_str(&format!("&numwant={}", numwant));
        }

        if let Some(key) = &self.key {
            url.push_str(&format!("&key={}", key));
        }

        if let Some(trackerid) = &self.trackerid {
            url.push_str(&format!("&trackerid={}", trackerid));
        }

        Ok(url)
    }

    pub fn send_request_to_tracker(&self, base_url: &str) -> Result<String, &'static str> {
        let client = Client::new();

        let request_url = self.build_request_url(base_url)
            .map_err(|_| "Failed to build request URL")?;

        println!("{}", request_url);

        let response = client
            .get(&request_url)
            .send()
            .map_err(|_| "Failed to send request to tracker")?;

        println!("{}", response.status());

        let response_text = response.text().map_err(|_| "Failed to read response body")?;
        Ok(response_text)

        // if response.status().is_success() {
        //     let response_text = response.text().map_err(|_| "Failed to read response body")?;
        //     Ok(response_text)
        // } else {
        //     Err("Tracker request failed")
        // }
    }
}

