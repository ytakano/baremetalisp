#include <stdint.h>

#define PAGESIZE (64 * 1024)

// flags of level 3 table
#define FLAG_L2_NS   (1LL << 63) // non secure table

// flags of level 2 table
#define FLAG_L3_XN   (1LL << 54) // execute never
#define FLAG_L3_PXN  (1LL << 53) // priviledged execute
#define FLAG_L3_CONT (1LL << 52) // contiguous
#define FLAG_L3_DBM  (1LL << 51) // dirty bit modifier
#define FLAG_L3_AF   (1LL << 10) // access flag
#define FLAG_L3_NS   (1LL <<  5) // non secure

// [9:8]: Shareability attribute, for Normal memory
//    | Shareability
// ---|------------------
// 00 | non sharedable
// 01 | reserved
// 10 | outer sharedable
// 11 | inner sharedable
#define FLAG_L3_OSH (0b10 << 8)
#define FLAG_L3_ISH (0b11 << 8)

// [7:6]: access permissions
//    | Access from            |
//    | higher Exception level | Access from EL0
// ---|------------------------|-----------------
// 00 | read/write             | none
// 01 | read/write             | read/write
// 10 | read-only              | none
// 11 | read-only              | read-only
#define FLAG_L3_SH_RW_N      0
#define FLAG_L3_SH_RW_RW (   1 << 6)
#define FLAG_L3_SH_R_N   (0b10 << 6)
#define FLAG_L3_SH_R_R   (0b11 << 6)

// [4:2]: AttrIndx
// defined in MAIR register
#define FLAG_L3_ATTR_MEM (0 << 2) // normal memory
#define FLAG_L3_ATTR_DEV (1 << 2) // device MMIO
#define FLAG_L3_ATTR_NC  (2 << 2) // non-cachable

extern volatile unsigned char _end;

void init_mmu() {
    uint64_t* const L2ENTRY = (uint64_t*)&_end;
    uint64_t* const L3ENTRY = L2ENTRY + 8192;

    int i;
    for (i = 0; i < 9; i++) { // 4GiB space
        L2ENTRY[i] = (uint64_t)&(L3ENTRY[8192 * i]) | 0b11;
    }

    for (i = 9; i < 8192; i++) {
        L2ENTRY[i] = 0;
    }

    for (i = 0; i < 8192; i++) { // 512MiB space
        L3ENTRY[i] = i * 64 * 1024 | 0b11 |
            FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM;
    }

#ifdef raspi4
    // 0xfd500000 ... 0xffffffff
    for (i = 8192 * 6; i < 8192 * 9; i++) { // device
        L3ENTRY[i] = i * 64 * 1024 | 0b11 |
            FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_OSH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_DEV;
    }
#else
    for (i = 15360; i < 16896; i++) { // device
        L3ENTRY[i] = i * 64 * 1024 | 0b11 |
            FLAG_L3_NS | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_OSH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_DEV;
    }
#endif // raspi4

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    uint64_t mair = (0xFF <<  0) | // AttrIdx=0: normal, IWBWA, OWBWA, NTR
                    (0x04 <<  8) | // AttrIdx=1: device, nGnRE (must be OSH too)
                    (0x44 << 16);  // AttrIdx=2: non cacheable
#ifdef raspi4
    asm volatile ("msr mair_el3, %0" : : "r" (mair));
#else
    asm volatile ("msr mair_el2, %0" : : "r" (mair));
#endif // raspi4

    uint64_t mmfr;
    asm volatile ("mrs %0, id_aa64mmfr0_el1" : "=r" (mmfr));
    mmfr &= 0b111;

    // next, specify mapping characteristics in translate control register
    uint64_t tcr = 1 << 31 | // Res1
                   1 << 23 | // Res1
                   mmfr << 16 |
                   1 << 14 | // 64KiB granule
                   3 << 12 | // inner shadable
                   1 << 10 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.
                   1 <<  8 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable.
                   32;       // T0SZ = 32, 2 levels (level 2 and 3 translation tables), 4GiB space
#ifdef raspi4
    asm volatile ("msr tcr_el3, %0" : : "r" (tcr));
#else
    asm volatile ("msr tcr_el2, %0" : : "r" (tcr));
#endif // raspi4

    // tell the MMU where our translation tables are.
#ifdef raspi4
    asm volatile ("msr ttbr0_el3, %0" : : "r" ((uint64_t)L2ENTRY | 1));
#else
    asm volatile ("msr ttbr0_el2, %0" : : "r" ((uint64_t)L2ENTRY | 1));
#endif // raspi4


    // finally, toggle some bits in system control register to enable page translation
    uint64_t sctlr;
#ifdef raspi4
    asm volatile ("dsb ish;"
                  "isb;"
                  "mrs %0, sctlr_el3" : "=r" (sctlr));
#else
    asm volatile ("dsb ish;"
                  "isb;"
                  "mrs %0, sctlr_el2" : "=r" (sctlr));
#endif // raspi4

    sctlr |=
        1 << 12 | // no effect on the Cacheability of instruction access to Normal memory
        1 <<  2 | // no effect on the Cacheability
        1;        // set M, enable MMU
    sctlr &= ~(
        1 << 25 | // clear EE
        1 << 19 | // clear WXN
        1 <<  3 | // clear SA
        1 <<  1   // clear A
    );

#ifdef raspi4
    asm volatile ("msr sctlr_el3, %0; dsb sy; isb" : : "r" (sctlr));
#else
    asm volatile ("msr sctlr_el2, %0; dsb sy; isb" : : "r" (sctlr));
#endif // raspi4
}
