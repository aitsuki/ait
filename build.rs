fn main() {
    #[cfg(windows)]
    embed_resource::compile("assets/ait.rc", embed_resource::NONE)
        .manifest_optional()
        .expect("failed to compile Windows resources");
}
