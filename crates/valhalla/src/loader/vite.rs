//! Vite dev server bridge.
//!
//! At a high level:
//!
//! - `connect()` opens a Vite WebSocket and confirms `connected`.
//! - `fetch_module()` is called by the rquickjs module loader to GET a module
//!   over HTTP. Vite serves transformed ES modules with their imports
//!   pre-rewritten to absolute paths, so we don't have to do resolution
//!   ourselves — just follow what the server says.
//! - The HMR thread reads WS frames and forwards each to `__host_hmr` on the
//!   JS side, which knows what to do with `update`/`full-reload`/etc.
//!
//! What lives in JS, not here: `import.meta.hot` semantics, the react-refresh
//! integration, the module accept/dispose graph. Vite's plugin already injects
//! all that — we just need to be a faithful HTTP+WS host for it.

use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::Context as _;

/// Live Vite session. Holds the HMR thread handle and the channel for
/// inbound HMR messages.
pub struct ViteSession {
    pub base_url: String,
    pub hmr_rx: mpsc::Receiver<HmrMessage>,
    _hmr_thread: JoinHandle<()>,
}

#[derive(Debug, Clone)]
pub enum HmrMessage {
    /// Raw JSON payload from Vite. We forward it verbatim to JS.
    Frame(String),
    /// The connection dropped. UI should show "dev server offline".
    Disconnected,
}

impl ViteSession {
    pub fn connect(base_url: String) -> anyhow::Result<Self> {
        // Sanity check the dev server is up before we try the WS dance.
        let probe = format!("{}/", base_url.trim_end_matches('/'));
        ureq::get(&probe)
            .timeout(Duration::from_secs(2))
            .call()
            .with_context(|| format!("Vite dev server unreachable at {}", probe))?;

        let (hmr_tx, hmr_rx) = mpsc::channel();
        let ws_url = ws_url_from_http(&base_url);

        let _hmr_thread = thread::spawn(move || hmr_loop(ws_url, hmr_tx));

        Ok(Self {
            base_url,
            hmr_rx,
            _hmr_thread,
        })
    }

    /// HTTP GET a module. The path is whatever the JS-side import resolution
    /// produced — typically absolute (`/src/App.tsx`) or with Vite query
    /// params (`?v=...`).
    pub fn fetch_module(&self, path: &str) -> anyhow::Result<String> {
        let url = if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else if path.starts_with('/') {
            format!("{}{}", self.base_url.trim_end_matches('/'), path)
        } else {
            format!("{}/{}", self.base_url.trim_end_matches('/'), path)
        };

        let response = ureq::get(&url)
            .set("Accept", "*/*")
            .timeout(Duration::from_secs(10))
            .call()
            .with_context(|| format!("fetching module {}", url))?;

        response
            .into_string()
            .with_context(|| format!("reading body for {}", url))
    }
}

fn ws_url_from_http(http: &str) -> String {
    let stripped = http.trim_end_matches('/');
    if let Some(rest) = stripped.strip_prefix("http://") {
        format!("ws://{}", rest)
    } else if let Some(rest) = stripped.strip_prefix("https://") {
        format!("wss://{}", rest)
    } else {
        format!("ws://{}", stripped)
    }
}

fn hmr_loop(ws_url: String, tx: mpsc::Sender<HmrMessage>) {
    use tungstenite::client::IntoClientRequest;
    use tungstenite::{connect, Message};

    let mut backoff = Duration::from_millis(250);
    let max_backoff = Duration::from_secs(5);

    loop {
        let request = match (&ws_url).into_client_request() {
            Ok(mut req) => {
                // Vite expects this subprotocol for the HMR channel.
                req.headers_mut()
                    .insert("Sec-WebSocket-Protocol", "vite-hmr".parse().unwrap());
                req
            }
            Err(err) => {
                log::error!("[valhalla] invalid Vite WS url {}: {}", ws_url, err);
                let _ = tx.send(HmrMessage::Disconnected);
                return;
            }
        };

        match connect(request) {
            Ok((mut socket, _resp)) => {
                backoff = Duration::from_millis(250);
                loop {
                    match socket.read() {
                        Ok(Message::Text(payload)) => {
                            if tx.send(HmrMessage::Frame(payload)).is_err() {
                                return;
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Ok(_) => {}
                        Err(err) => {
                            log::warn!("[valhalla] HMR socket error: {}", err);
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                log::warn!("[valhalla] HMR connect failed: {}; retry in {:?}", err, backoff);
            }
        }

        if tx.send(HmrMessage::Disconnected).is_err() {
            return;
        }
        thread::sleep(backoff);
        backoff = (backoff * 2).min(max_backoff);
    }
}
