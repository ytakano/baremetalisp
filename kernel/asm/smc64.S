#include "teesmc_optee.h"

.global thread_vector_table

thread_vector_table:
    b       vector_std_smc_entry
    b       vector_fast_smc_entry
    b       vector_cpu_on_entry
    b       vector_cpu_off_entry
    b       vector_cpu_resume_entry
    b       vector_cpu_suspend_entry
    b       vector_fiq_entry
    b       vector_system_off_entry
    b       vector_system_reset_entry

vector_std_smc_entry:
    bl      thread_handle_std_smc

    mov     w1, w0
    ldr     x0, =TEESMC_OPTEED_RETURN_CALL_DONE
    smc     #0
    b       .       /* SMC should not return */

vector_fast_smc_entry:
    sub     sp, sp, #16 * 4

    stp     x0,  x1,  [sp, #16 * 0]
    stp     x2,  x3,  [sp, #16 * 1]
    stp     x4,  x5,  [sp, #16 * 2]
    stp     x6,  x7,  [sp, #16 * 3]

    mov     x0, sp
    bl      thread_handle_fast_smc

    ldp     x1,  x2,  [sp, #16 * 0]
    ldp     x3,  x4,  [sp, #16 * 1]
    ldp     x5,  x6,  [sp, #16 * 2]
    ldp     x7,  x8,  [sp, #16 * 3]

    add     sp,  sp,  #16 * 4

    ldr     x0, =TEESMC_OPTEED_RETURN_CALL_DONE
    smc     #0
    b       .       /* SMC should not return */

vector_cpu_on_entry:
vector_cpu_off_entry:
vector_cpu_resume_entry:
vector_cpu_suspend_entry:
vector_fiq_entry:
vector_system_off_entry:
vector_system_reset_entry:
    smc     #0
    b       .       /* SMC should not return */
