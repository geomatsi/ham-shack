#![no_main]
#![no_std]

#[cfg(feature = "rtt-log")]
use panic_rtt_target as _;

#[cfg(feature = "rtt-log")]
use rtt_target::{rprint, rprintln, rtt_init_print};

#[cfg(not(feature = "rtt-log"))]
use panic_halt as _;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac::{self, USART3, interrupt},
    prelude::*,
    rcc,
    serial::Rx,
};

use core::cell::RefCell;
use cortex_m::interrupt::{Mutex, free as interrupt_free};
use wspr_beacon::{wspr_log, wspr_lognln};

// Neo-7M sends a groups of up to 12 msgs at once:
//
// $GPRMC,214533.00,V,,,,,,,020726,,,N*7E
// $GPVTG,,,,,,,,,N*30
// $GPGGA,214533.00,,,,,0,00,99.99,,,,,,*64
// $GPGSA,A,1,,,,,,,,,,,,,99.99,99.99,99.99*30
// $GPGSV,7,1,26,01,17,032,23,02,08,008,,03,,,23,05,,,22*79
// $GPGSV,7,2,26,06,-1,129,22,07,,,23,08,,,23,10,17,308,*6B
// $GPGSV,7,3,26,11,,,23,12,23,243,22,13,26,249,22,14,24,077,23*44
// $GPGSV,7,4,26,15,25,213,,16,,,23,17,49,079,,18,,,23*76
// $GPGSV,7,5,26,19,49,138,23,20,14,189,23,22,42,078,23,23,07,275,23*78
// $GPGSV,7,6,26,24,68,255,22,27,,,23,28,,,22,29,,,23*42
// $GPGSV,7,7,26,30,,,22,32,04,335,*4E
// $GPGLL,,,,,214533.00,V,N*48
//
// So pre-allocate enough space to keep all messages
const NMEA_LEN: usize = 2048;

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

    #[cfg(feature = "rtt-log")]
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
                        wspr_log!("NMEA buffer overflow");
                    }
                }
                rx.listen_idle();
            } else if rx.is_idle() {
                rx.unlisten_idle();
                let mut widx = WIDX.borrow(cs).borrow_mut();
                for i in 0..*widx {
                    let b = NMEA.borrow(cs).borrow_mut()[i];
                    wspr_lognln!("{}", b as char);
                }
                *widx = 0;
            }
        }
    })
}
