#![no_std]
#![no_main]
#![feature(asm)]

use embedded_hal::{digital::v2::OutputPin, prelude::_embedded_hal_timer_CountDown};

use core::fmt::Write;
use riscv_rt::entry;

use esp32c3_lib::{disable_wdts, EtsTimer, GpioOutput, Uart};

// make sure we have something in our data section
#[used]
static DATA_SECTION_TEST: &'static str = "TEST DATA";
// make sure we have something in our bss section
#[used]
static mut BSS_SECTION_TEST: [u8; 12] = [0xAA; 12];

#[entry]
fn main() -> ! {
    // disable interrupts, `csrwi        mie,0` throws an exception on the esp32c3
    unsafe {
        let mut _tmp: u32;
        asm!("csrrsi {0}, mstatus, {1}", out(reg) _tmp, const 0x00000008)
    };

    // disable wdt's
    disable_wdts();

    let mut gpio = GpioOutput::new(9);

    writeln!(Uart, "Hello world!").unwrap();
    writeln!(Uart, "{}", DATA_SECTION_TEST).unwrap();

    let mut delay = EtsTimer::new(1_000_000);

    loop {
        writeln!(Uart, "HIGH").unwrap();
        gpio.set_high().unwrap();
        nb::block!(delay.wait()).unwrap();

        writeln!(Uart, "LOW").unwrap();
        gpio.set_low().unwrap();
        nb::block!(delay.wait()).unwrap();

        bar(0);
    }
}

fn bar(v: i32) {
    foo(v);
}

fn foo(y: i32) {
    let _y = 5 / y;
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    writeln!(Uart, "{}", info).ok();
    print_backtrace_addresses();
    loop {}
}

fn print_backtrace_addresses() {
    let mut fp = unsafe {
        let mut _tmp: u32;
        asm!("mv {0}, x8", out(reg) _tmp);
        _tmp
    };

    let mut suppress = 2;
    loop {
        unsafe {
            let address = (fp as *const u32).offset(-1).read(); // RA/PC
            fp = (fp as *const u32).offset(-2).read(); // next FP

            // currently this only supports code in flash
            if !(0x42000000..=0x42800000).contains(&address) {
                break;
            }

            if suppress == 0 {
                write!(Uart, "0x{:x} \r\n", address).ok();
            } else {
                suppress -= 1;
            }
        }
    }
}
