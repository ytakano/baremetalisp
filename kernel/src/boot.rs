use super::driver;
use super::aarch64;

pub fn run() {
    print_firmware_version();
    print_board_info();
    print_el();
    print_fortune();
    print_splash();
}

/// print current execution level
fn print_el() {
    driver::uart::puts("[Current EL  ] EL");
    let el = aarch64::el::get_current_el();
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

fn print_firmware_version() {
    /*
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
    */
}

fn print_board_info() {
    /*
    match driver::mbox::get_board_rev() {
        Some(rev) => {
            driver::uart::puts("[Board Rev   ] ");
            print_revision(rev);
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
    */
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
                                                                      (_)\n");
}

fn print_revision(rev: u32) {
    // https://www.raspberrypi.org/documentation/hardware/raspberrypi/revision-codes/README.md
    // layout (bits)
    // uuuuuuuu FMMMCCCC PPPPTTTT TTTTRRRR

    // F: old or new
    if (rev >> 23) & 1 == 0 {
        // old style
        driver::uart::puts("0x");
        driver::uart::hex32(rev);
    } else {
        // new stype

        // MMM: memory size
        match (rev >> 20) & 0b111 {
            0 => { driver::uart::puts("256MB Mem, "); }
            1 => { driver::uart::puts("512MB Mem, "); }
            2 => { driver::uart::puts("1GB Mem, "); }
            3 => { driver::uart::puts("2GB Mem, "); }
            4 => { driver::uart::puts("4GB Mem, "); }
            _ => { driver::uart::puts("unknown Mem, "); }
        }

        // CCCC: manufacturer
        match (rev >> 16) & 0b1111 {
            0 => { driver::uart::puts("Sony UK, "); }
            1 => { driver::uart::puts("Egoman, "); }
            2 => { driver::uart::puts("Embest, "); }
            3 => { driver::uart::puts("Sony Japan, "); }
            4 => { driver::uart::puts("Embest, "); }
            5 => { driver::uart::puts("Stadium, "); }
            _ => { driver::uart::puts("unknown, "); }
        }

        // PPPP: processor
        match (rev >> 12) & 0b1111 {
            0 => { driver::uart::puts("BCM 2835, "); }
            1 => { driver::uart::puts("BCM 2836, "); }
            2 => { driver::uart::puts("BCM 2837, "); }
            3 => { driver::uart::puts("BCM 2711, "); }
            _ => { driver::uart::puts("unknown, "); }
        }

        // TTTTTTTT: type
        match (rev >> 4) & 0b11111111 {
             0 => { driver::uart::puts("Model A, "); }
             1 => { driver::uart::puts("Model B, "); }
             2 => { driver::uart::puts("Model A+, "); }
             3 => { driver::uart::puts("Model B+, "); }
             4 => { driver::uart::puts("Model 2B, "); }
             5 => { driver::uart::puts("Model Alpha (early prototype), "); }
             6 => { driver::uart::puts("Model CM1, "); }
             8 => { driver::uart::puts("Model 3B, "); }
             9 => { driver::uart::puts("Model Zero, "); }
            10 => { driver::uart::puts("Model CM3, "); }
            12 => { driver::uart::puts("Model Zero W, "); }
            13 => { driver::uart::puts("Model 3B+, "); }
            16 => { driver::uart::puts("Model CM3+, "); }
            17 => { driver::uart::puts("Model 4B, "); }
             _ => { driver::uart::puts("unknown, "); }
        }

        // RRRR: Revision
        driver::uart::puts("Rev. ");
        driver::uart::decimal((rev & 0b1111) as u64);
    }
}