use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use russh::server::{Msg, Session};
use russh::*;
use russh_keys::*;
use tokio::sync::Mutex;

use crate::errors::HovelError;

pub async fn run_server() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let keypair = russh_keys::key::KeyPair::generate_ed25519();

    let config = russh::server::Config {
        methods: MethodSet::PUBLICKEY,
        keys: vec![keypair.unwrap()],
        ..Default::default()
    };
    let config = Arc::new(config);
    let sh = Server {
        clients: Arc::new(Mutex::new(HashMap::new())),
        id: 0,
    };

    tracing::info!("Starting SSH server on port 2222");

    russh::server::run(config, ("0.0.0.0", 2222), sh).await?;

    Ok(())
}

#[derive(Clone)]
struct Server {
    clients: Arc<Mutex<HashMap<(usize, ChannelId), russh::server::Handle>>>,
    id: usize,
}

impl Server {
    async fn post(&mut self, data: CryptoVec) {
        let mut clients = self.clients.lock().await;
        for ((id, channel), ref mut s) in clients.iter_mut() {
            if *id != self.id {
                let _ = s.data(*channel, data.clone()).await;
            }
        }
    }
}

impl server::Server for Server {
    type Handler = Self;
    fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let s = self.clone();
        self.id += 1;
        s
    }
}

#[async_trait]
impl server::Handler for Server {
    type Error = HovelError;

    async fn channel_open_session(
        self,
        channel: Channel<Msg>,
        session: Session,
    ) -> Result<(Self, bool, Session), Self::Error> {
        {
            let mut clients = self.clients.lock().await;
            clients.insert((self.id, channel.id()), session.handle());
            tracing::info!("New client: {}", self.id);
        }
        Ok((self, true, session))
    }

    async fn auth_publickey_offered(
        self,
        user: &str,
        public_key: &key::PublicKey,
    ) -> Result<(Self, server::Auth), Self::Error> {

        let first_chars = public_key.fingerprint();
        tracing::info!("Authenticating offered with public key {} for user {}", first_chars, user);
        Ok((self, server::Auth::Accept))
    }
    async fn auth_publickey(
        self,
        _: &str,
        _: &key::PublicKey,
    ) -> Result<(Self, server::Auth), Self::Error> {
        tracing::info!("Authenticating with public key");
        Ok((self, server::Auth::Accept))
    }

    async fn auth_password(self, _: &str, _: &str) -> Result<(Self, server::Auth), Self::Error> {
        tracing::info!("Rejecting authentication with password");
        Ok((self, server::Auth::UnsupportedMethod))
    }

    async fn data(
        mut self,
        channel: ChannelId,
        data: &[u8],
        mut session: Session,
    ) -> Result<(Self, Session), Self::Error> {
        let data = CryptoVec::from(format!("Got data: {}\r\n", String::from_utf8_lossy(data)));
        self.post(data.clone()).await;

        tracing::info!("Got data: {}", String::from_utf8_lossy(data.as_ref()));
        session.data(channel, data);

        Ok((self, session))
    }

    async fn tcpip_forward(
        self,
        address: &str,
        port: &mut u32,
        session: Session,
    ) -> Result<(Self, bool, Session), Self::Error> {
        let handle = session.handle();
        let address = address.to_string();
        let port = *port;
        tokio::spawn(async move {
            tracing::info!("Forwarding TCP/IP: {}:{}", address, port);
            let mut channel = handle
                .channel_open_forwarded_tcpip(address, port, "1.2.3.4", 1234)
                .await
                .unwrap();
            let _ = channel.data(&b"Hello from a forwarded port"[..]).await;
            let _ = channel.eof().await;
        });

        Ok((self, true, session))
    }
}
