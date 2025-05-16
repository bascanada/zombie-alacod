fn main() {
    let version = std::env::var("APP_VERSION")
        .expect("APP_VERSION environment variable must be set by the Makefile");

    println!("cargo:rustc-env=APP_VERSION={}", version);
}