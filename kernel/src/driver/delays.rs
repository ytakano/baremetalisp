pub mod bcm2xxx;
pub mod generic;

pub trait Delays {
    fn init(&self);
    fn get_timer_value(&self) -> usize;
    fn wait_microsec(&self, usec: usize);
    fn wait_milisec(&self, msec: usize) {
        self.wait_microsec(msec * 1000);
    }
}
