#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal::tsens::TemperatureSensor;
use esp_hal::{main, tsens};
use esp_hal_smartled::{RmtSmartLeds, Ws2812Timing, buffer_size, color_order};
use esp_println::println;
use smart_leds::hsv::{Hsv, hsv2rgb};
use smart_leds::{RGB8, SmartLedsWrite, brightness, gamma};

esp_bootloader_esp_idf::esp_app_desc!();


#[esp_rtos::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();
    type LedColor = RGB8;
    let mut led = RmtSmartLeds::<
        { buffer_size::<LedColor>(1) },
        _,
        LedColor,
        color_order::Rgb,
        Ws2812Timing,
    >::new_with_memsize(rmt.channel0, peripherals.GPIO10, 2)
    .unwrap();

    let button = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(Pull::Up),
    );

    let delay = Delay::new();

    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };
    let mut color_data;

    let temperature_sensor =
        TemperatureSensor::new(peripherals.TSENS, tsens::Config::default()).unwrap();

    loop {
        if button.is_low() {
            color.hue = color.hue.wrapping_add(1);
            color_data = [hsv2rgb(color)];
            led.write(brightness(gamma(color_data.iter().cloned()), 10))
                .unwrap();
        }

        println!(
            "temp: {:?}",
            temperature_sensor.get_temperature().to_celsius()
        );
        delay.delay_millis(100);
    }
}

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    println!("PANICKED: {panic_info}");
    loop {}
}
