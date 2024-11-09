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

Hmmm. So far so good. Booting the OS 1.20 rom and looking for signs of live in (aleged) video area (`0x3000` - `0x7FFF`) yields only zeroes, except for a misterious:                                                                     
```                                                                             
00004880  7e 18 18 18 18 18 18 00  7e 18 18 18 18 18 18 00  |~.......~.......|  
```                                                                             
Emulating Mode 0, 3, or 4 monochrome graphics, this looks like `TT`. Hmm.    
![Screenshot-2024-11-09](https://github.com/user-attachments/assets/7159f916-6cf3-4d72-bda9-4e6889c4789a)
