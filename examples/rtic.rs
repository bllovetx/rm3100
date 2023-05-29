#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f3xx_hal::pac)]
mod app {
    use stm32f3xx_hal::{
        // self as hal,
        gpio::{
            gpioa::{PA2},
            gpioc::{PC10, PC11, PC12,}, 
            Output, Alternate, PushPull,
        },
        prelude::*, spi::Spi,
        pac::{Peripherals, SPI3},
    };
    use cortex_m::asm;

    type AF6 = Alternate<PushPull, 6>;
    type SCK = PC10<AF6>;
    type MISO = PC11<AF6>;
    type MOSI = PC12<AF6>;
    type SPI = Spi<SPI3, (SCK, MISO, MOSI), u8>;
    type CS = PA2<Output<PushPull>>;
    type SENSOR = rm3100::RM3100<SPI, CS>;


    #[shared]
    struct Shared{}

    #[local]
    struct Local {
        sensor: SENSOR,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let dp: Peripherals = cx.device;
        let mut flash = dp.FLASH.constrain();
        let mut rcc = dp.RCC.constrain();
        let mut gpioc= dp.GPIOC.split(&mut rcc.ahb);
        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(48.MHz())
            .pclk1(24.MHz())
            .freeze(&mut flash.acr);

        // config spi
        let sck: SCK= gpioc
            .pc10
            .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);
        let miso: MISO = gpioc
            .pc11
            .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);
        let mosi: MOSI = gpioc
            .pc12
            .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);
        let mut cs: CS = gpioa
                .pa2
                .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        cs.set_high().ok();
        let spi: SPI = Spi::new(dp.SPI3, (sck, miso, mosi), 1.MHz(), clocks, &mut rcc.apb1);

        // config rm3100
        let mut sensor: SENSOR = rm3100::RM3100::new(spi, cs, rm3100::Config::default());
        sensor
            .set_cycle_count(200) 
            .set_update_rate(rm3100::UpdateRate::Hz600) // max update rate
            .set_drdm(rm3100::DRDM::Any); // this also set disable continuous mode

        //let mut mono = Systick::new(cx.core.SYST, 8_000_000);


        (Shared {}, Local {sensor}, init::Monotonics(),)
    }

    #[idle(local = [sensor])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            cx.local.sensor.start_single_measure(
                true, false, false
            );
            asm::delay(1_000_000);
            cx.local.sensor.read_magx();
        }
    }

    

}