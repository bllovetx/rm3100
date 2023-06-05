/*
    # RM3100 Embedded Server

    ## Pin map

    ### spi (rm3100)
    SCK: pc10, MISO: pc11, MOSI: pc12, CS: PA2
    ### DRDY (rm3100): PA0(bind EXTI0, rise)
    ### trigger output: PA1
    ### trigger input: PC1(bind EXTI1, rise)
*/
// #![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f3xx_hal::pac)]
mod app {
    use stm32f3xx_hal::{
        // self as hal,
        gpio::{
            gpioa::{PA0, PA1, PA2, PA11, PA12},
            gpioc::{PC1, PC10, PC11, PC12,}, 
            gpioe::{PE13},
            Output, Input, Alternate, PushPull, Edge,
        },
        usb::{Peripheral, UsbBus},
        prelude::*, spi::Spi,
        pac::{Peripherals, SPI3},
    };
    use usb_device::{prelude::*, class_prelude::UsbBusAllocator};
    use usbd_serial::{SerialPort, USB_CLASS_CDC};
    use cortex_m::asm;

    const BUFFER_SIZE: usize = 32;

    type AF6 = Alternate<PushPull, 6>;
    type AF14 = Alternate<PushPull, 14>;
    type SCK = PC10<AF6>;
    type MISO = PC11<AF6>;
    type MOSI = PC12<AF6>;
    type SPI = Spi<SPI3, (SCK, MISO, MOSI), u8>;
    type CS = PA2<Output<PushPull>>;
    type DRDY = PA0<Input>;
    type TRIIN = PC1<Input>;
    type TRIOUT = PA1<Output<PushPull>>;
    type SENSOR = rm3100::RM3100<SPI, CS>;
    type LED = PE13<Output<PushPull>>;
    type DM = PA11<AF14>;
    type DP = PA12<AF14>;
    type USBPERIPHERAL = Peripheral<DM, DP>;
    type USBBUS = UsbBus<USBPERIPHERAL>;
    type USBBUSALLOCATOR = UsbBusAllocator<USBBUS>;
    type SERIAL<'a> = SerialPort<'a, USBBUS>;
    type USBDEV<'a> = UsbDevice<'a, USBBUS>;
    type BUFFER = rm3100::mincircularbuffer::MinCircularBuffer<i32, BUFFER_SIZE>;


    #[shared]
    struct Shared{
        trigger_output: TRIOUT,
        sensor: SENSOR,
        buffer: BUFFER,
        overflow: bool,
    }

    #[local]
    struct Local {
        drdy: DRDY,
        trigger_input: TRIIN,
        led: LED,
        serial: SERIAL<'static>,
        usb_dev: USBDEV<'static>,
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
        let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(48.MHz())
            .pclk1(24.MHz())
            .pclk2(24.MHz())
            .freeze(&mut flash.acr);
        assert!(clocks.usbclk_valid());

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

        // config DRDY(PA0) as EXTI0(rise)
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

        // Configure the on-board LED (LD10, south red)
        let mut led = gpioe
            .pe13
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        led.set_low().ok(); // Turn off

        // F3 Discovery board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let mut usb_dp = gpioa
            .pa12
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        usb_dp.set_low().ok();
        asm::delay(clocks.sysclk().0 / 100);

