#![feature(type_alias_impl_trait)]
#![feature(exclusive_range_pattern)]
#![feature(proc_macro_hygiene)]
#![feature(int_roundings)]
#![feature(is_some_and)]
#![feature(stdsimd)]

#![no_main]
#![no_std]

#![allow(stable_features)]
#![allow(unused_mut)]


use panic_halt as _;
use rtic;
use rtic_monotonics::systick::*;
use rtic_sync::{channel::*, make_channel};

use stm32f4xx_hal::{
    gpio::*,
    prelude::*,
};

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [TIM4, TIM7, TIM8_CC])]
mod app {
    use super::*;
    mod my_module;
    pub mod logging;
    use my_module::blink2;

    #[shared]
    struct Shared {
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        led2: PC14<Output<PushPull>>,
        state: bool,
        state2: bool,
        sender: Sender<'static, u32, CAPACITY>,
    }

    const CAPACITY: usize = 5;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Setup clocks
        //let mut flash = cx.device.FLASH.constrain();
        let mut device = cx.device;
        let mut rcc = device.RCC.constrain();

        // Initialize the systick interrupt & obtain the token to prove that we did
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 120_000_000, systick_mono_token); // default STM32F407 clock-rate is 36MHz

        let _clocks = rcc
            .cfgr
            .use_hse(25.MHz())
            .sysclk(120.MHz())
            .require_pll48clk()
            .freeze();

        // Setup LED
        let mut gpioc = device.GPIOC.split();
        let mut led = gpioc
            .pc13
            .into_push_pull_output();

        let mut led2 = gpioc
            .pc14
            .into_push_pull_output();

        led.set_high();
        led2.set_high();

        // Schedule the blinking task
        blink::spawn().ok();
        blink2::spawn().ok();

        let (s, r) = make_channel!(u32, CAPACITY);

        receiver::spawn(r).unwrap();

        sender1::spawn(s.clone()).unwrap();
        sender2::spawn(s.clone()).unwrap();

        (Shared {}, Local { state: false, state2: false ,led,led2, sender: s.clone()})
    }

    #[task(local = [state,led])]
    async fn blink(mut cx: blink::Context) {
        loop {
            if *cx.local.state {
                cx.local.led.set_high();
                *cx.local.state = false;
            } else {
                cx.local.led.set_low();
                *cx.local.state = true;
            }
            Systick::delay(100.millis()).await;
        }
    }

    // Externally defined tasks
    extern "Rust" {
        #[task(local = [state2,led2])]
        async fn blink2(cx: blink2::Context);
    }

    #[task]
    async fn receiver(_c: receiver::Context, mut receiver: Receiver<'static, u32, CAPACITY>) {
        while let Ok(val) = receiver.recv().await {
            debug!("Receiver got: {}", val);
        }
    }

    #[task]
    async fn sender1(_c: sender1::Context, mut sender: Sender<'static, u32, CAPACITY>) {
        debug!("Sender 1 sending: 1");
        sender.send(1).await.unwrap();
    }

    #[task]
    async fn sender2(_c: sender2::Context, mut sender: Sender<'static, u32, CAPACITY>) {
        debug!("Sender 2 sending: 2");
        sender.send(2).await.unwrap();
    }

    #[task]
    async fn sender3(_c: sender3::Context, mut sender: Sender<'static, u32, CAPACITY>) {
        debug!("Sender 3 sending: 3");
        sender.send(3).await.unwrap();
    }
}
