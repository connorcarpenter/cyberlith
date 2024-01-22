use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use naia_serde::{BitReader, BitWrite, ConstBitLength, SerdeErr, SerdeInternal as Serde};

#[derive(PartialEq, Clone, Debug)]
pub struct SerdeSocketAddr(SocketAddr);

impl SerdeSocketAddr {
    pub fn new(socket_addr: SocketAddr) -> Self {
        Self(socket_addr)
    }

    pub fn inner(&self) -> SocketAddr {
        self.0
    }
}

impl Serde for SerdeSocketAddr {
    fn ser(&self, writer: &mut dyn BitWrite) {
        let addr = self.0;
        let is_ipv4 = addr.is_ipv4();
        is_ipv4.ser(writer);

        match addr {
            SocketAddr::V4(addr_impl) => {
                let octets = addr_impl.ip().octets();
                octets.ser(writer);

                let port = addr_impl.port();
                port.ser(writer);
            }
            SocketAddr::V6(addr_impl) => {
                let octets = addr_impl.ip().octets();
                octets.ser(writer);

                let port = addr_impl.port();
                port.ser(writer);

                let flowinfo = addr_impl.flowinfo();
                flowinfo.ser(writer);

                let scope_id = addr_impl.scope_id();
                scope_id.ser(writer);
            }
        }
    }

    fn de(reader: &mut BitReader) -> Result<Self, SerdeErr> {
        let is_ipv4 = bool::de(reader)?;

        match is_ipv4 {
            true => {
                let octets: [u8; 4] = <[u8; 4]>::de(reader)?;
                let port = u16::de(reader)?;

                let ip_addr = Ipv4Addr::from(octets);
                let socket_addr = SocketAddrV4::new(ip_addr, port);
                Ok(SerdeSocketAddr(SocketAddr::from(socket_addr)))
            }
            false => {
                let octets: [u8; 16] = <[u8; 16]>::de(reader)?;
                let port = u16::de(reader)?;
                let flowinfo = u32::de(reader)?;
                let scope_id = u32::de(reader)?;

                let ip_addr = Ipv6Addr::from(octets);
                let socket_addr = SocketAddrV6::new(ip_addr, port, flowinfo, scope_id);

                Ok(SerdeSocketAddr(SocketAddr::from(socket_addr)))
            }
        }
    }

    fn bit_length(&self) -> u32 {
        let mut count = 0;

        // is ipv4 ?
        count += 1;

        match self.0.is_ipv4() {
            true => {
                // ipv4
                count += <[u8; 4]>::const_bit_length();
                count += u16::const_bit_length();
            }
            false => {
                // ipv6
                count += <[u8; 16]>::const_bit_length();
                count += u16::const_bit_length();
                count += u32::const_bit_length();
                count += u32::const_bit_length();
            }
        }

        count
    }
}