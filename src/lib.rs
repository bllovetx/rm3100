#![no_std]

use core::str::Bytes;

use embedded_hal::{self, digital::v2::OutputPin};

// regs
const POLL_REG: u8 = 0x00;
const CMM_REG: u8 = 0x01;
const CCX_REG: u8 = 0x04;
const CCY_REG: u8 = 0x06;
const CCZ_REG: u8 = 0x08;
const TMRC_REG: u8 = 0x0B;
const MX_REG: u8 = 0x24;
const MY_REG: u8 = 0x27;
const MZ_REG: u8 = 0x2A;
const BIST_REG: u8 = 0x33;
const STATUS_REG: u8 = 0x34;
const HSHAKE_REG: u8 = 0x35;
const REVID_REG: u8 = 0x36;

// flags
const READ_FLAG: u8 = 0x80;

/// spi packet
/// 
/// first byte: read/write address
/// remaining N-1 bytes: data
struct Packet<const N:usize> ([u8; N]);

impl<const N:usize> Default for Packet<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N:usize> Packet<N> {
    pub fn address(&mut self, address: u8) {
        self.0[0] = address;
    }
}

impl From<u16> for Packet<3> {
    fn from(data: u16) -> Self {
        Packet::<3>([0, (data >> 8) as u8, data as u8])
    }
}

impl From<Packet<3>> for u16 {
    fn from(packet: Packet<3>) -> Self {
        ((packet.0[1] as u16) << 8) | packet.0[2] as u16
    }
}

#[derive(Clone, Copy)]
pub struct CycleCount {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

pub enum Status {
    Available,
    Unavailable,
}

impl From<bool> for Status {
    fn from(bit: bool) -> Self {
        if bit {Status::Available} else {Status::Unavailable}
    }
}

pub struct  Config {
    pub cc: CycleCount,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cc: CycleCount { x: 200, y: 200, z: 200 }
        }
    }
}

pub struct RM3100<Spi, CsPin> {
    spi: Spi,
    cs: CsPin,
    config: Config
}

impl<Spi, SpiError, CsPin, PinError> RM3100<Spi, CsPin> 
where
    Spi: embedded_hal::blocking::spi::Transfer<u8, Error = SpiError>
        + embedded_hal::blocking::spi::Write<u8, Error = SpiError>,
    CsPin: OutputPin<Error = PinError>,
{
    pub fn new(spi: Spi, cs: CsPin, config: Config) -> Self {
        let mut rm3100 = RM3100 {
            spi,
            cs,
            config,
        };
        rm3100.cs.set_high().ok();
        rm3100
    }

    // # basic interface
    /// read/write N-1 bytes
    /// 
    /// N: packet length(address + data)
    /// N-1: for efficiency and rust const generic restriction 
    pub fn read_bytes<const N: usize, OutPutType>(
        &mut self, address: u8
    ) -> OutPutType
    where OutPutType: From<Packet<N>>
    {
        let mut packet = *Packet::<N>::default()
            .address(READ_FLAG | address);
        self.cs.set_low().ok();
        self.spi.transfer(&mut packet.0).ok();
        self.cs.set_high().ok();
        OutPutType::from(packet)
    }

    pub fn write_bytes<const N: usize, InputType>(
        &mut self, address: u8, value: InputType
    ) where InputType: Into<Packet<N>>
    {
        let mut packet: Packet<N> = value.into();
        packet.address(address);
        self.cs.set_low().ok();
        self.spi.write(&mut packet.0).ok();
        self.cs.set_high().ok();
    }

    fn read_byte(&mut self, address: u8) -> u8 {
        let mut bytes = [READ_FLAG | address, 0x0];
        self.cs.set_low().ok();
        self.spi.transfer(&mut bytes).ok();
        self.cs.set_high().ok();
        bytes[1]
    }

    fn write_byte(&mut self, address: u8, value: u8) {
        let mut request = [address, value];
        self.cs.set_low().ok();
        self.spi.write(&mut request).ok();
        self.cs.set_high().ok();
    }

    fn read_word(&mut self, address: u8) -> u16 {
        let mut bytes = [READ_FLAG | address, 0x0, 0x0];
        self.cs.set_low().ok();
        self.spi.transfer(&mut bytes).ok();
        self.cs.set_high().ok();
        ((bytes[1] as u16) << 8) | bytes[2] as u16
    }

    fn write_word(&mut self, address: u8, value: u16) {
        let mut request= [address, (value >> 8) as u8, value as u8];
        self.cs.set_low().ok();
        self.spi.write(&mut request).ok();
        self.cs.set_high().ok();
    }

    /// read N-1 bytes
    /// 
    /// N-1: for efficiency and rust const generic restriction 
    fn read_bytes<const N: usize, OutPutType>(&mut self, address: u8) -> OutPutType
    where OutPutType: From<Packet<N>>
    {
        let mut bytes: [u8; N] = [0; N];
        bytes[0] = READ_FLAG | address;
        self.cs.set_low().ok();
        self.spi.transfer(&mut bytes).ok();
        self.cs.set_high().ok();
        OutPutType::from(Packet::<N>{data: bytes})
    }

    fn read_tri_bytes(&mut self, address: u8) -> u32 {
        let mut bytes = [READ_FLAG | address, 0x0, 0x0, 0x0];
        self.cs.set_low().ok();
        self.spi.transfer(&mut bytes).ok();
        self.cs.set_high().ok();
        ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | bytes[3] as u32
    }

    // # configurations

    /// ## Set the Cycle Count Registers (0x04 – 0x09)
    /// 
    /// Increasing the cycle count value increases measurement gain and resolution. 
    /// Lowering the cycle count value reduces acquisition time, which increases maximum achievable sample rate 
    /// or, with a fixed sample rate, decreases power consumption.
    /// 
    /// quantization issues generally dictate working above a cycle count value of ~30, while noise limits the useful upper range to ~400 cycle counts.
    /// 
    /// value type: u16
    /// default: 0x00C8(200)
    pub fn set_cycle_count_x(&mut self, ccx: u16) {
        self.write_word(CCX_REG, ccx);
        self.config.cc.x = ccx;
    }

    pub fn set_cycle_count_y(&mut self, ccy: u16) {
        self.write_word(CCY_REG, ccy);
        self.config.cc.y = ccy;
    }

    pub fn set_cycle_count_z(&mut self, ccz: u16) {
        self.write_word(CCZ_REG, ccz);
        self.config.cc.z = ccz;
    }

    pub fn set_cycle_count_xyz(
        &mut self, ccx: u16, ccy: u16, ccz: u16
    ) {
        self.write_bytes::<7, [u16; 3]>(CCX_REG, [ccx, ccy, ccz]);
        self.config.cc = CycleCount{x: ccx, y:ccy, z:ccz};
    }

    pub fn set_cycle_count(&mut self, cc: u16) {
        self.set_cycle_count_xyz(cc, cc, cc);
    }

    pub fn get_cycle_count(&mut self) -> CycleCount {self.config.cc}

    // # IO
    /// ## Read mag field
    pub fn read_magx(&mut self) -> i32 {
        self.read_bytes::<4, i32>(MX_REG)
    }

    pub fn read_magy(&mut self) -> i32 {
        self.read_bytes::<4, i32>(MY_REG)
    }

    pub fn read_magz(&mut self) -> i32 {
        self.read_bytes::<4, i32>(MZ_REG)
    }

    pub fn read_mag(&mut self) -> [i32; 3] {
        self.read_bytes::<10, [i32;3]>(MX_REG)
    }

    
    

    
}