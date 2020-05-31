use crate::slab;
use crate::lang;
use crate::driver;

use alloc::boxed::Box;

const GLOBAL_CODE: &str =
"
(data Dim2 (Dim2 Int Int))

(data (Maybe t)
    (Just t)
    Nothing)

(defun id (x) (Pure (-> (t) t))
    x)

(export test-label (x y) (Pure (-> (Int Int) (Maybe Dim2)))
    (Just (id (Dim2 x y))))

(export test-callback (x y z) (IO (-> (Int Int Int) Int))
    (call-rust x y z))

(export lambda-test () (Pure (-> () (Pure (-> (Int Int) Int))))
    (lambda (x y) (+ x y)))
";

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
";

fn callback(x: i64, y: i64, z: i64) -> i64 {
    let msg = format!("callback: x = {}, y = {}, z = {}\n", x, y, z);
    driver::uart::puts(&msg);
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

                    // eval
                    let result = lang::eval(EVAL_CODE, &ctx);
                    let msg = format!("{:#?}\n", result);
                    driver::uart::puts(&msg);
                }
                Err(e) => {
                    let msg = format!("{:#?}\n", e);
                    driver::uart::puts(&msg);
                }
            }
        }
        Err(e) => {
            let msg = format!("{:#?}\n", e);
            driver::uart::puts(&msg);
        }
    }
}

#[no_mangle]
pub fn el0_entry() -> ! {
    // initialize memory allocator
    slab::init();

    driver::uart::puts("global code:\n");
    driver::uart::puts(GLOBAL_CODE);
    driver::uart::puts("\n");

    driver::uart::puts("eval code:\n");
    driver::uart::puts(EVAL_CODE);
    driver::uart::puts("\n");

    run_lisp();

    let p = 0x400000000 as *mut u64;
    unsafe { *p = 10 };

    loop{}
}


/*
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