        let usb_dm = gpioa
            .pa11
            .into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);
        let usb_dp = usb_dp.into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);

        let usb = Peripheral {
            usb: dp.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };
        static mut USB_BUS_CONTAINER: Option<USBBUSALLOCATOR> = None;
        unsafe {
            USB_BUS_CONTAINER.replace(UsbBus::new(usb));
        }

        let serial = SerialPort::new(unsafe {USB_BUS_CONTAINER.as_ref().unwrap()});

        let usb_dev = UsbDeviceBuilder::new(unsafe {USB_BUS_CONTAINER.as_ref().unwrap()}, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build();

        // config circular buffer
        let buffer: BUFFER = rm3100::mincircularbuffer::MinCircularBuffer::new(0);

        // init overflow flag
        let overflow: bool = false;

        // config trigger input(PC1) as EXTI1(rise)
        let mut trigger_input: TRIIN = gpioc
            .pc1
            .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr);
        syscfg.select_exti_interrupt_source(&trigger_input);
        trigger_input.trigger_on_edge(&mut exti, Edge::Rising);
        trigger_input.enable_interrupt(&mut exti);


        //let mut mono = Systick::new(cx.core.SYST, 8_000_000);


        (Shared {trigger_output, sensor, buffer, overflow}, Local {drdy, trigger_input, led, serial, usb_dev}, init::Monotonics(),)
    }

    /// listen to usb port
    /// 
    /// TODO: can also be realized in 'interrupt' manner with usb_lp/usb_hp
    #[idle(local = [led, serial, usb_dev], shared = [buffer, overflow])]
    fn idle(mut cx: idle::Context) -> ! {
        let led = cx.local.led;
        let serial = cx.local.serial;
        let usb_dev = cx.local.usb_dev;
        loop {
            if !usb_dev.poll(&mut [serial]) {continue;}
            let mut buf = [0u8; 64];
            let mut outputbuf = [0u8; 5];
            let mut outputlen;

            // read instructions, better one byte one read
            // 0x80: read mag i32, return five bytes, first byte 0 if no data available
            // 0x81: is_overflow? return one byte, 0 if not overflow
            match serial.read(&mut buf) {
                // has instruction
                Ok(count) if count > 0 => {
                    // iter over every instruction
                    for c in buf[0..count].iter() {
                        // encode according to instr
                        match c {
                            0x80 => { // return mag
                                led.set_high().ok();
                                // encode pop result
                                outputbuf = match cx.shared.buffer.lock(
                                    |_buffer| {
                                        _buffer.pop()
                                    }
                                ) {
                                    Some(data) => {
                                        let bytes = data.to_be_bytes();
                                        [1u8, bytes[0], bytes[1], bytes[2], bytes[3]]
                                    },
                                    None => [0u8; 5],
                                };
                                outputlen = 5;
                                led.set_low().ok();
                            },
                            0x81 => {// is overflow?
                                outputbuf[0] = cx.shared.overflow.lock(
                                    |_of| *_of
                                ).into();
                                outputlen = 1;
                            },
                            _ => {outputlen = 0;}
                        }
                        // write
                        let mut write_offsite = 0usize;
                        while write_offsite < outputlen {
                            match serial.write(&outputbuf[write_offsite..outputlen]) {
                                Ok(len) if len > 0 => {
                                    write_offsite += len;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    #[task(binds = EXTI0, local = [drdy], shared = [trigger_output, sensor, buffer, overflow])]
    fn read_result(mut cx: read_result::Context) {
        // TEST: delay after drdy trigger EXTI0
        cx.shared.trigger_output.lock(|triout| {
            triout.set_low().ok();
        });
        // read result, push into buffer, update flag if needed
        (
            cx.shared.sensor,
            cx.shared.buffer,
            cx.shared.overflow
        ).lock(|_sensor, _buffer, _overflow| {
            if !_buffer.push(_sensor.read_magx()) {
                *_overflow = true;
            }
        });
        // clear EXTI0(drdy)
        cx.local.drdy.clear_interrupt();
    }

    #[task(binds = EXTI1, local = [trigger_input], shared = [trigger_output, sensor])]
    fn start_measure(mut cx: start_measure::Context) {
        // TEST: delay after trigger input
        cx.shared.trigger_output.lock(|triout| {
            triout.set_high().ok();
        });
        // start measure x
        cx.shared.sensor.lock(|_sensor| {
            _sensor.start_single_measure(true, false, false);
        });
        // clear EXTI1(trigger_input)
        cx.local.trigger_input.clear_interrupt();
    }

    

}