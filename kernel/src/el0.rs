use crate::aarch64::{mmu, syscall};
use crate::driver::{delays, uart};
use crate::memalloc;

use alloc::boxed::Box;
use blisp;

const GLOBAL_CODE: &str = "
(data (Maybe t)
    (Just t)
    Nothing)

(export car (x) (Pure (-> ('(Int)) (Maybe Int)))
    (match x
        ((Cons n _) (Just n))
        (_ Nothing)))

(export cdr (x) (Pure (-> ('(Int)) '(Int)))
    (match x
        ((Cons _ l) l)
        (_ '())))

; switch to normal world
(export switch-world () (IO (-> () Int))
    (call-rust 1 0 0))

(export factorial (n) (Pure (-> (Int) Int))
    (if (<= n 0)
        1
        (* n (factorial (- n 1)))))
";

fn callback(x: i64, _y: i64, _z: i64) -> i64 {
    if x == 1 {
        syscall::svc::switch_world();
        0
    } else {
        -1
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
                    ctx.set_callback(Box::new(callback));

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
                    uart::puts(r);
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
pub fn el0_entry_core_0() -> ! {
    // initialize memory allocator
    let addr = mmu::get_memory_map();
    let size = addr.el0_heap_end - addr.el0_heap_start;
    let mid = (addr.el0_heap_start + (size >> 1)) as usize;
    memalloc::init(addr.el0_heap_start as usize, mid, mid);

    uart::puts("global code:\n");
    uart::puts(GLOBAL_CODE);
    uart::puts("\n");

    run_lisp();

    // let p = 0x400000000 as *mut u64;
    // unsafe { *p = 10 };

    delays::forever()
}

#[no_mangle]
pub fn el0_entry_core_x() -> ! {
    // TODO:
    // currently, secondary CPUs do nothing
    loop {
        syscall::svc::switch_world();
    }
}
