#![no_main]
#![no_std]

use cortex_m_rt as rt;
use nb::block;
use panic_rtt_target as _;
use rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    let mut c: u8 = 0;

    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut gpioc = dp.GPIOC.split(&mut rcc);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut tmr = dp.TIM3.counter_hz(&mut rcc);

    rtt_init_print!();

    tmr.start(1.Hz()).unwrap();

    loop {
        c += 1;
        rprintln!("cycle {}", c);

        led.set_high();
        block!(tmr.wait()).ok();
        led.set_low();
        block!(tmr.wait()).ok();
    }
}
