//! Example of configuring spi.
//! Target board: STM32F3DISCOVERY
#![no_std]
#![no_main]

use cortex_m_semihosting::hprintln;
use panic_halt as _;

use stm32f3xx_hal as hal;

use cortex_m::asm;
use cortex_m_rt::entry;

use hal::pac;
use hal::prelude::*;
use hal::spi::Spi;

use rm3100::{RM3100, Config, Status};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    // Configure pins for SPI
    let sck = gpioc
        .pc10
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);
    let miso = gpioc
        .pc11
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);
    let mosi = gpioc
        .pc12
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);

    
    let mut cs = gpioa
            .pa2
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    let mut trigger = gpioa
            .pa1
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    let drdy = gpioa
            .pa0
            .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);
    cs.set_high().ok();
    trigger.set_low().ok();

    let spi = Spi::new(dp.SPI3, (sck, miso, mosi), 1.MHz(), clocks, &mut rcc.apb1);

    let mut rm3100 = RM3100::new(spi, cs, Config::default());
    rm3100
        .set_cycle_count(0xC8) // 200 cc for each axis
        .write_byte(0x01, 0b00000100); // DRDY to HIGH after the completion of a measurement on any axis & disable continous mode
    hprintln!("{:?}", rm3100.read_word(0x04)); // verify ccx
    hprintln!("{:02X?}", rm3100.read_byte(0x01)); // verify CMM
    rm3100.write_byte(0x0B, 0x92);
    hprintln!("{:02X?}", rm3100.read_byte(0x0B)); // TMRC: data rate register
    


    loop {
        rm3100.start_single_measure(true, false, false);
        trigger.set_high().ok();
        asm::delay(100_000);
        trigger.set_low().ok();
        // hprintln!("{:02X?}", rm3100.read_byte(0x34));
        // let init_status = drdy.is_high();//rm3100.read_byte(0x34);
        let mags = rm3100.read_mag();
        // let final_status = drdy.is_high();//rm3100.read_byte(0x34);
        // hprintln!("{:?}", init_status);
        // hprintln!("{:?}", mags);
        // hprintln!("{:?}", final_status);
        
    }
}