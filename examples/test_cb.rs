#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(unused)]

use cortex_m::asm;
use cortex_m_semihosting::hprintln;
use panic_halt as _;
use cortex_m_rt::entry;

use stm32f3xx_hal as hal;
use hal::pac;
use hal::prelude::*;

use rm3100::mincircularbuffer::MinCircularBuffer;



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
    let mut buffer = MinCircularBuffer::<u32, 5>::new(0);
    hprintln!("{:?}", buffer.pop());
    hprintln!("{:?}", buffer.push(1));
    hprintln!("{:?}", buffer.push(2));
    hprintln!("{:?}", buffer.push(3));
    hprintln!("{:?}", buffer.pop());
    hprintln!("{:?}", buffer.pop());
    hprintln!("{:?}", buffer.pop());
    hprintln!("{:?}", buffer.push(1));
    hprintln!("{:?}", buffer.push(2));
    hprintln!("{:?}", buffer.push(3));
    hprintln!("{:?}", buffer.pop());
    hprintln!("{:?}", buffer.pop());
    hprintln!("{:?}", buffer.pop());
    hprintln!("{:?}", buffer.push(1));
    hprintln!("{:?}", buffer.push(2));
    hprintln!("{:?}", buffer.push(3));
    hprintln!("{:?}", buffer.pop());
    hprintln!("{:?}", buffer.pop());
    hprintln!("{:?}", buffer.pop());

    
    loop{
        asm::wfi();
    }
}