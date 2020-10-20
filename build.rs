fn main() {
    cc::Build::new()
        .cpp(true)
        .file("src/shared_ptr.cc")
        .compile("native");
}
