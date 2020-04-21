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
(data (Maybe t)
    (Just t)
    Nothing)

(data (Tree t)
    (Node [(Tree t) (Tree t)])
    Leaf)

(defun add (x y) (Pure (-> (Int Int) Int))
  (+ x y))
";

    driver::uart::puts("Input:\n  ");
    driver::uart::puts(code);
    driver::uart::puts("\n");

    let mut ps = parser::Parser::new(code);
    match ps.parse() {
        Ok(e) => {
            let msg = format!("AST:\n  {:#?}\n", e);
            driver::uart::puts(&msg);

            match semantics::exprs2context(&e) {
                Ok(ctx) => {
                    let msg = format!("Context:\n  {:#?}\n", ctx);
                    driver::uart::puts(&msg);

                    match ctx.typing() {
                        Err(err) => {
                            let msg = format!("Typing Error:\n  {:?}\n", err);
                            driver::uart::puts(&msg);
                        }
                        _ => {}
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