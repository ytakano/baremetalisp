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
(defun add (x y) (Pure (-> (Int Int) Int))
  (+ x y))
";

    driver::uart::puts("Input:\n  ");
    driver::uart::puts(code);
    driver::uart::puts("\n");

    let mut ps = parser::Parser::new(code);
    match ps.parse() {
        Ok(e) => {
            let msg = format!("AST:\n  {:?}\n", e);
            driver::uart::puts(&msg);

            match semantics::typing(&e) {
                Ok(cxt) => {
                    let msg = format!("Context:\n  {:?}\n", cxt);
                    driver::uart::puts(&msg);
                }
                Err(err) => {
                    let msg = format!("Typing Error:\n  {:?}\n", err);
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