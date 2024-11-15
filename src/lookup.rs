use crate::server::QueryType;
use hickory_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts};
use hickory_resolver::lookup::Lookup;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::proto::xfer::Protocol;
use hickory_resolver::{ResolveError, TokioResolver};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

pub(crate) async fn hickory_lookup(resolver: &TokioResolver, x0: &String, query_type: QueryType) -> Result<Lookup, ResolveError> {
    let mut resolver_opts = ResolverOpts::default();
    resolver_opts.try_tcp_on_error = false;

    let final_resolver = if x0.ends_with("nordvpn.com") {
        let mut resolver_config = ResolverConfig::new();
        resolver_config.add_name_server(NameServerConfig::new(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 53)), Protocol::Udp));
        println!("Forwarding {} lookup via 8.8.8.8:53 over udp", x0);
        &TokioResolver::tokio(resolver_config, resolver_opts)
    } else {
        resolver
    };
    final_resolver.lookup(x0, match query_type {
        QueryType::UNKNOWN(a) => RecordType::Unknown(a),
        QueryType::A => RecordType::A,
        QueryType::NS => RecordType::NS,
        QueryType::CNAME => RecordType::CNAME,
        QueryType::MX => RecordType::MX,
        QueryType::AAAA => RecordType::AAAA,
        QueryType::PTR => RecordType::PTR,
        QueryType::HTTPS => RecordType::HTTPS,
        QueryType::SRV => RecordType::SRV
    }).await
}