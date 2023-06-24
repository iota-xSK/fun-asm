|0100
start:
   lit r4
   01
   lit r5
   01
   lit r2
   l_add
   lit r1
   h_add
   call
   halt
ff
ff
ff
ff
ff
ff
ff
ff
ff
ff
|0200
add:
   tac r4
   add r5
   ret
