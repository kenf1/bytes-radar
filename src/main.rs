#[cfg(feature = "cli")]
use bytes_radar::cli;

#[cfg(feature = "cli")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::run().await?;
    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    panic!("CLI feature is not enabled");
}
