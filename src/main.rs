#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use esp_hal::{
    clock::ClockControl,
    gpio::{GpioPin, Output, IO},
    ledc::{
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
        LSGlobalClkSource, LowSpeed, LEDC,
    },
    peripherals::Peripherals,
    prelude::*,
    Delay,
};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let gpio4 = io.pins.gpio4.into_push_pull_output();

    let mut ledc = LEDC::new(peripherals.LEDC, &clocks);

    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.get_timer::<LowSpeed>(timer::Number::Timer0);

    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty5Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: 50u32.kHz(),
        })
        .unwrap();

    let mut channel4: esp_hal::ledc::channel::Channel<
        '_,
        LowSpeed,
        GpioPin<Output<esp_hal::gpio::PushPull>, 4>,
    > = ledc.get_channel(channel::Number::Channel4, gpio4);

    channel4
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 50,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();



    // Duty cycle values for ~500µs to ~2500µs pulse width (scaled to 8-bit range 0-255)
    let duty_min = (500.0 / 20000.0 * 255.0) as u8;  // 500µs pulse width
    let duty_max = (2500.0 / 20000.0 * 255.0) as u8; // 2500µs pulse width

    
    loop {
        // Pivot to "-90" degrees
        if let Err(e) = channel4.set_duty(duty_min) {
            println!("Error: {:?}", e);
            loop {}
        }
        delay.delay_ms(2000 as u32);

        // Pivot to "90" degrees
        if let Err(e) = channel4.set_duty(duty_max) {
            println!("Error: {:?}", e);
            loop {}
        }
        delay.delay_ms(2000 as u32);
    }
}
