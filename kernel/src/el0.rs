use crate::slab;
use crate::lang::parser;
use crate::lang::semantics;
use crate::lang::runtime;
use crate::driver;

#[no_mangle]
pub fn el0_entry() -> ! {
    // initialize memory allocator
    slab::init();

    let code =
"
(data Dim2 (Dim2 Int Int))

(data (Maybe t)
    (Just t)
    Nothing)

(defun id (x) (Pure (-> (t) t))
    x)

(export test-label () (Pure (-> () Dim2))
    (id (Dim2 10 20)))
";

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
    driver::uart::puts("Input:\n");
    driver::uart::puts(code);
    driver::uart::puts("\n");

    let mut ps = parser::Parser::new(code);
    match ps.parse() {
        Ok(e) => {
            match semantics::exprs2context(&e) {
                Ok(ctx) => {
                    let expr =
"
(Cons 30 (Cons 20 (Cons 10 Nil)))
'(30 20 10)
(let ((x 10) (y 20) (z 30))
    '(z y x))
test-label
(match (Cons 50 Nil)
    ((Cons x _) x))
";
                    driver::uart::puts("Eval:\n");
                    driver::uart::puts(expr);
                    driver::uart::puts("\n");

                    let result = runtime::eval(expr, &ctx);
                    let msg = format!("{:#?}\n", result);
                    driver::uart::puts(&msg);
                }
                Err(err) => {
                    let msg = format!("{:#?}\n", err);
                    driver::uart::puts(&msg);
                }
            }
        }
        Err(err) => {
            let msg = format!("Syntax Error:\n  {:?}\n", err);
            driver::uart::puts(&msg);
        }
    }

    let p = 0x400000000 as *mut u64;
    unsafe { *p = 10 };

    loop{}
}