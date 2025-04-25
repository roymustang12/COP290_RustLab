#[cfg(feature = "main2")]
fn main2() {
    lalrpop::Configuration::new()
        .generate_in_source_tree()
        .process()
        .unwrap();
}
fn main() {
    #[cfg(feature = "main2")]
    main2();
}
