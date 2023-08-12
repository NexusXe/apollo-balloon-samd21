#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm_experimental_arch)]
#![feature(nonzero_ops)]

use core::intrinsics::*;
use core::num::*;
//extern crate libm;
extern crate panic_halt;
use core::arch::asm;

use xiao_m0 as bsp;
use bsp::{hal, pac, entry};

use hal::{clock::GenericClockController, delay::Delay, prelude::*};
use pac::{CorePeripherals, Peripherals};
use apollo::{parameters, generate_packet};
use hal::pwm::Pwm3;

mod sensors;

#[macro_export]
macro_rules! transmit_packet {
    ($packet: expr, $txpin: expr, $ledpin: expr, $delay: expr) => {
        $txpin.set_duty(127);
        for _byte in $packet.iter() {
            for _bit in 0..8 {
                match _byte & (1 << _bit) {
                    0 => {
                        $txpin.disable();
                        let _ = $ledpin.set_low();
                        $delay.delay_ms(1000u16/parameters::BAUDRATE);
                        $txpin.disable();
                        let _ = $ledpin.set_low();
                    }
                    _ => {
                        $txpin.enable();
                        let _ = $ledpin.set_high();
                        $delay.delay_ms(1000u16/parameters::BAUDRATE);
                        $txpin.disable();
                        let _ = $ledpin.set_low();
                    }
                }
            }
        }
        //ufmt::uwriteln!(&mut $serial, "").void_unwrap();
    };
}

#[inline(never)]
fn no_operation(count: usize) {
    for _ in 0..count {
        unsafe { asm!("nop"); }
    }
}

#[entry]
fn main() -> ! {
    
    let mut dp = Peripherals::take().unwrap();
    let pins = bsp::Pins::new(dp.PORT);

    let mut status_pin = pins.a3.into_push_pull_output();
    status_pin.set_high().unwrap();

    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        dp.GCLK,
        &mut dp.PM,
        &mut dp.SYSCTRL,
        &mut dp.NVMCTRL,
    );

    let mut ledpin = pins.led0.into_push_pull_output();
    let gclk0 = clocks.gclk0();
    let mut txpin = Pwm3::new(
        &clocks.tcc2_tc3(&gclk0).unwrap(),
        1.khz(),
        dp.TC3,
        &mut dp.PM,
    );

    
    
    let mut delay = Delay::new(core.SYST, &mut clocks);

    delay.delay_ms(4000u16);
    // signal to the receiver that we are ready. pull down a3
    status_pin.set_low().unwrap();

    loop {
        let mut _packet = generate_packet(sensors::get_location, sensors::get_altitude, sensors::get_voltage, sensors::get_temperature);
        transmit_packet!(_packet, txpin, ledpin, delay);
    }
}
