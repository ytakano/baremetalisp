use super::driver;
use super::aarch64;

pub fn run() {
    print_firmware_version();
    print_board_info();
    print_el();
    print_splash();
}

/// print current execution level
fn print_el() {
    driver::uart::puts("[Current EL  ] EL");
    let el = aarch64::el::get_current_el();
    driver::uart::decimal(el as u64);
    driver::uart::puts("\n");
}

fn print_firmware_version() {
    match driver::mbox::get_firmware_version() {
        Some(ver) => {
            driver::uart::puts("[Firmware Ver] ");
            driver::uart::decimal(ver as u64);
            driver::uart::puts("\n");
        }
        None => {
            driver::uart::puts("[Firmware Ver] failed to get\n")
        }
    }
}

fn print_board_info() {
    match driver::mbox::get_board_model() {
        Some(ver) => {
            driver::uart::puts("[Board Model ] ");
            driver::uart::decimal(ver as u64);
            driver::uart::puts("\n");
        }
        None => {
            driver::uart::puts("[Board Model ] failed to get\n")
        }
    }

    match driver::mbox::get_board_rev() {
        Some(ver) => {
            driver::uart::puts("[Board Rev   ] ");
            driver::uart::decimal(ver as u64);
            driver::uart::puts("\n");
        }
        None => {
            driver::uart::puts("[Board Rev   ] failed to get\n")
        }
    }

    match driver::mbox::get_serial() {
        Some(serial) => {
            driver::uart::puts("[Serial#     ] 0x");
            driver::uart::hex(serial);
            driver::uart::puts("\n");
        }
        None => {
            driver::uart::puts("[Serial#     ] failed to get\n")
        }
    }

    match driver::mbox::get_memory() {
        Some(mem) => {
            driver::uart::puts("[Memory      ] ");
            driver::uart::decimal(mem.1 as u64);
            driver::uart::puts(" (base: ");
            driver::uart::decimal(mem.0 as u64);
            driver::uart::puts(")\n");
        }
        None => {
            driver::uart::puts("[Memory      ] failed to get\n")
        }
    }
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
                                                                      (_)\n");

    let cnt = driver::delays::get_system_timer() as usize;
    let fortune = ["大吉", "吉", "吉", "吉", "吉", "中吉", "中吉", "中吉",
                   "中吉", "小吉", "小吉", "小吉", "末吉", "末吉", "末吉", "凶"];
    driver::uart::puts("⛩ ⛩ ⛩  ");
    driver::uart::puts(fortune[cnt & 0xF]);
    driver::uart::puts(" ⛩ ⛩ ⛩\n");
}
