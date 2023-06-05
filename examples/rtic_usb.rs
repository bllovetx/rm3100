#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]
// #![allow(unused_imports)]

use panic_halt as _;

#[rtic::app(device = stm32f3xx_hal::pac)]
mod app {
    use stm32f3xx_hal::{
        // self as hal,
        gpio::{
            gpioa::{PA11, PA12},
            // gpioc::{PC10, PC11, PC12,}, 
            gpioe::{PE13},
            Output, Alternate, PushPull,
        },
        usb::{Peripheral, UsbBus},
        prelude::*, 
        pac::{Peripherals},
    };
    use usb_device::{prelude::*, class_prelude::UsbBusAllocator};
    use usbd_serial::{SerialPort, USB_CLASS_CDC};
    use cortex_m::asm;

    // type AF6 = Alternate<PushPull, 6>;
    // type SCK = PC10<AF6>;
    // type MISO = PC11<AF6>;
    // type MOSI = PC12<AF6>;
    // type SPI = Spi<SPI3, (SCK, MISO, MOSI), u8>;
    // type CS = PA2<Output<PushPull>>;
    // type SENSOR = rm3100::RM3100<SPI, CS>;
    type AF14 = Alternate<PushPull, 14>;
    type LED = PE13<Output<PushPull>>;
    type DM = PA11<AF14>;
    type DP = PA12<AF14>;
    type USBPERIPHERAL = Peripheral<DM, DP>;
    type USBBUS = UsbBus<USBPERIPHERAL>;
    type USBBUSALLOCATOR = UsbBusAllocator<USBBUS>;
    // type SERIAL<'a> = SerialPort<'a, USBBUS>;
    // type USBDEV<'a> = UsbDevice<'a, USBBUS>;



    #[shared]
    struct Shared{}

    #[local]
    struct Local<'_> {
        led: LED,
        usbbus: USBBUSALLOCATOR,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let dp: Peripherals = cx.device;
        let mut flash = dp.FLASH.constrain();
        let mut rcc = dp.RCC.constrain();
        // let mut gpioc= dp.GPIOC.split(&mut rcc.ahb);
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
        let usb_bus = UsbBus::new(usb);

        // let mut serial = SerialPort::new(&usb_bus);

        // let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        //     .manufacturer("Fake company")
        //     .product("Serial port")
        //     .serial_number("TEST")
        //     .device_class(USB_CLASS_CDC)
        //     .build();

        //let mut mono = Systick::new(cx.core.SYST, 8_000_000);


        (Shared {}, Local {led: led, usbbus: usb_bus}, init::Monotonics(),)
    }

    #[idle(local = [led, usbbus])]
    fn idle(cx: idle::Context) -> ! {
        let led = cx.local.led;
        let usb_bus = cx.local.usbbus;
        let mut serial = SerialPort::new(&usb_bus);

        let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build();

        loop {
            if !usb_dev.poll(&mut [&mut serial]) {
                continue;
            }
    
            let mut buf = [0u8; 64];
    
            match serial.read(&mut buf) {
                Ok(count) if count > 0 => {
                    led.set_high().ok(); // Turn on
    
                    // Echo back in upper case
                    for c in buf[0..count].iter_mut() {
                        if 0x61 <= *c && *c <= 0x7a {
                            *c &= !0x20;
                        }
                    }
    
                    let mut write_offset = 0;
                    while write_offset < count {
                        match serial.write(&buf[write_offset..count]) {
                            Ok(len) if len > 0 => {
                                write_offset += len;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
    
            led.set_low().ok(); // Turn off
        }
    }

    

}