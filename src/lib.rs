pub fn get_port(buf: &[u8], start: usize) -> u16 {
    (buf[start] as u16) << 8 | (buf[start + 1] as u16)
}

pub enum S5Addr {
    IPv4(u8, u8, u8, u8),
    Domain(String),
}

pub struct S5Request {
    pub ver: u8,
    pub cmd: u8,
    pub atype: u8,
    pub dst_addr: S5Addr,
    pub dst_port: u16,
}

impl S5Request {
    pub fn new(ver: u8, cmd: u8, atype: u8, dst_addr: S5Addr, dst_port: u16) -> S5Request {
        S5Request {
            ver,
            cmd,
            atype,
            dst_addr,
            dst_port,
        }
    }

    pub fn into_addr_string(self) -> String {
        match self.dst_addr {
            S5Addr::IPv4(a, b, c, d) => {
                format!("{}.{}.{}.{}:{}", a, b, c, d, self.dst_port)
            }
            S5Addr::Domain(s) => {
                format!("{}:{}", s, self.dst_port)
            }
        }
    }
}
