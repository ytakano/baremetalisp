use crate::aarch64::bits::clz;

/// 64 * 64 * 64 pages = 64 * 64 * 64 * 64KiB = 16GiB
///
/// ```
/// static mut PAGEMNG: PageManager = PageManager{
///     start: 0,
///     end: 64 * 1024 * 1024 * 512,
///     vacancy_books: 0,
///     vacancy_pages: [0; 64],
///     book: [Book{pages: [0; 64]}; 64],
/// };
/// ```
pub struct PageManager {
    pub start: usize,
    pub end: usize,
    pub vacancy_books: u64,
    pub vacancy_pages: [u64; 64],
    pub book: [Book; 64],
}

#[derive(Copy, Clone)]
pub struct Book {
    pub pages: [u64; 64]
}

impl PageManager {
    pub const fn new() -> PageManager {
        PageManager {
            start: 0,
            end: 0,
            vacancy_books: 0,
            vacancy_pages: [0; 64],
            book: [Book{pages: [0; 64]}; 64],
        }
    }

    pub fn alloc(&mut self) -> Option<usize> {
        if self.vacancy_books == !0 {
            return None;
        }

        let idx1 = clz(!self.vacancy_books) as usize;
        let idx2 = clz(!self.vacancy_pages[idx1]) as usize;
        let idx3 = clz(!self.book[idx1].pages[idx2]) as usize;

        let addr = 64 * 1024 * 64 * 64 * idx1 + 64 * 1024 * 64 * idx2 + 64 * 1024 * idx3 + self.start;

        if addr >= self.end {
            return None;
        }

        self.book[idx1].pages[idx2] |= 1 << (63 - idx3);
        if self.book[idx1].pages[idx2] == !0 {
            self.vacancy_pages[idx1] |= 1 << (63 - idx2);
            if self.vacancy_pages[idx1] == !0 {
                self.vacancy_books |= 1 << (63 - idx1);
            }
        }

        return Some(addr);
    }

    pub fn free(&mut self, addr: usize) {
        if addr & 0xFFFF != 0 || addr >= self.end || addr < self.start {
            panic!("invalid address");
        }

        let idx1 = ((addr - self.start) >> 28) & 0b111111;
        let idx2 = (addr >> 22) & 0b111111;
        let idx3 = (addr >> 16) & 0b111111;

        self.book[idx1].pages[idx2] &= !(1 << (63 - idx3));
        self.vacancy_pages[idx1] &= !(1 << (63 - idx2));
        self.vacancy_books &= !(1 << (63 - idx1));
    }
}