CC ?= clang
CFLAGS ?= -Wall -Wextra -O2 -g -fsanitize=address,undefined -fPIC -std=c99

all: librax.a test_rax

rax.o: rax.c rax.h rax_malloc.h
	$(CC) $(CFLAGS) -c rax.c -o rax.o

librax.a: rax.o
	ar rcs librax.a rax.o

test_rax: main_test.c librax.a
	$(CC) $(CFLAGS) main_test.c -L. -lrax -o test_rax

clean:
	rm -f *.o *.a test_rax compile_commands.json
