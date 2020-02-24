#include "uart.h"
#include "lfb.h"

extern void rust_entry();

#define UART_CLOCK  48000000
#define UART_BAUD   115200

void entry() {
    // set up serial console and linear frame buffer
    uart_init(UART_CLOCK, UART_BAUD);
    lfb_init();

    // display a pixmap
    lfb_showpicture();

    rust_entry();
}