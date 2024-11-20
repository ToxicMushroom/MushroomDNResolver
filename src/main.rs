pub mod lookup;
pub mod server;
pub mod access;
pub mod authority;
pub mod error;
pub mod store;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use anyhow::Error;
use hickory_resolver::config::*;
use hickory_resolver::TokioResolver;
use sd_notify::NotifyState;
use socket2::{Domain, Socket, Type};
use tokio::net::UdpSocket;
use tokio::runtime;
use tracing::{error, info};
use crate::authority::Catalog;
use crate::authority::mushroom::Mushroom;
use crate::server::ServerFuture;


/// Low-level types for DNSSEC operations
#[cfg(feature = "dnssec")]
pub mod dnssec {
    use hickory_proto::rr::dnssec::Nsec3HashAlgorithm;
    use serde::Deserialize;
    use std::sync::Arc;

    /// The kind of non-existence proof provided by the nameserver
    #[cfg(feature = "dnssec")]
    #[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    pub enum NxProofKind {
        /// Use NSEC
        Nsec,
        /// Use NSEC3
        Nsec3 {
            /// The algorithm used to hash the names.
            #[serde(default)]
            algorithm: Nsec3HashAlgorithm,
            /// The salt used for hashing.
            #[serde(default)]
            salt: Arc<[u8]>,
            /// The number of hashing iterations.
            #[serde(default)]
            iterations: u16,
        },
    }
}

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


fn main() -> Result<(), String>{
    // Construct a new Resolver with default configuration options
    // let tokio_fallback_resolver = TokioResolver::tokio(ResolverConfig::google(), ResolverOpts::default());
    tracing_subscriber::fmt().init();
    let mut runtime = runtime::Builder::new_multi_thread();
    runtime.enable_all().thread_name("hickory-server-runtime");
    runtime.worker_threads(8);
    let runtime = runtime
        .build()
        .map_err(|err| format!("failed to initialize Tokio runtime: {err:?}"))?;


    let resolver = TokioResolver::tokio(ResolverConfig::cloudflare_tls(), ResolverOpts::default());
    let mut binds = vec![];
    let _guard = runtime.enter();
    binds.push(build_udp_socket(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8853));
    // binds.push(build_udp_socket(IpAddr::V6(Ipv6Addr::new()), 53));

    let mut catalog: Catalog = Catalog::new(); // Used for serving zone files
    let mushroom = Mushroom {
        resolver,
    };
    let deny_networks = &[];
    let allow_networks = &[];
    let mut server = ServerFuture::with_access(mushroom, deny_networks, allow_networks);

    for bind in binds {
        match bind {
            Ok(bind) => {
                server.register_socket(bind)
            }
            Err(err) => {
                error!("{}", err);
            }
        }
    }

    info!("server starting up, awaiting connections...");
    if (sd_notify::booted().unwrap_or(false)) {
        sd_notify::notify(true, &[NotifyState::Ready]).unwrap();
    }
    match runtime.block_on(server.block_until_done()) {
        Ok(()) => {
            // we're exiting for some reason...
            info!("Hickory MushroomDNResolver {} stopping", "5");
        }
        Err(e) => {
            let error_msg = format!(
                "Hickory MushroomDNResolver {} has encountered an error: {}",
                "5",
                e
            );

            error!("{}", error_msg);
            panic!("{}", error_msg);
        }
    };

    Ok(())
}

/// Build a UdpSocket for a given IP, port pair; IPv6 sockets will not accept v4 connections
fn build_udp_socket(ip: IpAddr, port: u16) -> Result<UdpSocket, std::io::Error> {
    let sock = if ip.is_ipv4() {
        Socket::new(Domain::IPV4, Type::DGRAM, None)?
    } else {
        let s = Socket::new(Domain::IPV6, Type::DGRAM, None)?;
        s.set_only_v6(true)?;
        s
    };

    sock.set_nonblocking(true)?;

    let s_addr = SocketAddr::new(ip, port);
    sock.bind(&s_addr.into())?;

    UdpSocket::from_std(sock.into())
}

