pub mod http;
pub mod tcp;
pub mod udp;
use std::{
    io::*,
    net::SocketAddr,
    ops::{Deref, DerefMut},
};

use crate::{packet::Message, DrakulaProtocol};

use self::{http::WSConnection, tcp::TcpConnection, udp::UDPConnection};

static TUNNEL_FLAG: [u8; 4] = [0x38, 0x38, 0x38, 0x38];

pub trait Client {
    fn connect(address: &str) -> Result<Self>
    where
        Self: Sized;
    fn tunnel(remote_addr: &str, server_local_port: u16) -> Result<Self>
    where
        Self: Sized;
    fn recv(&mut self) -> Result<Vec<u8>>;
    fn send(&mut self, buf: &mut [u8]) -> Result<()>;
    fn local_addr(&self) -> Result<SocketAddr>;
    fn close(&mut self);
}

pub trait Server {
    fn new<
        CBCB: 'static + Fn(Message) + Send + Copy,
        CB: 'static + Fn(DrakulaProtocol, Vec<u8>, SocketAddr, CBCB) + Send,
    >(
        address: &str,
        cb_data: CB,
        cbcb: CBCB,
    ) -> std::io::Result<Self>
    where
        Self: Sized;

    fn local_addr(&self) -> Result<SocketAddr>;
    fn sendto(&mut self, peer_addr: &SocketAddr, buf: &[u8]) -> Result<()>;
    fn contains_addr(&mut self, peer_addr: &SocketAddr) -> bool;
    fn close(&mut self);
}

pub struct ClientWrapper {
    typ: DrakulaProtocol,
    tcp_client: Option<TcpConnection>,
    http_client: Option<WSConnection>,
    udp_client: Option<UDPConnection>,
}

impl Deref for ClientWrapper {
    type Target = dyn Client;

    fn deref(&self) -> &Self::Target {
        match self.typ {
            DrakulaProtocol::TCP => self.tcp_client.as_ref().unwrap(),
            DrakulaProtocol::HTTP => self.http_client.as_ref().unwrap(),
            DrakulaProtocol::UDP => self.udp_client.as_ref().unwrap(),
            DrakulaProtocol::Unknow => panic!("unknow protocol"),
        }
    }
}

impl DerefMut for ClientWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.typ {
            DrakulaProtocol::TCP => self.tcp_client.as_mut().unwrap(),
            DrakulaProtocol::HTTP => self.http_client.as_mut().unwrap(),
            DrakulaProtocol::UDP => self.udp_client.as_mut().unwrap(),
            DrakulaProtocol::Unknow => panic!("unknow protocol"),
        }
    }
}

impl Clone for ClientWrapper {
    fn clone(&self) -> Self {
        Self {
            typ: self.typ.clone(),
            tcp_client: self.tcp_client.clone(),
            http_client: self.http_client.clone(),
            udp_client: self.udp_client.clone(),
        }
    }
}

impl ClientWrapper {
    pub fn connect(typ: &DrakulaProtocol, address: &str) -> Result<Self> {
        match typ {
            DrakulaProtocol::TCP => {
                let client = TcpConnection::connect(address)?;
                Ok(Self {
                    typ: typ.clone(),
                    tcp_client: Some(client),
                    http_client: None,
                    udp_client: None,
                })
            }
            DrakulaProtocol::HTTP => {
                let client = WSConnection::connect(address)?;
                Ok(Self {
                    typ: typ.clone(),
                    tcp_client: None,
                    http_client: Some(client),
                    udp_client: None,
                })
            }
            DrakulaProtocol::UDP => {
                let client = UDPConnection::connect(address)?;
                Ok(Self {
                    typ: typ.clone(),
                    tcp_client: None,
                    http_client: None,
                    udp_client: Some(client),
                })
            }
            DrakulaProtocol::Unknow => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invaild protocol type",
                ))
            }
        }
    }
}

pub fn create_tunnel(
    addr: &str,
    protocol: &DrakulaProtocol,
    server_local_port: u16,
) -> Result<Box<dyn Client>> {
    Ok(match protocol {
        DrakulaProtocol::TCP => Box::new(TcpConnection::tunnel(addr, server_local_port)?),
        DrakulaProtocol::HTTP => Box::new(WSConnection::tunnel(addr, server_local_port)?),
        DrakulaProtocol::UDP => Box::new(UDPConnection::tunnel(addr, server_local_port)?),
        DrakulaProtocol::Unknow => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            ));
        }
    })
}
