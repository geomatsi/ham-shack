#![no_main]
#![no_std]

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac::{self, USART3, interrupt},
    prelude::*,
    rcc,
    serial::Rx,
};

use panic_rtt_target as _;
use rtt_target::{rprint, rprintln, rtt_init_print};

use core::cell::RefCell;
use cortex_m::interrupt::{Mutex, free as interrupt_free};

const NMEA_LEN: usize = 82 * 9; // we receive groups of 8 msgs
static WIDX: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(0));
static NMEA: Mutex<RefCell<[u8; NMEA_LEN]>> = Mutex::new(RefCell::new([0; NMEA_LEN]));

static RX: Mutex<RefCell<Option<Rx<USART3>>>> = Mutex::new(RefCell::new(None));

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

    let tx = gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh);
    let rx = gpiob.pb11;

    let (_, mut rx) = dp
        .USART3
        .remap(&mut afio.mapr)
        .serial((tx, rx), 9600.bps(), &mut rcc)
        .split();

    rx.listen();
    rx.listen_idle();

    interrupt_free(|cs| {
        RX.borrow(cs).replace(Some(rx));
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART3);
    }

    loop {
        cortex_m::asm::nop()
    }
}

#[interrupt]
fn USART3() {
    interrupt_free(|cs| {
        let mut rx = RX.borrow(cs).borrow_mut();
        if let Some(rx) = rx.as_mut() {
            if rx.is_rx_not_empty() {
                if let Ok(w) = nb::block!(rx.read()) {
                    let mut widx = WIDX.borrow(cs).borrow_mut();

                    if *widx < NMEA_LEN - 1 {
                        NMEA.borrow(cs).borrow_mut()[*widx] = w;
                        *widx += 1;
                    } else {
                        rprintln!("NMEA buffer overflow");
                    }
                }
                rx.listen_idle();
            } else if rx.is_idle() {
                rx.unlisten_idle();
                let mut widx = WIDX.borrow(cs).borrow_mut();
                for i in 0..*widx {
                    let b = NMEA.borrow(cs).borrow_mut()[i];
                    rprint!("{}", b as char);
                }
                *widx = 0;
            }
        }
    })
}
