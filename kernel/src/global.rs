use core::mem;

pub enum GlobalVar<T> {
    UnInit,
    Taked,
    Having(T),
}

impl<T> GlobalVar<T> {
    pub fn take(&mut self) -> Self {
        if let GlobalVar::UnInit = self {
            panic!("uninitialized");
        }
        mem::take(self)
    }
}

impl<T> Default for GlobalVar<T> {
    fn default() -> Self {
        GlobalVar::Taked
    }
}
