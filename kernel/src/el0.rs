use crate::aarch64::{cpu, delays, mmu};
use crate::driver::uart;
use crate::lang;
use crate::slab;

use alloc::boxed::Box;

const GLOBAL_CODE: &str = "
(data Dim2 (Dim2 Int Int))

(data (Maybe t)
    (Just t)
    Nothing)

(defun id (x) (Pure (-> (t) t))
    x)

(export label-test () (Pure (-> () Int))
    (match (Just 10)
        ((Just x) x)
        (Nothing 0)))

(export callback-test (x)
    (IO (-> (Int) Int))
    (call-rust x 0 0))

(export lambda-test (f) (Pure (-> ((Pure (-> (Int) Int))) Int))
    (mul2 (f 2)))

(defun mul2 (x) (Pure (-> (Int) Int))
    (* 2 x))

(export tail-call-test (n) (Pure (-> (Int) Int))
    (if (<= n 0)
        0
        (tail-call-test (- n 1))))

(export factorial (n) (Pure (-> (Int) Int))
    (if (<= n 0)
        1
        (* n (factorial (- n 1)))))
";

fn callback(x: i64, y: i64, z: i64) -> i64 {
    let msg = format!("callback: x = {}, y = {}, z = {}\n", x, y, z);
    uart::puts(&msg);
    x * y * z
}

fn run_lisp() {
    // initialize
    match lang::init(GLOBAL_CODE) {
        Ok(exprs) => {
            // typing
            match lang::typing(&exprs) {
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
                    let msg = format!("{:#?}\n", e);
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

fn repl_uart(ctx: &lang::semantics::Context) -> ! {
    loop {
        uart::puts("input code:\n");
        let code_str = uart::read_line();
        let code = alloc::str::from_utf8(&code_str).unwrap();
        uart::puts("code = ");

        uart::puts(code);
        uart::puts("\n");
        uart::puts("run\n\n");

        let result = lang::eval(code, &ctx);
        let msg = format!("{:#?}\n", result);
        uart::puts(&msg);
    }
}

#[no_mangle]
pub fn el0_entry() -> ! {
    let addr = mmu::Addr::new();

    // initialize memory allocator
    slab::init(&addr);

    // wake up slave CPUs
    cpu::send_event();
    cpu::wait_event();

    uart::puts("global code:\n");
    uart::puts(GLOBAL_CODE);
    uart::puts("\n");

    run_lisp();

    // let p = 0x400000000 as *mut u64;
    // unsafe { *p = 10 };

    delays::infinite_loop()
}

/*
const EVAL_CODE: &str =
"
(test-callback 30 40 50)
[10 20 30]
(xor (and true false) true)
(* (+ 143 200) 10)
(Cons 30 (Cons 20 (Cons 10 Nil)))
'(30 20 10)
(let ((x 10) (y 20) (z 30))
    '(z y x))
(test-label 700 50)
(match (Cons 50 Nil)
    ((Cons x _) x))
((lambda-test 30) 40)
(let ((x (lambda (x) (* x 2))))
    (x 50))
";

(defun test-match (a) (Pure (-> ((Maybe Dim2)) Int))
    (match a
        ((Just val)
            (test-let val))
        (Nothing
            0)))

(defun test-let (b) (Pure (-> (Dim2) Int))
    (let (((Dim2 x y) b))
                y))
";


    let code =
"
(defun test_tuple (z) (Pure (-> ([Bool Int]) Bool))
    (let (([v1 v2] z))
        v1))
";

    let code =
"
(data (Dim2 t)
    (Dim2 t t))

(data (Maybe t)
    (Just t)
    Nothing)

(data (Tree t)
    (Node (Tree t) (Tree t))
    Leaf)

(defun test_if (x y) (Pure (-> (Bool Bool) Bool))
    (if x x y))

(defun test_let (z) (Pure (-> ((Dim2 Int)) Int))
    (let (((Dim2 n1 n2) z))
        n1))

(defun test_tuple (z) (Pure (-> ([Bool Int]) Int))
    (let (([v1 v2] z))
        v1))
";
*/
