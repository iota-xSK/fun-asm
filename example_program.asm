|0100
start:
    lit r1 
    00
    lit r2 
    01
    r r5 

    lit r1 
    h_on_state
    lit r2 
    l_on_state
    cjmp r5

    lit r1 
    h_start
    lit r2 
    l_start
    jmp

    on_state:
     lit r6
     01
     lit r1 
     00
     lit r2 
     00
     w r6 ; write led
    on_state_end:
     lit r1 
     h_on_state_end
     lit r2 
     l_on_state_end
     jmp
