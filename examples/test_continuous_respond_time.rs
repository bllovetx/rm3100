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
    let drdy = gpioa
            .pa0
            .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);
    cs.set_high().ok();

    let spi = Spi::new(dp.SPI3, (sck, miso, mosi), 1.MHz(), clocks, &mut rcc.apb1);

    let mut rm3100 = RM3100::new(spi, cs, Config::default());
    rm3100
        .set_cycle_count(200) // 200 cc for each axis
        // DRDY to HIGH after the completion of a measurement on any axis
        // & enable continous mode
        // & measure x
        .write_byte(0x01, 0b00010101); 
    hprintln!("ccx: {:?}", rm3100.read_word(0x04)); // verify ccx
    hprintln!("CMM: 0x{:02X?}", rm3100.read_byte(0x01)); // verify CMM
    rm3100.write_byte(0x0B, 0x97);// set tmrc(data rate)
    hprintln!("TMRC: 0x{:02X?}", rm3100.read_byte(0x0B)); // TMRC: data rate register
    hprintln!("BIST: 0x{:02X?}", rm3100.read_byte(0x33));   
    hprintln!("HSHAKE: 0x{:02X?}", rm3100.read_byte(0x35)); 
    hprintln!("REVID: 0x{:02X?}", rm3100.read_byte(0x36));

    


    loop {
        while drdy.is_low().unwrap() {}
        asm::delay(1_000_000);
        let mags = rm3100.read_mag();
        // hprintln!("{:?}", mags);
    }
}