pub mod access;
pub mod authority;
pub mod error;
pub mod lookup;
pub mod server;
pub mod store;

use crate::authority::mushroom::Mushroom;
use crate::server::ServerFuture;
use hickory_resolver::config::*;
use hickory_resolver::TokioResolver;
use sd_notify::NotifyState;
use socket2::{Domain, Socket, Type};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::runtime;
use tracing::{error, info};

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

fn main() -> Result<(), String> {
    // Construct a new Resolver with default configuration options
    tracing_subscriber::fmt().init();
    let mut runtime = runtime::Builder::new_multi_thread();
    runtime.enable_all().thread_name("hickory-server-runtime");
    runtime.worker_threads(8);
    let runtime = runtime
        .build()
        .map_err(|err| format!("failed to initialize Tokio runtime: {err:?}"))?;

    let mut binds = vec![];
    let _guard = runtime.enter();
    binds.push(build_udp_socket(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        53,
    ));
    let ipv6_sock = build_udp_socket(
        IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
        53,
    );
    binds.push(ipv6_sock);

    let mut opts = ResolverOpts::default();
    opts.cache_size = 256;
    opts.timeout = Duration::from_secs(5);
    opts.try_tcp_on_error = false;
    opts.attempts = 1;
    opts.server_ordering_strategy = ServerOrderingStrategy::QueryStatistics;
    opts.num_concurrent_reqs = 2;
    opts.use_hosts_file = ResolveHosts::Always;
    let mut resolver_config = ResolverConfig::new();
    let mut ipv4_resolver_config = ResolverConfig::new();
    let config_groups = [
        NameServerConfigGroup::cloudflare_tls(),
        NameServerConfigGroup::cloudflare_https(),
        NameServerConfigGroup::quad9_tls(),
        NameServerConfigGroup::quad9_https(),
        NameServerConfigGroup::google_h3(),
    ];
    for config_group in config_groups {
        for name_server_cfg in config_group.iter() {
            if name_server_cfg.socket_addr.is_ipv4() {
                ipv4_resolver_config.add_name_server(name_server_cfg.clone());
                continue
            }
            resolver_config.add_name_server(name_server_cfg.clone());
        }
    }

    let resolver = TokioResolver::tokio(resolver_config, opts.clone());
    let ipv4_resolver = TokioResolver::tokio(ipv4_resolver_config, opts);
    let mushroom = Mushroom { resolver, ipv4_resolver };
    let deny_networks = &[];
    let allow_networks = &[];
    let mut server = ServerFuture::with_access(mushroom, deny_networks, allow_networks);

    for bind in binds {
        match bind {
            Ok(bind) => {
                info!("Bound {:?}", bind.local_addr().unwrap());
                server.register_socket(bind);
            },
            Err(err) => {
                error!("{}", err);
            }
        }
    }

    info!("server starting up, awaiting connections...");
    if sd_notify::booted().unwrap_or(false) {
        sd_notify::notify(true, &[NotifyState::Ready]).unwrap();
    }
    &server.register_watchdog_feeder();
    match runtime.block_on(server.block_until_done()) {
        Ok(()) => {
            // we're exiting for some reason...
            info!("Hickory MushroomDNResolver {} stopping", "5");
        }
        Err(e) => {
            let error_msg = format!(
                "Hickory MushroomDNResolver {} has encountered an error: {}",
                "5", e
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
