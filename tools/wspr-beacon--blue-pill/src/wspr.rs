#![no_main]
#![no_std]

use panic_rtt_target as _;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [SPI1])]
mod app {
    use cortex_m::singleton;
    use nmea0183;
    use rtic_monotonics::stm32::prelude::*;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f1xx_hal::{
        dma::CircBuffer,
        gpio::{Edge, ExtiPin, Input, Output, PushPull, gpiob::PB1, gpioc::PC13},
        pac::{self, DMA1, USART3},
        prelude::*,
        serial,
        timer::{CounterMs, Event},
    };
    use wspr_encoder;

    const SYSCLK_MHZ: u32 = 32;
    const UBLOX_LEN: usize = 2048;

    stm32_tim4_monotonic!(Mono, 1_000_000);

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        // LED task
        led: PC13<Output<PushPull>>,
        tim3: CounterMs<pac::TIM3>,
        tick1: u32,
        // GPS task
        circ: Option<CircBuffer<[u8; UBLOX_LEN], serial::RxDma3>>,
        parser: nmea0183::Parser,
        tick2: u32,
        // PPS task
        pps: PB1<Input>,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local) {
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.freeze(
            stm32f1xx_hal::rcc::Config::hse(8.MHz())
                .sysclk(SYSCLK_MHZ.MHz())
                .pclk1(16.MHz()),
            &mut flash.acr,
        );
        let mut cp = cx.core;

        rcc.clocks.sysclk().to_MHz();

        cp.DCB.enable_trace();
        cp.DWT.enable_cycle_counter();

        rtt_init_print!();

        let mut afio = cx.device.AFIO.constrain(&mut rcc);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc);
        let channels = cx.device.DMA1.split(&mut rcc);

        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        //// LED
        let mut tim3 = cx.device.TIM3.counter_ms(&mut rcc);
        let tick1 = cp.DWT.cyccnt.read();
        tim3.start(5000u32.millis()).unwrap();
        tim3.listen(Event::Update);

        //// PPS
        let mut pps = gpiob.pb1.into_floating_input(&mut gpiob.crl);
        pps.make_interrupt_source(&mut afio);
        pps.trigger_on_edge(&mut cx.device.EXTI, Edge::Rising);
        pps.enable_interrupt(&mut cx.device.EXTI);

        //// GPS
        let stx = gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh);
        let srx = gpiob.pb11;
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

        let tick2 = cp.DWT.cyccnt.read();

        //// Interrupts

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::USART3);
            pac::NVIC::unmask(pac::Interrupt::EXTI1);
        }

        Mono::start(SYSCLK_MHZ * 1_000_000);

        (
            Shared {},
            Local {
                // LED task
                led,
                tim3,
                tick1,
                // GPS task
                circ: Some(circ),
                parser: nmea_parser,
                tick2,
                // PPS task
                pps,
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

    #[task(priority = 10)]
    async fn wspr(_cx: wspr::Context, x: i32) {
        rprintln!("WSPR started: {}", x);

        match wspr_encoder::encode("R1BRL", "KP50", 37) {
            Ok(symbols) => {
                // 20m WSPR dial frequency in KHz
                let dial = 14095.6;

                // WSPR transmit frequencies are 1.5KHz above the dial frequency
                let offset = 1.5;

                for symbol in symbols.iter() {
                    let _frequency = dial + offset + (0.001464 * (*symbol as f64));
                    // TODO
                    // set_frequency(frequency);
                    // enable_tx();
                    rprintln!("WSPR sending: {}", symbol);
                    Mono::delay(10_u64.millis() /* 683_u64.millis() */).await;
                    // disable_tx();
                }
            },
            Err(e) => {
                rprintln!("WSPR: fatal encoding failure: {:?}", e);
            }
        }
    }

    #[task(binds = EXTI1, priority = 10, local = [pps])]
    fn pps(cx: pps::Context) {
        if cx.local.pps.check_interrupt() {
            cx.local.pps.clear_interrupt_pending_bit();
            match wspr::spawn(42) {
                Ok(_) => { rprintln!("PPS: spawned WSPR"); },
                Err(_) => { rprintln!("PPS: WSPR is already running")}
            }
        }
    }

    #[task(binds = TIM3, priority = 1, local = [led, tim3, tick1])]
    fn led(cx: led::Context) {
        let current = cortex_m::peripheral::DWT::cycle_count();
        rprintln!(
            "LED: delta {} ms",
            current.wrapping_sub(*cx.local.tick1) / (SYSCLK_MHZ * 1000)
        );
        *cx.local.tick1 = current;

        cx.local.led.toggle();
        cx.local.tim3.clear_interrupt(Event::Update);
    }

    #[task(binds = USART3, priority = 5, local = [circ, parser, tick2])]
    fn gps(cx: gps::Context) {
        let current = cortex_m::peripheral::DWT::cycle_count();
        rprintln!(
            "GPS delta {} ms",
            current.wrapping_sub(*cx.local.tick2) / (SYSCLK_MHZ * 1000)
        );
        *cx.local.tick2 = current;

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

                let start = cortex_m::peripheral::DWT::cycle_count();

                for result in cx.local.parser.parse_from_bytes(&buf[0][..]) {
                    match result {
                        Ok(nmea0183::ParseResult::RMC(Some(rmc))) => {
                            //rprintln!(
                            //    "GPRMC: mode {:?} date {:?} Lat {} Lon {}",
                            //    rmc.mode,
                            //    rmc.datetime,
                            //    rmc.latitude.degrees,
                            //    rmc.longitude.degrees
                            //);
                        }
                        Ok(nmea0183::ParseResult::RMC(None)) => {
                            rprintln!("GPRMC: no fix...");
                        }
                        Ok(nmea0183::ParseResult::GGA(Some(gga))) => {
                            //rprintln!(
                            //    "GPGGA: Quality {:?} satellites {}",
                            //    gga.gps_quality,
                            //    gga.sat_in_use
                            //);
                        }
                        Ok(nmea0183::ParseResult::GGA(None)) => {
                            //rprintln!("GPGGA: no fix...");
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

                let elapsed = cortex_m::peripheral::DWT::cycle_count().wrapping_sub(start);

                rprintln!("NMEA processing: {} us", elapsed / SYSCLK_MHZ);

                *cx.local.circ = Some(rxdma.circ_read(buf));
            }
        }
    }
}
