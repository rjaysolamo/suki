fn main() {
    // Android devices can use 4 KB or 16 KB pages. NDK r27 needs explicit flags.
    // Keep this on the final Android binary; dx sets its own target rustflags.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        println!("cargo::rustc-link-arg=-Wl,-z,max-page-size=16384");
        println!("cargo::rustc-link-arg=-Wl,-z,common-page-size=16384");
    }
}
