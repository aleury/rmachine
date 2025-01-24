_start:
  li a0, 1
  la a1, helloworld
  li a2, 13
  li a7, 64
  ecall
  ebreak

helloworld:
  .ascii "Hello World!\n"
