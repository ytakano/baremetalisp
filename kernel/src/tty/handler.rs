use crate::{bsp::uart, driver::uart::UART};

const TY_BACKSP: u8 = 0x08; // backspace,, ^H
const TY_BACKSP2: u8 = 0x7f; // delete, ^?
const TY_BELL: u8 = 0x07; // bell, ^G
const TY_EOFCH: u8 = 0x04; // end of transmission, ^D
const TY_BLANK: u8 = b' '; // white space
const TY_NEWLINE: u8 = b'\n'; // line feed
const TY_RETURN: u8 = b'\r'; // carrige return
const TY_STOPCH: u8 = 0x13; // tty stop, ^S
const TY_STRTCH: u8 = 0x11; // tty start, ^Q
const TY_KILLCH: u8 = 0x16; // line kill, ^U
const TY_UPARROW: u8 = b'^'; // used by control caracters
const TY_FULLCH: u8 = TY_BELL; //

enum Mode {
    Raw,
    Cooked,
    Cbreak,
}

struct Cooked {
    is_crlf_echo: bool, // echo CR-LF when echo
    is_map_crlf: bool,  // map \r to \n
}

struct TTY {
    mode: Mode,
    is_echo: bool,
    is_rm_bs_echo: bool, // remove backspace when echo
    is_evis: bool,       // echo control ^X

    is_rcv_erase: bool, // receive erase characters
    erasec: u8,         // erase character
    eracec2: u8,        // another erace character

    is_rcv_eof: bool, // receive EOF
    eofch: u8,        // EOF character

    is_rcv_kill: u8, // receive kill character
    killc: u8,       // kill character

    icursor: isize, // current position of cursor

    is_rcv_oflow: bool, // receive ostop/ostart
    is_oheld: bool,     // output character beeing held?

    ostop: u8,  // character to stop output
    ostart: u8, // character to start output

    is_ocrlf: bool, // output CR-LF for LF

    fullc: u8, // character to be sent when buffer is full
}

fn echo(uart0: &uart::DevUART, c: u8) {
    uart0.send(c as u32);
}

fn drop(_uart0: &uart::DevUART, _c: u8) {}

impl TTY {
    pub fn int_handler(&self) {
        let mut buf = [0; 1024];

        let ef = if self.is_echo { echo } else { drop };
        let _len = uart::read(&mut buf, ef);
    }
}
