// approximately 5us latency
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
            gpioa::{PA0, PA1, PA2},
            gpioc::{PC10, PC11, PC12,}, 
            Output, Input, Alternate, PushPull, Edge,
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
    type DRDY = PA0<Input>;
    type TRIOUT = PA1<Output<PushPull>>;
    type SENSOR = rm3100::RM3100<SPI, CS>;


    #[shared]
    struct Shared{
        trigger_output: TRIOUT,
    }

    #[local]
    struct Local {
        sensor: SENSOR,
        drdy: DRDY,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let dp: Peripherals = cx.device;
        let mut flash = dp.FLASH.constrain();
        let mut rcc = dp.RCC.constrain();
        let mut syscfg = dp.SYSCFG.constrain(&mut rcc.apb2);
        let mut exti = dp.EXTI;
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

        // config DRDY(PA0) as EXTI0
        let mut drdy: DRDY = gpioa
            .pa0
            .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);
        syscfg.select_exti_interrupt_source(&drdy);
        drdy.trigger_on_edge(&mut exti, Edge::Rising);
        drdy.enable_interrupt(&mut exti);

        // config triger_output: used for oscillator test
        let mut trigger_output = gpioa
            .pa1
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        trigger_output.set_low().ok();

        //let mut mono = Systick::new(cx.core.SYST, 8_000_000);


        (Shared {trigger_output}, Local {sensor, drdy}, init::Monotonics(),)
    }

    #[idle(local = [sensor], shared = [trigger_output])]
    fn idle(mut cx: idle::Context) -> ! {
        loop {
            cx.local.sensor.start_single_measure(
                true, false, false
            );
            asm::delay(1_000_000);
            cx.shared.trigger_output.lock( |triout| {
                triout.set_low().ok();
            });
            cx.local.sensor.read_magx();
        }
    }

    #[task(binds = EXTI0, local = [drdy], shared = [trigger_output])]
    fn test_delay(mut cx: test_delay::Context) {
        cx.shared.trigger_output.lock(|triout| {
            triout.set_high().ok();
        });
        cx.local.drdy.clear_interrupt();
    }

    

}