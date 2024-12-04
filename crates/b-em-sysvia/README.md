# B-em System VIA excerpt

As I've been wrestling with keyboard emulation I decided to make a little
detour into Rust FFI-land. I yanked `via.c` and `sysvia.c` plus a bunch of
headers from another BBC emulator that does a better job: Stardot's
[B-em](https://github.com/stardot/b-em).

Compilation using the *cc* crate from `build.rs` works without issues (on
Linux at least). Stripping the C code and adding stubs is straight forward too.

Need to figure out how to Rust sub-crates work and how to hook it into my
emulator, but ... so far, so good.

