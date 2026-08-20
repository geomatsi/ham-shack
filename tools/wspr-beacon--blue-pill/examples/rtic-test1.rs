#![no_main]
#![no_std]

#[cfg(feature = "rtt-log")]
use panic_rtt_target as _;

#[cfg(not(feature = "rtt-log"))]
use panic_halt as _;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [SPI1, SPI2])]
mod app {
    use core::sync::atomic::{AtomicBool, Ordering};
    use rtic_monotonics::stm32::prelude::*;
    use wspr_beacon::wspr_log;

    use stm32f1xx_hal::{
        gpio::{self, Output, PushPull},
        pac,
        prelude::*,
        timer,
    };

    #[cfg(feature = "rtt-log")]
    use rtt_target::{rprintln, rtt_init_print};

    const SYSCLK_MHZ: u32 = 32;

    static LED_GATE1: AtomicBool = AtomicBool::new(false);
    static LED_GATE2: AtomicBool = AtomicBool::new(false);

    stm32_tim4_monotonic!(Mono, 100_000);

    #[shared]
    struct Shared {
        s: u64,
    }

    #[local]
    struct Local {
        // LED task
        led: gpio::gpioc::PC13<Output<PushPull>>,
        tim: timer::CounterMs<pac::TIM2>,

        // IDLE task
        delay: timer::DelayUs<pac::TIM1>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.freeze(
            stm32f1xx_hal::rcc::Config::hse(8.MHz())
                .sysclk(SYSCLK_MHZ.MHz())
                .pclk1(16.MHz()),
            &mut flash.acr,
        );

        #[cfg(feature = "dwt-profile")]
        {
            cx.core.DCB.enable_trace();
            cx.core.DWT.enable_cycle_counter();
        }

        #[cfg(feature = "rtt-log")]
        rtt_init_print!();

        let mut gpioc = cx.device.GPIOC.split(&mut rcc);

        //// IDLE task

        let delay = cx.device.TIM1.delay_us(&mut rcc);

        //// LED task

        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        let mut tim = cx.device.TIM2.counter_ms(&mut rcc);
        tim.start(1000u32.millis()).unwrap();
        tim.listen(timer::Event::Update);

        //// shared globals

        let s: u64 = 0;

        ////

        Mono::start(rcc.clocks.pclk1_tim().to_Hz());

        (
            Shared { s },
            Local {
                // LED task
                led,
                tim,

                // IDLE task
                delay,
            },
        )
    }

    #[idle(shared = [s], local = [delay])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            wspr_log!("SCHED: heartbeat");

            match task1::spawn() {
                Ok(_) => {
                    wspr_log!("SCHED: spawned task1");
                }
                Err(_) => {
                    wspr_log!("SCHED: failed to spawn task1");
                }
            }

            match task2::spawn() {
                Ok(_) => {
                    wspr_log!("SCHED: spawned task2");
                }
                Err(_) => {
                    wspr_log!("SCHED: failed to spawn task2");
                }
            }

            cx.local.delay.delay_ms(1000u32);

            #[cfg(feature = "rtt-log")]
            cortex_m::asm::nop();

            #[cfg(not(feature = "rtt-log"))]
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = TIM2, priority = 10, local = [led, tim], shared = [s])]
    fn led(cx: led::Context) {
        wspr_log!("LED: started");

        LED_GATE1.store(false, Ordering::Release);
        LED_GATE2.store(false, Ordering::Release);

        cx.local.tim.clear_interrupt(timer::Event::Update);
        cx.local.led.toggle();

        #[cfg(feature = "dwt-profile")]
        wspr_log!(
            "LED: DWT {} ms",
            cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000
        );
    }

    #[task(priority = 5, shared = [s])]
    async fn task1(_cx: task1::Context) {
        wspr_log!("TASK1: started");

        for i in 0..5 {
            //  wait for the tick
            LED_GATE1.store(true, Ordering::Relaxed);
            while LED_GATE1.load(Ordering::Acquire) {
                //cortex_m::asm::nop();
                Mono::delay(500u64.micros()).await;
            }

            wspr_log!("TASK1: LED gate {}", i);

            Mono::delay(1000u64.millis()).await;
        }
    }

    #[task(priority = 15, shared = [s])]
    async fn task2(_cx: task2::Context) {
        wspr_log!("TASK2: started");

        for i in 0..5 {
            //  wait for the tick
            LED_GATE2.store(true, Ordering::Relaxed);
            while LED_GATE2.load(Ordering::Acquire) {
                //cortex_m::asm::nop();
                Mono::delay(500u64.micros()).await;
            }

            wspr_log!("TASK2: LED gate {}", i);

            Mono::delay(1000u64.millis()).await;
        }
    }
}
