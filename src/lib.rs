#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(start)]
#![feature(asm)]
#![no_std]

mod parser;
mod aarch64;

use core::panic::PanicInfo;

#[no_mangle]
fn func() {
    ()
}

fn print_splash() {
    aarch64::uart::puts(
" ___                                         _           _
(  _`\\                                      ( )_        (_ )  _
| (_) )   _ _  _ __   __    ___ ___     __  | ,_)   _ _  | | (_)  ___  _ _
|  _ <' /'_` )( '__)/'__`\\/' _ ` _ `\\ /'__`\\| |   /'_` ) | | | |/',__)( '_`\\
| (_) )( (_| || |  (  ___/| ( ) ( ) |(  ___/| |_ ( (_| | | | | |\\__, \\| (_) )
(____/'`\\__,_)(_)  `\\____)(_) (_) (_)`\\____)`\\__)`\\__,_)(___)(_)(____/| ,__/'
                                                                      | |
                                                                      (_)\n");

    let cnt = aarch64::delays::get_system_timer() as usize;
    let fortune = ["大吉", "吉", "吉", "吉", "吉", "中吉", "中吉", "中吉",
                   "中吉", "小吉", "小吉", "小吉", "末吉", "末吉", "末吉", "凶"];
    aarch64::uart::puts("⛩ ⛩ ⛩  ");
    aarch64::uart::puts(fortune[cnt & 0xF]);
    aarch64::uart::puts(" ⛩ ⛩ ⛩\n");
}

#[no_mangle]
pub fn entry() -> ! {
    unsafe {
        asm!(
            "bl func;"
            : : : :
        );
    }

    let ctx = aarch64::init();
    print_splash();

    match aarch64::mbox::get_serial() {
        Some(serial) => {
                aarch64::uart::puts("serial#: ");
                aarch64::uart::hex(serial);
                aarch64::uart::puts("\n");
            }
        None => { aarch64::uart::puts("failed to get serial#\n") }
    }

    match ctx.graphics0 {
        Some(mut gr) => {
            aarch64::uart::puts("drawing mandelbrot set...\n");
            let mut cnt = aarch64::delays::get_system_timer();
            gr.plot_mandelbrot_set();
            cnt = aarch64::delays::get_system_timer() - cnt;
            aarch64::uart::puts("elapsed time to draw: ");
            aarch64::uart::decimal(cnt);
            aarch64::uart::puts(" CPU clocks\n");
        }
        None => { aarch64::uart::puts("failed to initialize graphics\n") }
    }

//    aarch64::uart::puts("halting...\n");
//    aarch64::power::shutdown();

//    aarch64::uart::puts("reseting...\n");
//    aarch64::power::reset();

    parser::test(10);

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
extern fn eh_personality() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    aarch64::uart::puts("kernel panic!");
    loop {}
}

#[no_mangle]
pub fn abort() -> ! {
    loop {}
}