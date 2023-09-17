use rtic_monotonics::systick::*;
use crate::app;

pub(crate) async fn blink2(mut cx: app::blink2::Context<'_>) {
    loop {
        if *cx.local.state2 {
            cx.local.led2.set_high();
            *cx.local.state2 = false;
        } else {
            cx.local.led2.set_low();
            *cx.local.state2 = true;
        }
        Systick::delay(50.millis()).await;
    }
}