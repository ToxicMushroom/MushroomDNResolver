mod server;
mod lookup;

use anyhow::Error;
use hickory_resolver::config::*;
use hickory_resolver::TokioResolver;
//
// /// 2 ms response time = 1% reliability
// const RESP_TIME_ON_SUCCESS_RATE: f64 = 2.0/1.0;
//
// const SLIDING_WINDOW: u8 = 100; // Last X queries will count towards reliability
// const ALPHA: f64 = 0.2; // percentage (0-1) of how much we want to forget the last result.
//
// struct WeightedResolver {
//     last_reliability: f64,
//     avg_response_time: f64,
//     resolver: TokioResolver
// }
//
// impl WeightedResolver {
//     fn new(resolver: TokioResolver) -> WeightedResolver {
//
//     }
//
//     fn recompute_reliability(success: bool)  {
//
//     }
//
//     // smaller better
//     fn heuristic() -> usize {
//         let
//     }
// }


#[tokio::main]
async fn main() -> Result<(), Error>{
    // Construct a new Resolver with default configuration options
    // let tokio_fallback_resolver = TokioResolver::tokio(ResolverConfig::google(), ResolverOpts::default());
    let resolver = TokioResolver::tokio(ResolverConfig::cloudflare_tls(), ResolverOpts::default());

    server::server(resolver).await?;

    Ok(())
}
