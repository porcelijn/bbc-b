# BBC-b
Pet project to learn about 6502 CPU, Rust and BBC micro computer

* ALU does binary `ADC`/`SBC`, shifts and rotates, bit wise Boolean, ...
* Addressing modes tested in original prototype
* Stack, register transfers, ...
* Branch, jump and subroutines
* `BRK` and rudimentary IRQ and NMI handling
* 6502 disassembler
* preliminary benchmark performance ~7e7 instructions / second
## todo
* Instruction execution puts PC increment in wrong place
* Missing all peripherals
* No timing or cycle counting whatsoever
* Memory is flat 64kB RAM --- Needs some way to add memory mapped I/O, maybe write protect ROM area, bank switching, ...
* needs mos6522 for system VIA to build some kind of keyboard interface
* needs quick 'n' dirty frame buffer to see what's going on (do proper video ULA and 6845 later)
* OMG, [Toby Nelson](https://tobylobster.github.io/mos/mos/index.html)'s annotated MOS assembly is a treasure!
