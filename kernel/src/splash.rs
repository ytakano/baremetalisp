use super::aarch64;
use super::driver;
use super::print;

pub fn run() {
    print_el();
    print_fortune();
    print_splash();
}

/// print current execution level
fn print_el() {
    let el = aarch64::cpu::get_current_el();
    print::decimal("Current EL", el as u64);

    match aarch64::mmu::enabled() {
        Some(m) => {
            if m {
                print::msg("MMU", "Enabled");
            } else {
                print::msg("MMU", "Disabled");
            }
        }
        None => {
            driver::uart::puts("failed to access the system control register\n");
        }
    }
}

fn print_fortune() {
    let cnt = driver::delays::get_timer_value() as usize;
    let fortune = [
        "⛩  大吉 ⛩",
        "⛩  吉 ⛩",
        "⛩  吉 ⛩",
        "⛩  吉 ⛩",
        "⛩  吉 ⛩",
        "⛩  中吉 ⛩",
        "⛩  中吉 ⛩",
        "⛩  中吉 ⛩",
        "⛩  中吉 ⛩",
        "⛩  小吉 ⛩",
        "⛩  小吉 ⛩",
        "⛩  小吉 ⛩",
        "⛩  末吉 ⛩",
        "⛩  末吉 ⛩",
        "⛩  末吉 ⛩",
        "⛩  凶 ⛩",
    ];
    print::msg("Fortune", fortune[cnt & 0xF]);
}

/// print splash message
fn print_splash() {
    driver::uart::puts(
        " ___                                         _           _
(  _`\\                                      ( )_        (_ )  _
| (_) )   _ _  _ __   __    ___ ___     __  | ,_)   _ _  | | (_)  ___  _ _
|  _ <' /'_` )( '__)/'__`\\/' _ ` _ `\\ /'__`\\| |   /'_` ) | | | |/',__)( '_`\\
| (_) )( (_| || |  (  ___/| ( ) ( ) |(  ___/| |_ ( (_| | | | | |\\__, \\| (_) )
(____/'`\\__,_)(_)  `\\____)(_) (_) (_)`\\____)`\\__)`\\__,_)(___)(_)(____/| ,__/'
                                                                      | |
                                                                      (_)\n",
    );
}
