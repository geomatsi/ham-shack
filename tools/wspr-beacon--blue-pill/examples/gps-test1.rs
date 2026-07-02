#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprint, rtt_init_print};
use stm32f1xx_hal::{pac, prelude::*, rcc, serial::Config};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(32.MHz()).pclk1(16.MHz()),
        &mut flash.acr,
    );

    rtt_init_print!();

    let mut afio = dp.AFIO.constrain(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);

    let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    let rx = gpiob.pb11;

    let mut serial = dp.USART3.remap(&mut afio.mapr).serial(
        (tx, rx),
        Config::default().baudrate(9600.bps()),
        &mut rcc,
    );

    loop {
        if let Ok(ch) = nb::block!(serial.rx.read()) {
            rprint!("{}", ch as char);
        }
    }
}
