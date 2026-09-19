#![no_main]
#![no_std]

#[cfg(feature = "rtt-log")]
use panic_rtt_target as _;

#[cfg(feature = "rtt-log")]
use rtt_target::rtt_init_print;

#[cfg(not(feature = "rtt-log"))]
use panic_halt as _;

use cortex_m_rt as rt;
use nb::block;
use rt::entry;
use stm32f1xx_hal::gpio::{ErasedPin, Output, PushPull};
use stm32f1xx_hal::{pac, prelude::*};

use wspr_beacon::beacon::config;
use wspr_beacon::wspr_log;

#[entry]
fn main() -> ! {
    let mut c: u8 = 0;

    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();

    let mut led: ErasedPin<Output<PushPull>> = match config::CFG.hw.model {
        config::BluePill::Classic => {
            let mut gpioc = dp.GPIOC.split(&mut rcc);
            gpioc.pc13.into_push_pull_output(&mut gpioc.crh).erase()
        }
        config::BluePill::Plus => {
            let mut gpiob = dp.GPIOB.split(&mut rcc);
            gpiob.pb2.into_push_pull_output(&mut gpiob.crl).erase()
        }
    };

    let mut tmr = dp.TIM3.counter_hz(&mut rcc);

    #[cfg(feature = "rtt-log")]
    rtt_init_print!();

    tmr.start(1.Hz()).unwrap();

    loop {
        c += 1;
        wspr_log!("cycle {}", c);

        led.set_high();
        block!(tmr.wait()).ok();
        led.set_low();
        block!(tmr.wait()).ok();
    }
}
