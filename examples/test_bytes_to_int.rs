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


fn three_bytes_to_i32(bytes: [u8; 3]) -> i32 {
    let prefix = if (bytes[0] & 0x80) != 0 {(0xff as i32) << 24} else {0};
    prefix | ((bytes[0] as i32) << 16) | ((bytes[1] as i32) << 8) | bytes[2] as i32
}

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
    // !This doesn't work, only convert four bytes to i32
    // hprintln!("{:?}", i32::from_be_bytes([0x00, 0x12, 0x34]));
    hprintln!("{:?}", three_bytes_to_i32([0x80, 0x0, 0x0])).ok();
    let list: [u8; 4] = [1, 2, 3, 4];
    hprintln!("{:?}", &list[1..4].len());
    loop{
        asm::wfi();
    }
}