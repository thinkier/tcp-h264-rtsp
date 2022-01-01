use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub stream: HashMap<String, Stream>,
}

fn default_host() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}

fn default_port()->u16{
    554
}

#[derive(Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Deserialize)]
pub struct Stream {
    pub addr: String,
}