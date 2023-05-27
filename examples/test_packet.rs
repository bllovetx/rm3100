#![no_std]
#![no_main]
#![allow(dead_code)]

use cortex_m::asm;
use cortex_m_semihosting::hprintln;
use panic_halt as _;
use cortex_m_rt::entry;

use stm32f3xx_hal as hal;
use hal::pac;
use hal::prelude::*;

use rm3100::packet::Packet;


#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);
    let mut packet = Packet([0, 0xff, 0xff, 0xff]);
    let res: i32 = packet.into();
    assert_eq!(res, -1);
    packet.0 = [0, 0, 0, 0] as [u8;4];
    let res: i32 = packet.into();
    assert_eq!(res, 0);
    
    loop{
        asm::wfi();
    }
}