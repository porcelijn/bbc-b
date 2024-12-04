use cc;

fn main() {
  // automatically rebuild if C files change
  println!("cargo::rerun-if-changed=src/sysvia.c");
  println!("cargo::rerun-if-changed=src/via.c");
  // automatically rebuild if C headers change
  println!("cargo::rerun-if-changed=src/keyboard.h");
  println!("cargo::rerun-if-changed=src/led.h");
  println!("cargo::rerun-if-changed=src/sn76489.h");
  println!("cargo::rerun-if-changed=src/sysvia.h");
  println!("cargo::rerun-if-changed=src/via.h");
  println!("cargo::rerun-if-changed=src/video.h");
  // use the `cc` crate to build a C file and statically link it
  cc::Build::new()
    .file("src/via.c")
    .file("src/sysvia.c")
    .compile("b-em-via");

  // implicitly linked with src/lib.rs
}
