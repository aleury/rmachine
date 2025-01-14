ex1:
	rm -rf ex1
	riscv64-linux-gnu-as ./examples/ex1.s -o ex1.o
	riscv64-linux-gnu-gcc -o ex1 ex1.o -nostdlib -static

ex2:
	rm -rf ex2
	riscv64-linux-gnu-as ./examples/ex2.s -o ex2.o
	riscv64-linux-gnu-gcc -o ex2 ex2.o -nostdlib -static
