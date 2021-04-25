use crate::driver::uart;
use crate::syscall;

use alloc::boxed::Box;
use blisp;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

const APPS: &'static [&'static str] = &[include_str!("init.lisp")];

fn callback(x: &BigInt, y: &BigInt, _z: &BigInt) -> Option<BigInt> {
    let c = x.to_u64()?;
    match c {
        syscall::SYS_SPAWN => {
            // call spawn
            let app = y.to_usize()?;
            let n = syscall::spawn(app)?;
            let n = BigInt::from_u32(n)?;
            Some(n)
        }
        syscall::SYS_SCHED => {
            syscall::sched_yield();
            None
        }
        syscall::SYS_GETPID => {
            let id = syscall::getpid();
            let id = BigInt::from_u32(id)?;
            Some(id)
        }
        _ => None,
    }
}

fn run_lisp(s: &str) {
    uart::puts(s);
    uart::puts("\n");

    // initialize
    match blisp::init(s) {
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

fn get_app(id: usize) -> Option<&'static str> {
    if id >= APPS.len() {
        None
    } else {
        Some(APPS[id])
    }
}

#[no_mangle]
pub fn el0_entry(app: usize) -> ! {
    uart::puts("entered EL0\n");
    if let Some(s) = get_app(app) {
        run_lisp(s);
    } else {
        uart::puts("no such application\n");
        syscall::exit();
    }

    loop {}
}
