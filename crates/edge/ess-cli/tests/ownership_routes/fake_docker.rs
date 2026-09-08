//! Local process witness; it never invokes Docker or a network service.
fn main() {
    std::fs::write(std::env::var_os("ESS_OWNERSHIP_DOCKER_WITNESS").expect("witness path"),b"executor reached").unwrap();
}
