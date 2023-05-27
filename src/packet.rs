/// spi packet
/// 
/// first byte: read/write address
/// remaining N-1 bytes: data
#[derive(Clone, Copy)]
pub struct Packet<const N:usize> (pub [u8; N]);

impl<const N:usize> Default for Packet<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N:usize> Packet<N> {
    pub fn address(&mut self, address: u8) -> &mut Self {
        self.0[0] = address;
        self
    }
}

// ## byte
impl From<u8> for Packet<2> {
    fn from(data: u8) -> Self {Packet([0, data])}
}

impl From<Packet<2>> for u8 {
    fn from(packet: Packet<2>) -> Self {packet.0[1]}
}

// ## word
impl From<u16> for Packet<3> {
    fn from(data: u16) -> Self {
        Packet([0, (data >> 8) as u8, data as u8])
    }
}

impl From<Packet<3>> for u16 {
    fn from(packet: Packet<3>) -> Self {
        ((packet.0[1] as u16) << 8) | packet.0[2] as u16
    }
}


// ## tri-byte (u24)
impl From<u32> for Packet<4>  {
    fn from(data: u32) -> Self {
        Packet([0, (data >> 16) as u8, (data >> 8) as u8, data as u8])
    }
}

impl From<Packet<4>> for u32 {
    fn from(packet: Packet<4>) -> Self {
        ((packet.0[1] as u32) << 16) | ((packet.0[2] as u32) << 8) | packet.0[3] as u32
    }
}

impl From<Packet<4>> for i32 {
    fn from(packet: Packet<4>) -> Self {
        three_bytes_to_i32((&packet.0[1..4]).try_into().unwrap())
    }
}

// ## triple word
impl From<[u16; 3]> for Packet<7> {
    fn from(data: [u16; 3]) -> Self {
        Packet([0,
            (data[0] >> 8) as u8, data[0] as u8,
            (data[1] >> 8) as u8, data[1] as u8,
            (data[2] >> 8) as u8, data[2] as u8,
        ])
    }
}

// ## triple tri-byte (3xu24)
impl From<Packet<10>> for [i32; 3] {
    fn from(packet: Packet<10>) -> Self {
        [
            three_bytes_to_i32((&packet.0[1..4]).try_into().unwrap()),
            three_bytes_to_i32((&packet.0[4..7]).try_into().unwrap()),
            three_bytes_to_i32((&packet.0[7..10]).try_into().unwrap()),
        ]
    }
}

// assist function: convert u8x3 as i24 to i32
fn three_bytes_to_i32(bytes: &[u8; 3]) -> i32 {
    let prefix = if (bytes[0] & 0x80) != 0 {(0xff as i32) << 24} else {0};
    prefix | ((bytes[0] as i32) << 16) | ((bytes[1] as i32) << 8) | bytes[2] as i32
}