fn main() {

    println!("cargo:rustc-flags=-l liquid -L liquid");
    println!("cargo:rustc-link-search=./");

}
