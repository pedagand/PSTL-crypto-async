extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/des.c")
        .compile("libdes.a");
}
