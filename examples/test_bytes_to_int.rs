#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_semihosting::hprintln;
use panic_halt as _;

#[entry]
fn main() -> ! {
    // !This doesn't work, only convert four bytes to i32
    hprintln!("{:?}", i32::from_be_bytes([0x00, 0x12, 0x34]));
    loop{
        asm::wfi();
    }
}