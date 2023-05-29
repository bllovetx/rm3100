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

use rm3100::{RM3100, Config, Status, UpdateRate};

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
    cs.set_high().ok();

    let spi = Spi::new(dp.SPI3, (sck, miso, mosi), 1.MHz(), clocks, &mut rcc.apb1);

    let mut rm3100 = RM3100::new(spi, cs, Config::default());
    rm3100
        .set_cycle_count(0xC8)
        .write_byte(0x01, 0b00000100);
    hprintln!("{:?}", rm3100.check_connect(0x22)); // check connect
    hprintln!("{:?}", rm3100.read_word(0x04));
    
    hprintln!("{:02X?}", rm3100.read_byte(0x01));
    hprintln!("{:?}", f32::from(UpdateRate::Hz0_075));
    
    
    // rm3100.write_byte(0x04 as u8, 0x1);
    // rm3100.write_byte(0x05 as u8, 0xC8);
    // let temp = rm3100.read_byte(0x04 as u8);
    // let temp = rm3100.read_byte(0x05 as u8);
    // hprintln!("{:?}", temp);
    // let temp = rm3100.read_byte(0x06 as u8);
    // hprintln!("{:?}", temp);
    // let temp = rm3100.read_byte(0x07 as u8);
    // hprintln!("{:?}", temp);
    // rm3100.write_word(0x04, 12345);
    // hprintln!("{:?}", rm3100.read_word(0x04));

    // let rm3100_revid = spi.transfer(&mut read_seq).unwrap();
    // hprintln!("{:?}", rm3100_revid);

    // // write 
    // read_seq = [0x00, 0x0F];
    // cs.set_low().ok();
    // spi.write(&mut read_seq);
    // cs.set_high().ok();

    // read
    // read_seq = [0xA4, 0x00, 0x00, 0x00];
    // cs.set_low().ok();
    // let temp = spi.transfer(&mut read_seq);
    // cs.set_high().ok();
    // hprintln!("{:?}", temp);

    // let result = spi.transfer(&mut read_tmrc).unwrap();
    // hprintln!("{:?}", result);
    // let result = spi.transfer(&mut read_mx).unwrap();
    // hprintln!("{:?}", result);

    // spi.send(0x8B as u8).unwrap();



    // // Create an `u8` array, which can be transfered via SPI.
    // let msg_send: [u8; 8] = [0xD, 0xE, 0xA, 0xD, 0xB, 0xE, 0xE, 0xF];
    // // Copy the array, as it would be mutually shared in `transfer` while simultaneously would be
    // // immutable shared in `assert_eq`.
    // let mut msg_sending = msg_send;
    // // Transfer the content of the array via SPI and receive it's output.
    // // When MOSI and MISO pins are connected together, `msg_received` should receive the content.
    // // from `msg_sending`
    // let msg_received = spi.transfer(&mut msg_sending).unwrap();

    // // Check, if msg_send and msg_received are identical.
    // // This succeeds, when master and slave of the SPI are connected.
    // assert_eq!(msg_send, msg_received);

    loop {
        // // Transfer the content of the array via SPI and receive it's output.
        // // When MOSI and MISO pins are connected together, `msg_received` should receive the content.
        // // from `msg_sending`
        // let msg_received = spi.transfer(&mut msg_sending).unwrap();

        // // Check, if msg_send and msg_received are identical.
        // // This succeeds, when master and slave of the SPI are connected.
        // // hprintln!("{:?}", msg_received);
        // assert_eq!(msg_send, msg_received);
        
        // spi.send(&mut read_tmrc).unwrap();
        // let temp = spi.read().unwrap();

        // let mut read_seqing = read_seq;
        // let temp = spi.transfer(&mut read_seqing).unwrap();

        // spi.send(0x8B as u8).unwrap();
        // asm::delay(100_000);
        // asm::wfi();
        // let temp = spi.read();
        // hprintln!("{:?}", temp);
        rm3100.start_single_measure(true, false, false);
        asm::delay(100_000_000);
        // hprintln!("{:02X?}", rm3100.read_byte(0x34));
        let init_status = rm3100.read_byte(0x34);
        let mags = rm3100.read_mag();
        let final_status = rm3100.read_byte(0x34);
        hprintln!("{:02X?}", init_status);
        hprintln!("{:?}", mags);
        hprintln!("{:02X?}", final_status);
        
    }
}