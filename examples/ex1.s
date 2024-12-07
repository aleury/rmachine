.global _start

.section .text
_start:
  li a0, 1  # fd = 1 (stdout)
  la a1, helloworld
  li a2, 14
  li a7, 64 # write syscall
  ecall

  li a0, 0 # status = 0
  li a7, 93 # exit syscall
  ecall

helloworld:
  .ascii "Hello World!\n"
