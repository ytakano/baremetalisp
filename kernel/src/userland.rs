use crate::{driver::uart, syscall, syscall::Locator};

use alloc::boxed::Box;
use memac::Allocator;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive, Zero};

const APPS: &[&str] = &[include_str!("init.lisp")];

fn callback(x: &BigInt, y: &BigInt, z: &BigInt) -> Option<BigInt> {
    let c = x.to_u64()?;
    match c {
        syscall::SYS_SPAWN => {
            // call spawn
            let app = y.to_usize()?;
            let n = syscall::spawn(app)?;
            let n = BigInt::from_u32(n)?;
            Some(n)
        }
        syscall::SYS_EXIT => {
            syscall::exit();
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
        syscall::SYS_SEND => {
            let loc = Locator::Process(y.to_u32()?);
            if syscall::send(&loc, z.to_u32()?) {
                Some(Zero::zero())
            } else {
                None
            }
        }
        syscall::SYS_RECV => {
            let mut loc = Locator::Unknown;
            let val = syscall::recv(&mut loc);
            let val = BigInt::from_u32(val)?;
            Some(val)
        }
        syscall::SYS_KILL => {
            syscall::kill(y.to_u32()?);
            None
        }
        _ => {
            let msg = format!("unsupported syscall: {}\n", c);
            uart::puts(&msg);
            None
        }
    }
}

fn run_lisp(s: &str) {
    uart::puts(s);
    uart::puts("\n");

    // initialize
    match blisp::init(s, vec![]) {
        Ok(exprs) => {
            // typing
            match blisp::typing(exprs) {
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
        let pid = syscall::getpid();
        let msg = format!("\n(pid: {}) >> ", pid);
        uart::puts(&msg);

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
    let mut allc = Allocator::new();
    syscall::set_allocator(&mut allc);

    if id >= APPS.len() {
        None
    } else {
        Some(APPS[id])
    }
}

#[no_mangle]
pub fn userland_entry(app: usize) -> ! {
    use crate::out;
    out::decimal("app id", app as u64);
    if let Some(s) = get_app(app) {
        run_lisp(s);
    } else {
        uart::puts("no such application\n");
        syscall::exit();
    }

    loop {}
}
