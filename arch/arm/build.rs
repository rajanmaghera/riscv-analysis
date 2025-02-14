use cmake::Config;

fn export_env(var: &str, value: &str) {
    println!("cargo::rustc-env={var}={value}");
}

#[allow(unused)]
macro_rules! echo {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    };
}

fn main() {
    // Build the parser
    let dst = Config::new("parser")
        .build_target("all")
        .build();
    println!("cargo:rustc-link-search=native={}", dst.display());

    // Return the path of the parser binary
    let bin_dir = dst.as_path().join("build/lib");
    let bin = bin_dir.join("arm-parser");
    let bin = bin.to_str().unwrap();
    export_env("RVA_AARCH64_PARSER", bin);
}
