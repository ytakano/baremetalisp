use crate::aarch64::syscall;
use crate::driver::uart;

use alloc::boxed::Box;
use blisp;
use num_bigint::BigInt;
use num_traits::{One, Zero};

const GLOBAL_CODE: &str = "
(export factorial (n) (Pure (-> (Int) Int))
    (factorial' n 1))

(defun factorial' (n total) (Pure (-> (Int Int) Int))
    (if (<= n 0)
        total
        (factorial' (- n 1) (* n total))))
";

fn callback(x: &BigInt, _y: &BigInt, _z: &BigInt) -> Option<BigInt> {
    if *x == One::one() {
        syscall::svc::switch_world();
        Some(Zero::zero())
    } else {
        None
    }
}

fn run_lisp() {
    // initialize
    match blisp::init(GLOBAL_CODE) {
        Ok(exprs) => {
            // typing
            match blisp::typing(&exprs) {
                Ok(mut ctx) => {
                    // register callback function
                    ctx.set_callback(Box::new(|x, y, z| callback(x, y, z)));

                    repl_uart(&ctx);

                    // eval
                    //let result = lang::eval(EVAL_CODE, &ctx);
                    //let msg = format!("{:#?}\n", result);
                    //uart::puts(&msg);
                }
                Err(e) => {
                    let msg = format!("{}:{}: {}", e.pos.line + 1, e.pos.column + 1, e.msg);
                    uart::puts(&msg);
                }
            }
        }
        Err(e) => {
            let msg = format!("{:#?}\n", e);
            uart::puts(&msg);
        }
    }
}

fn repl_uart(ctx: &blisp::semantics::Context) -> ! {
    loop {
        uart::puts("\n> ");
        let code_str = uart::read_line();
        let code = alloc::str::from_utf8(&code_str).unwrap();

        let result = blisp::eval(code, &ctx);
        match result {
            Ok(rs) => {
                for r in &rs {
                    match r {
                        Ok(msg) => {
                            uart::puts(&msg);
                        }
                        Err(e) => {
                            let msg = format!("error: {}", e);
                            uart::puts(&msg);
                        }
                    }
                }
            }
            Err(e) => {
                let msg = format!("{}:{}: {}", e.pos.line + 1, e.pos.column + 1, e.msg);
                uart::puts(&msg);
            }
        }
    }
}

#[no_mangle]
pub fn el0_entry() -> ! {
    crate::print_msg("EL0", "Entered");

    //memalloc::init(addr.el0_heap_start as usize, mid, mid);

    uart::puts("global code:\n");
    uart::puts(GLOBAL_CODE);
    uart::puts("\n");

    run_lisp();

    // let p = 0x400000000 as *mut u64;
    // unsafe { *p = 10 };

    loop {}
}
