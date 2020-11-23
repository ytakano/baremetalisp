fn board_info() -> String {
    let s = "".to_string();

    match driver::mbox::get_board_rev() {
        Some(rev) => {
            driver::uart::puts("[Board Rev   ] ");
            print_revision(rev);
            driver::uart::puts("\n");
        }
        None => driver::uart::puts("[Board Rev   ] failed to get\n"),
    }

    match driver::mbox::get_serial() {
        Some(serial) => {
            driver::uart::puts("[Serial#     ] 0x");
            driver::uart::hex(serial);
            driver::uart::puts("\n");
        }
        None => driver::uart::puts("[Serial#     ] failed to get\n"),
    }

    s
}

fn revision(rev: u32) -> String {
    // https://www.raspberrypi.org/documentation/hardware/raspberrypi/revision-codes/README.md
    // layout (bits)
    // uuuuuuuu FMMMCCCC PPPPTTTT TTTTRRRR

    let s = "".to_string();

    // F: old or new
    if (rev >> 23) & 1 == 0 {
        // old style
        driver::uart::puts("0x");
        driver::uart::hex32(rev);
    } else {
        // new stype

        // MMM: memory size
        match (rev >> 20) & 0b111 {
            0 => {
                driver::uart::puts("256MB Mem, ");
            }
            1 => {
                driver::uart::puts("512MB Mem, ");
            }
            2 => {
                driver::uart::puts("1GB Mem, ");
            }
            3 => {
                driver::uart::puts("2GB Mem, ");
            }
            4 => {
                driver::uart::puts("4GB Mem, ");
            }
            _ => {
                driver::uart::puts("unknown Mem, ");
            }
        }

        // CCCC: manufacturer
        match (rev >> 16) & 0b1111 {
            0 => {
                driver::uart::puts("Sony UK, ");
            }
            1 => {
                driver::uart::puts("Egoman, ");
            }
            2 => {
                driver::uart::puts("Embest, ");
            }
            3 => {
                driver::uart::puts("Sony Japan, ");
            }
            4 => {
                driver::uart::puts("Embest, ");
            }
            5 => {
                driver::uart::puts("Stadium, ");
            }
            _ => {
                driver::uart::puts("unknown, ");
            }
        }

        // PPPP: processor
        match (rev >> 12) & 0b1111 {
            0 => {
                driver::uart::puts("BCM 2835, ");
            }
            1 => {
                driver::uart::puts("BCM 2836, ");
            }
            2 => {
                driver::uart::puts("BCM 2837, ");
            }
            3 => {
                driver::uart::puts("BCM 2711, ");
            }
            _ => {
                driver::uart::puts("unknown, ");
            }
        }

        // TTTTTTTT: type
        match (rev >> 4) & 0b11111111 {
            0 => {
                driver::uart::puts("Model A, ");
            }
            1 => {
                driver::uart::puts("Model B, ");
            }
            2 => {
                driver::uart::puts("Model A+, ");
            }
            3 => {
                driver::uart::puts("Model B+, ");
            }
            4 => {
                driver::uart::puts("Model 2B, ");
            }
            5 => {
                driver::uart::puts("Model Alpha (early prototype), ");
            }
            6 => {
                driver::uart::puts("Model CM1, ");
            }
            8 => {
                driver::uart::puts("Model 3B, ");
            }
            9 => {
                driver::uart::puts("Model Zero, ");
            }
            10 => {
                driver::uart::puts("Model CM3, ");
            }
            12 => {
                driver::uart::puts("Model Zero W, ");
            }
            13 => {
                driver::uart::puts("Model 3B+, ");
            }
            16 => {
                driver::uart::puts("Model CM3+, ");
            }
            17 => {
                driver::uart::puts("Model 4B, ");
            }
            _ => {
                driver::uart::puts("unknown, ");
            }
        }

        // RRRR: Revision
        driver::uart::puts("Rev. ");
        driver::uart::decimal((rev & 0b1111) as u64);
    }

    s
}

fn firmware_version() {
    match driver::mbox::get_firmware_version() {
        Some(ver) => {
            driver::uart::puts("[Firmware Ver] ");
            driver::uart::decimal(ver as u64);
            driver::uart::puts("\n");
        }
        None => driver::uart::puts("[Firmware Ver] failed to get\n"),
    }
}
