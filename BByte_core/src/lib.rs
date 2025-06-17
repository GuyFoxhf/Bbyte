use std::io::*;
use std::net::SocketAddr;

pub mod module;

use BByte_util::protocol::http::WSServer;
use BByte_util::protocol::udp::UDPServer;
use BByte_util::{packet::Message, protocol::tcp::*, protocol::Server, *};

pub struct DrakulaServer {
    tcp_server: Option<TcpServer>,
    ws_server: Option<WSServer>,
    udp_server: Option<UDPServer>,
   pub  protocol: DrakulaProtocol,
}

impl DrakulaServer {
    fn cb_connection<CB: 'static + Fn(Message) + Send + Copy>(
        proto: DrakulaProtocol,
        data: Vec<u8>,
        peer_addr: SocketAddr,
        cb: CB,
    ) {
        let msg = Message::new(peer_addr, proto, &data).unwrap();
        cb(msg);
    }

    pub fn new<CB: 'static + Fn(Message) + Send + Copy>(
        protocol: DrakulaProtocol,
        port: u16,
        cb_msg: CB,
    ) -> std::io::Result<Self> {
        match protocol {
            DrakulaProtocol::TCP => {
                match TcpServer::new(
                    format!("0.0.0.0:{}", port).as_str(),
                    DrakulaServer::cb_connection,
                    cb_msg,
                ) {
                    Ok(tcp_server) => Ok(Self {
                        tcp_server: Some(tcp_server),
                        ws_server: None,
                        udp_server: None,
                        protocol,
                    }),
                    Err(e) => Err(e),
                }
            }
            DrakulaProtocol::HTTP => {
                match WSServer::new(
                    format!("0.0.0.0:{}", port).as_str(),
                    DrakulaServer::cb_connection,
                    cb_msg,
                ) {
                    Ok(ws_server) => Ok(Self {
                        tcp_server: None,
                        ws_server: Some(ws_server),
                        udp_server: None,
                        protocol,
                    }),
                    Err(e) => Err(e),
                }
            }
            DrakulaProtocol::UDP => {
                match UDPServer::new(
                    format!("0.0.0.0:{}", port).as_str(),
                    DrakulaServer::cb_connection,
                    cb_msg,
                ) {
                    Ok(udp_server) => Ok(Self {
                        tcp_server: None,
                        ws_server: None,
                        udp_server: Some(udp_server),
                        protocol,
                    }),
                    Err(e) => Err(e),
                }
            }
            DrakulaProtocol::Unknow => panic!("unknow protocol"),
        }
    }

    pub fn sendto(&mut self, peer_addr: &SocketAddr, buf: &[u8]) -> Result<()> {
        match self.protocol {
            DrakulaProtocol::TCP => self.tcp_server.as_mut().unwrap().sendto(peer_addr, buf),
            DrakulaProtocol::HTTP => self.ws_server.as_mut().unwrap().sendto(peer_addr, buf),
            DrakulaProtocol::UDP => self.udp_server.as_mut().unwrap().sendto(peer_addr, buf),
            DrakulaProtocol::Unknow => panic!("unknow protocol"),
        }
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        match self.protocol {
            DrakulaProtocol::TCP => self.tcp_server.as_ref().unwrap().local_addr(),
            DrakulaProtocol::HTTP => self.ws_server.as_ref().unwrap().local_addr(),
            DrakulaProtocol::UDP => self.udp_server.as_ref().unwrap().local_addr(),
            DrakulaProtocol::Unknow => panic!("unknow protocol"),
        }
    }

    pub fn proto(&self) -> DrakulaProtocol {
        self.protocol.clone()
    }

    pub fn contains_addr(&mut self, peer_addr: &SocketAddr) -> bool {
        match self.protocol {
            DrakulaProtocol::TCP => self.tcp_server.as_mut().unwrap().contains_addr(peer_addr),
            DrakulaProtocol::HTTP => self.ws_server.as_mut().unwrap().contains_addr(peer_addr),
            DrakulaProtocol::UDP => self.udp_server.as_mut().unwrap().contains_addr(peer_addr),
            DrakulaProtocol::Unknow => panic!("unknow protocol"),
        }
    }

    pub fn close(&mut self) {
        match self.protocol {
            DrakulaProtocol::TCP => self.tcp_server.as_mut().unwrap().close(),
            DrakulaProtocol::HTTP => self.ws_server.as_mut().unwrap().close(),
            DrakulaProtocol::UDP => self.udp_server.as_mut().unwrap().close(),
            DrakulaProtocol::Unknow => panic!("unknow protocol"),
        }
    }
}
