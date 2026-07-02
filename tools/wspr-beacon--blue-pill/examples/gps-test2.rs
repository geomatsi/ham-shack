#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use core::cell::RefCell;
use cortex_m::interrupt::{Mutex, free as interrupt_free};
use cortex_m_rt::entry;
use pac::interrupt;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::gpio::*;
use stm32f1xx_hal::{pac, prelude::*, rcc};

static PPS: Mutex<RefCell<Option<stm32f1xx_hal::gpio::gpiob::PB1<Input>>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(32.MHz()).pclk1(16.MHz()),
        &mut flash.acr,
    );

    let mut afio = dp.AFIO.constrain(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);

    rtt_init_print!();

    interrupt_free(|cs| {
        let mut pps = gpiob.pb1.into_floating_input(&mut gpiob.crl);
        pps.make_interrupt_source(&mut afio);
        pps.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
        pps.enable_interrupt(&mut dp.EXTI);
        PPS.borrow(cs).replace(Some(pps));
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::EXTI1);
    }

    rprintln!("Ready to go...");

    loop {
        cortex_m::asm::nop()
    }
}

#[interrupt]
fn EXTI1() {
    interrupt_free(|cs| {
        let mut pps = PPS.borrow(cs).borrow_mut();
        if let Some(pps) = pps.as_mut()
            && pps.check_interrupt()
        {
            rprintln!("PPS interrupt");
            pps.clear_interrupt_pending_bit();
        }
    })
}
