#![no_main]
#![no_std]

use panic_rtt_target as _;

#[rtic::app(device = stm32f1xx_hal::pac)]
mod app {
    use cortex_m::singleton;
    use nmea0183;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f1xx_hal::{
        dma::CircBuffer,
        gpio::{Output, PushPull, gpioc::PC13},
        pac::{self, DMA1, USART3},
        prelude::*,
        serial,
        timer::{CounterMs, Event},
    };

    const UBLOX_LEN: usize = 2048;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        // LED task
        led: PC13<Output<PushPull>>,
        tim3: CounterMs<pac::TIM3>,
        // GPS task
        circ: Option<CircBuffer<[u8; UBLOX_LEN], serial::RxDma3>>,
        parser: nmea0183::Parser,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.freeze(
            stm32f1xx_hal::rcc::Config::hse(8.MHz())
                .sysclk(32.MHz())
                .pclk1(16.MHz()),
            &mut flash.acr,
        );

        rtt_init_print!();

        let mut afio = cx.device.AFIO.constrain(&mut rcc);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc);
        let channels = cx.device.DMA1.split(&mut rcc);

        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        let stx = gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh);
        let srx = gpiob.pb11;

        let mut tim3 = cx.device.TIM3.counter_ms(&mut rcc);
        tim3.start(200.millis()).unwrap();
        tim3.listen(Event::Update);

        let (_, mut rx) = cx
            .device
            .USART3
            .remap(&mut afio.mapr)
            .serial((stx, srx), 9600.bps(), &mut rcc)
            .split();

        // setup serial 'idle' interrupt before converting rx into rxdma
        rx.listen_idle();

        let nmea_parser = nmea0183::Parser::new()
            .sentence_filter(nmea0183::Sentence::RMC | nmea0183::Sentence::GGA);
        let dmabuf = singleton!(: [[u8; UBLOX_LEN]; 2] = [[0; UBLOX_LEN]; 2]).unwrap();
        let rxdma = rx.with_dma(channels.3);
        let circ = rxdma.circ_read(dmabuf);

        unsafe {
            cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART3);
        }

        (
            Shared {},
            Local {
                // LED task
                led,
                tim3,
                // GPS task
                circ: Some(circ),
                parser: nmea_parser,
            },
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            // Keep the core awake so host-side RTT attach does not time out.
            cortex_m::asm::nop();
        }
    }

    #[task(binds = TIM3, priority = 2, local = [led, tim3])]
    fn led(cx: led::Context) {
        rprintln!("TIM3 LED blink");
        cx.local.led.toggle();
        cx.local.tim3.clear_interrupt(Event::Update);
    }

    #[task(binds = USART3, priority = 1, local = [circ, parser])]
    fn gps(cx: gps::Context) {
        // Note: rx is 'moved' on rx.with_dma, so we can not use rx anymore.
        // IIUC there is no legitimate way to use rx.is_idle together with rxdma in current stm32f1xx HAL code.
        // For now just use direct unsafe access to USART3 and DMA1 regs to check interrupt status and transferred bytes.
        let usart3 = unsafe { &*USART3::ptr() };
        if usart3.sr().read().idle().bit_is_set() {
            // clear flag — read SR then DR sequence
            let _ = usart3.sr().read();
            let _ = usart3.dr().read();

            if let Some(circ) = cx.local.circ.take() {
                let (buf, rxdma) = circ.stop();
                let recv = (UBLOX_LEN * 2)
                    - unsafe { (*DMA1::ptr()).ch3().ndtr().read().ndt().bits() as usize };

                rprintln!("GPS received {} bytes", recv);

                // TODO: measure cycles required for NMEA parsing: probably we can not afford copying and parsing during WSPR Tx process

                for result in cx.local.parser.parse_from_bytes(&buf[0][..]) {
                    match result {
                        Ok(nmea0183::ParseResult::RMC(Some(rmc))) => {
                            rprintln!(
                                "GPRMC: mode {:?} date {:?} Lat {} Lon {}",
                                rmc.mode,
                                rmc.datetime,
                                rmc.latitude.degrees,
                                rmc.longitude.degrees
                            );
                        }
                        Ok(nmea0183::ParseResult::RMC(None)) => {
                            rprintln!("GPRMC: no fix...");
                        }
                        Ok(nmea0183::ParseResult::GGA(Some(gga))) => {
                            rprintln!(
                                "GPGGA: Quality {:?} satellites {}",
                                gga.gps_quality,
                                gga.sat_in_use
                            );
                        }
                        Ok(nmea0183::ParseResult::GGA(None)) => {
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

                buf[0].fill(0);

                *cx.local.circ = Some(rxdma.circ_read(buf));
            }
        }
    }
}
