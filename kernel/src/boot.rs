use super::aarch64;
use super::driver;

pub fn run() {
    print_el();
    print_fortune();
    print_splash();
}

/// print current execution level
fn print_el() {
    driver::uart::puts("[Current EL  ] EL");
    let el = aarch64::cpu::get_current_el();
    driver::uart::decimal(el as u64);
    driver::uart::puts("\n");

    driver::uart::puts("[MMU         ] ");
    match aarch64::mmu::enabled() {
        Some(m) => {
            if m {
                driver::uart::puts("true\n");
            } else {
                driver::uart::puts("false\n");
            }
        }
        None => {
            driver::uart::puts("failed to access the system control register\n");
        }
    }
}

fn print_fortune() {
    /*
    driver::uart::puts("[Fortune     ] ");
    let cnt = driver::delays::get_system_timer() as usize;
    let fortune = ["大吉", "吉", "吉", "吉", "吉", "中吉", "中吉", "中吉",
                   "中吉", "小吉", "小吉", "小吉", "末吉", "末吉", "末吉", "凶"];
    driver::uart::puts("⛩  ");
    driver::uart::puts(fortune[cnt & 0xF]);
    driver::uart::puts(" ⛩\n");
    */
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
