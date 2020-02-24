SRCS = $(wildcard *.c)
OBJS = $(SRCS:.c=.o)
CFLAGS = -Wall -O2 -ffreestanding -nostdinc -nostdlib -nostartfiles
CC = aarch64-linux-gnu-gcc
AR = aarch64-linux-gnu-ar
OUT = libbaremetal_c.a
TARGET = raspi4

all: $(OUT)

%.o: %.c %.h gpio.h
	$(CC) $(CFLAGS) -D$(TARGET) -c $< -o $@

$(OUT): $(OBJS)
	$(AR) r $(OUT) $(OBJS)

clean:
	rm -f *.o $(OUT)