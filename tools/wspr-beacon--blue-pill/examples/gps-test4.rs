#![no_main]
#![no_std]

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    dma::CircBuffer,
    pac::{self, DMA1, USART3, interrupt},
    prelude::*,
    rcc, serial,
};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::interrupt::{Mutex, free as interrupt_free};
use cortex_m::singleton;

use nmea0183::{ParseResult, Parser, Sentence};

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
const UBLOX_LEN: usize = 2048;

static NMEA_BUF: Mutex<RefCell<[u8; UBLOX_LEN]>> = Mutex::new(RefCell::new([0; UBLOX_LEN]));
static CB: Mutex<RefCell<Option<CircBuffer<[u8; UBLOX_LEN], serial::RxDma3>>>> =
    Mutex::new(RefCell::new(None));
static NMEA_RDY: AtomicBool = AtomicBool::new(false);

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
    let channels = dp.DMA1.split(&mut rcc);

    let tx = gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh);
    let rx = gpiob.pb11;

    let (_, mut rx) = dp
        .USART3
        .remap(&mut afio.mapr)
        .serial((tx, rx), 9600.bps(), &mut rcc)
        .split();

    // setup serial 'idle' interrupt before converting rx into rxdma
    rx.listen_idle();

    let dmabuf = singleton!(: [[u8; UBLOX_LEN]; 2] = [[0; UBLOX_LEN]; 2]).unwrap();
    let rxdma = rx.with_dma(channels.3);

    interrupt_free(|cs| {
        let circ = rxdma.circ_read(dmabuf);
        CB.borrow(cs).replace(Some(circ));
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART3);
    }

    let mut parser = Parser::new().sentence_filter(Sentence::RMC | Sentence::GGA);
    let mut nmeabuf: [u8; UBLOX_LEN] = [0; UBLOX_LEN];

    loop {
        if NMEA_RDY.load(Ordering::Acquire) {
            interrupt_free(|cs| {
                nmeabuf.copy_from_slice(&*NMEA_BUF.borrow(cs).borrow());
            });
            NMEA_RDY.store(false, Ordering::Relaxed);

            for result in parser.parse_from_bytes(&nmeabuf[..]) {
                match result {
                    Ok(ParseResult::RMC(Some(rmc))) => {
                        rprintln!(
                            "GPRMC: mode {:?} date {:?} Lat {} Lon {}",
                            rmc.mode,
                            rmc.datetime,
                            rmc.latitude.degrees,
                            rmc.longitude.degrees
                        );
                    }
                    Ok(ParseResult::RMC(None)) => {
                        rprintln!("GPRMC: no fix...");
                    }
                    Ok(ParseResult::GGA(Some(gga))) => {
                        rprintln!(
                            "GPGGA: Quality {:?} satellites {}",
                            gga.gps_quality,
                            gga.sat_in_use
                        );
                    }
                    Ok(ParseResult::GGA(None)) => {
                        rprintln!("GPGGA: no fix...");
                    }
                    Ok(_) => {
                        // skip other messages for now
                    }
                    Err(e) => {
                        rprintln!("Error parsing NMEA: {}", e);
                    }
                }
            }
        }
    }
}

#[interrupt]
fn USART3() {
    interrupt_free(|cs| {
        // Note: rx is 'moved' on rx.with_dma, so we can not use rx anymore.
        // IIUC there is no legitimate way to use rx.is_idle together with rxdma in current stm32f1xx HAL code.
        // For now just use direct unsafe access to USART3 and DMA1 regs to check interrupt status and transferred bytes.
        let usart3 = unsafe { &*USART3::ptr() };
        if usart3.sr().read().idle().bit_is_set() {
            // clear flag — read SR then DR sequence
            let _ = usart3.sr().read();
            let _ = usart3.dr().read();

            if let Some(circ) = CB.borrow(cs).take() {
                let (buf, rxdma) = circ.stop();
                let recv = (UBLOX_LEN * 2)
                    - unsafe { (*DMA1::ptr()).ch3().ndtr().read().ndt().bits() as usize };

                // skip if main loop is stuck on making its own copy for some reason
                if !NMEA_RDY.load(Ordering::Acquire) {
                    NMEA_BUF.borrow(cs).borrow_mut()[0..recv].copy_from_slice(&buf[0][0..recv]);
                    NMEA_RDY.store(true, Ordering::Release);
                }

                buf[0].fill(0);

                let circ = rxdma.circ_read(buf);
                CB.borrow(cs).replace(Some(circ));
            }
        }
    })
}
