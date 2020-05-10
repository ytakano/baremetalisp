use crate::slab;
use crate::parser;
use crate::semantics;
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

(defun match-let (a) (Pure (-> ((Maybe Dim2)) Int))
    (match a
        ((Just val)
            (let (((Dim2 x y) val))
                y))
        (Nothing
            0)))
";
/*
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
    driver::uart::puts("Input:\n  ");
    driver::uart::puts(code);
    driver::uart::puts("\n");

    let mut ps = parser::Parser::new(code);
    match ps.parse() {
        Ok(e) => {
            let msg = format!("AST:\n  {:#?}\n", e);
            driver::uart::puts(&msg);

            match semantics::exprs2context(&e) {
                Ok(mut ctx) => {
                    match ctx.typing() {
                        Err(err) => {
                            let msg = format!("Typing Error:\n  {:?}\n", err);
                            driver::uart::puts(&msg);
                        }
                        _ => {
                            let msg = format!("Context:\n  {:#?}\n", ctx);
                            driver::uart::puts(&msg);
                        }
                    }
                }
                Err(err) => {
                    let msg = format!("Context Error:\n  {:?}\n", err);
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