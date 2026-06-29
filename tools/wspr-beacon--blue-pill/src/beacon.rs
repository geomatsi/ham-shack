#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal as _;

#[entry]
fn main() -> ! {
    loop {
        asm::nop();
    }
}
