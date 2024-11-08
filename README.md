# bbc-b
Pet project to learn about 6502 cpu, Rust and BBC micro computer

* ALU does binary `ADC`/`SBC`, shifts and rotates, bitwise boolean, ...
* Addressing modes tested in original prototype
* Stack, register tranfers, etc
* Branch, jump and subroutines
* `BRK` and rudimentary IRQ and NMI handling
## todo
* Instruction excution puts PC increment in wrong place
* Missing all peripherals
* No timing or cycle counting whatsoever
* needs mos6522 for system VIA to build some kind of keyboard interface
* needs quick&dirty frame buffer to see what's going on (do proper video ULA and 6845 later)
