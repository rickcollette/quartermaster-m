; MADS: A=ATASCII, returns A=Graphics 0 screen code
AtasciiToScreen:
        pha
        and #$7f
        cmp #32
        bcc low
        cmp #96
        bcc printable
        pla
        rts
low:    clc
        adc #64
        sta temp
        pla
        and #$80
        ora temp
        rts
printable:
        sec
        sbc #32
        sta temp
        pla
        and #$80
        ora temp
        rts
temp    .byte 0
