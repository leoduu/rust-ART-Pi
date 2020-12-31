//! Blinks an LED
//!
//! This assumes that LD2 (blue) is connected to pb7 and LD3 (red) is connected
//! to pb14. This assumption is true for the nucleo-h743zi board.

#![no_std]
#![no_main]

use panic_halt as _;

use nb::block;

use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::timer::Timer;

use cortex_m_rt::entry;

use embedded_hal::digital::v2::OutputPin;

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = stm32h7xx_hal::stm32::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIOB peripheral
    let mut gpioi = dp.GPIOI.split(&mut rcc.ahb4);

    // Configure gpio B pin 7 as a push-pull output.
    let mut led = gpioi.pi8.into_push_pull_output(&mut gpioi.moder, &mut gpioi.otyper);

    // Configure the timer to trigger an update every second
    let mut timer = Timer::tim1(dp.TIM1, 1.hz(), clocks, &mut rcc.apb2);

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        block!(timer.wait()).unwrap();
        led.set_high().unwrap();

        block!(timer.wait()).unwrap();
        led.set_low().unwrap();
    }

}