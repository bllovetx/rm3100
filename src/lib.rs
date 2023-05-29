#![no_std]

pub mod packet;
use packet::Packet;

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

// masks
const PMX_MASK: u8 = 0x10;
const PMY_MASK: u8 = 0x20;
const PMZ_MASK: u8 = 0x30;

// bit_shift
const PMX_SHIFT: u8 = 4;
const PMY_SHIFT: u8 = 5;
const PMZ_SHIFT: u8 = 6;
const CMX_SHIFT: u8 = 4;
const CMY_SHIFT: u8 = 5;
const CMZ_SHIFT: u8 = 6;
const STATUS_SHIFT: u8 = 7;



#[derive(Clone, Copy)]
pub struct CycleCount {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

impl Default for CycleCount {
    fn default() -> Self {
        CycleCount { x: 200, y: 200, z: 200 }
    }
}

#[derive(PartialEq)]
pub enum Status {
    Available,
    Unavailable,
}


impl From<bool> for Status {
    fn from(bit: bool) -> Self {
        if bit {Status::Available} else {Status::Unavailable}
    }
}

#[derive(Clone, Copy)]
pub enum UpdateRate {
    Hz600 = 0x92,
    Hz300 = 0x93,
    Hz150 = 0x94,
    Hz75 = 0x95,
    Hz37 = 0x96,
    Hz18 = 0x97,
    Hz9 = 0x98,
    Hz4_5 = 0x99,
    Hz2_3 = 0x9A,
    Hz1_2 = 0x9B,
    Hz0_6 = 0x9C,
    Hz0_3 = 0x9D,
    Hz0_15 = 0x9E,
    Hz0_075 = 0x9F,
}

impl From<f32> for UpdateRate {
    fn from(rate: f32) -> Self {
        let factor = ((600 as f32) / rate) as u32;
        match factor {
            0..=1 => UpdateRate::Hz600,
            2..=3 => UpdateRate::Hz300,
            4..=7 => UpdateRate::Hz150,
            8..=15 => UpdateRate::Hz75,
            16..=31 => UpdateRate::Hz37,
            32..=63 => UpdateRate::Hz18,
            64..=127 => UpdateRate::Hz9,
            128..=255 => UpdateRate::Hz4_5,
            256..=511 => UpdateRate::Hz2_3,
            512..=1023 => UpdateRate::Hz1_2,
            1024..=2047 => UpdateRate::Hz0_6,
            2048..=4095 => UpdateRate::Hz0_3,
            4096..=8191 => UpdateRate::Hz0_15,
            _ => UpdateRate::Hz0_075,
        }
    }
}

impl From<UpdateRate> for f32 {
    fn from(rate: UpdateRate) -> Self {
        600 as f32 / (1 << ((0xF & rate as u8) - 2)) as f32
    }
}

impl Default for UpdateRate {
    fn default() -> Self {
        UpdateRate::Hz600
    }
}

#[derive(Clone, Copy)]
pub enum DRDM {
    AlarmFull = 0b0000,
    Any = 0b0100,
    Full = 0b1000,
    Alarm = 0b1100,
}

impl Default for DRDM {
    fn default() -> Self {
        DRDM::Full
    }
}

pub struct  Config {
    pub cc: CycleCount,
    pub rate: UpdateRate,
    pub drdm: DRDM,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cc: CycleCount::default(),
            rate: UpdateRate::default(),
            drdm: DRDM::default(),
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
    ) -> &mut Self
    where InputType: Into<Packet<N>> {
        let mut packet: Packet<N> = value.into();
        packet.address(address);
        self.cs.set_low().ok();
        self.spi.write(&mut packet.0).ok();
        self.cs.set_high().ok();
        self
    }

    pub fn read_byte(&mut self, address: u8) -> u8 {
        self.read_bytes::<2, u8>(address)
    }

    pub fn write_byte(&mut self, address: u8, value: u8) -> &mut Self {
        self.write_bytes::<2, u8>(address, value)
    }

    pub fn read_word(&mut self, address: u8) -> u16 {
        self.read_bytes::<3, u16>(address)
    }

    pub fn write_word(&mut self, address: u8, value: u16) -> &mut Self {
        self.write_bytes::<3, u16>(address, value)
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
    pub fn set_cycle_count_x(&mut self, ccx: u16) -> &mut Self {
        self.config.cc.x = ccx;
        self.write_word(CCX_REG, ccx)
    }

    pub fn set_cycle_count_y(&mut self, ccy: u16) -> &mut Self {
        self.config.cc.y = ccy;
        self.write_word(CCY_REG, ccy)
    }

    pub fn set_cycle_count_z(&mut self, ccz: u16) -> &mut Self {
        self.config.cc.z = ccz;
        self.write_word(CCZ_REG, ccz)
    }

    pub fn set_cycle_count_xyz(
        &mut self, ccx: u16, ccy: u16, ccz: u16
    ) -> &mut Self {
        self.config.cc = CycleCount{x: ccx, y:ccy, z:ccz};
        self.write_bytes::<7, [u16; 3]>(CCX_REG, [ccx, ccy, ccz])
    }

    pub fn set_cycle_count(&mut self, cc: u16) -> &mut Self {
        self.set_cycle_count_xyz(cc, cc, cc)
    }

    pub fn get_cycle_count(&mut self) -> CycleCount {self.config.cc}

    /// ## Set Update Rate (TMRC)
    /// 
    /// rate: use enum UpdateRate or use f32.into()
    pub fn set_update_rate(&mut self, rate: UpdateRate) -> &mut Self {
        self.config.rate = rate;
        self.write_byte(TMRC_REG, rate as u8)
    }

    pub fn get_update_rate(&mut self) -> UpdateRate {self.config.rate}

    /// ## Set DRDY Mode (CMM bit 3&2)
    /// 
    /// Alarm is omitted currently
    pub fn set_drdm(&mut self, mode: DRDM) -> &mut Self {
        self.config.drdm = mode;
        self.write_byte(CMM_REG, mode as u8)
    }


    // # IO
    /// ## start single measurement
    /// 
    /// require user to ensure START(bit 0 of CMM) to be 0 
    /// for efficiency
    pub fn start_single_measure(
        &mut self, x: bool, y: bool, z: bool
    ) {
        self.write_byte(POLL_REG, 
            ((x as u8) << PMX_SHIFT) |
            ((y as u8) << PMY_SHIFT) |
            ((z as u8) << PMZ_SHIFT)
        );
    }

    /// ## start continuous measurement
    pub fn start_continuous_measure(
        &mut self, x: bool, y: bool, z: bool
    ) {
        self.write_byte(CMM_REG, 
            self.config.drdm as u8 |
            ((x as u8) << CMX_SHIFT) |
            ((y as u8) << CMY_SHIFT) |
            ((z as u8) << CMZ_SHIFT) |
            true as u8 // Start bit
        );
    }

    /// ## stop continuous measurement
    pub fn stop_continuous_measure(&mut self) -> &mut Self {
        self.write_byte(CMM_REG, 
            self.config.drdm as u8 | false as u8
        )
    }

    /// ## check connect
    /// 
    /// compare revid (0x22 for rm3100 from wit)
    pub fn check_connect(&mut self, revid: u8) -> bool {
        self.read_byte(REVID_REG) == revid
    }

    /// ## DRDY by spi
    pub fn get_status(&mut self) -> Status {
        ((self.read_byte(STATUS_REG) 
        >> STATUS_SHIFT) != 0).into()
    }

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