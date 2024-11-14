mod server;
mod lookup;

use anyhow::Error;
use hickory_resolver::config::*;
use hickory_resolver::TokioResolver;

#[tokio::main]
async fn main() -> Result<(), Error>{
    // Construct a new Resolver with default configuration options
    let tokio_fallback_resolver = TokioResolver::tokio(ResolverConfig::google(), ResolverOpts::default());
    let resolver = TokioResolver::tokio_from_system_conf().unwrap_or_else(|e| {
        println!("Failed to load system DNS config /etc/resolv.conf or registry entries on w*ndows: {}", e);
        tokio_fallback_resolver
    });

    server::server(resolver).await?;

    Ok(())
}
