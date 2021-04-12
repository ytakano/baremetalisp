#if defined(raspi3) || defined(raspi4)
    #include "device/raspi.S"
#elif defined(pine64)
    #include "device/pine64.S"
#endif

#include "smc64.S"