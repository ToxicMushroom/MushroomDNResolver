use dbus::arg::{RefArg, Variant};
use dbus::blocking::Connection;
use hickory_resolver::config::{
    NameServerConfig, NameServerConfigGroup, ResolverConfig, ResolverOpts,
};
use hickory_resolver::lookup::Lookup;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::proto::xfer::Protocol;
use hickory_resolver::{ResolveError, TokioResolver};
use networkmanager::devices::{Any, Device, Wired};
use networkmanager::NetworkManager;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;
use tracing::{error, info, warn};

pub(crate) async fn hickory_lookup(
    resolver: &TokioResolver,
    x0: &String,
    record_type: RecordType,
) -> Result<Lookup, ResolveError> {
    let mut resolver_opts = ResolverOpts::default();
    resolver_opts.try_tcp_on_error = false;

    let final_resolver = if x0.ends_with("nordvpn.com.") {
        let mut resolver_config = ResolverConfig::new();
        try_adding_ns_from_dhcp(&mut resolver_config);

        if resolver_config.name_servers().is_empty() {
            for ns in NameServerConfigGroup::google().iter() {
                resolver_config.add_name_server(ns.clone())
            }
        }

        info!(
            "Forwarding {} lookup via {:?}",
            x0,
            resolver_config.name_servers()
        );
        &TokioResolver::tokio(resolver_config, resolver_opts)
    } else {
        resolver
    };
    final_resolver.lookup(x0, record_type).await
}

fn try_adding_ns_from_dhcp(resolver_config: &mut ResolverConfig) {
    let dbus_connection = Connection::new_system();

    if let Ok(dbus_connection) = dbus_connection {
        let nm = NetworkManager::new(&dbus_connection);

        for dev in nm.get_devices().unwrap_or_default() {
            match dev {
                Device::Ethernet(x) => {
                    let dhcp4_map = x.dhcp4_config().unwrap().options().unwrap();
                    try_adding_dhcp4_ns(resolver_config, dhcp4_map);

                    let dhcp6_map = x.dhcp6_config().unwrap().options().unwrap();
                    try_adding_dhcp6_ns(resolver_config, &dhcp6_map);
                }
                Device::WiFi(x) => {
                    let dhcp4_map = x.dhcp4_config().unwrap().options().unwrap();
                    try_adding_dhcp4_ns(resolver_config, dhcp4_map);

                    let dhcp6_map = x.dhcp6_config().unwrap().options().unwrap();
                    try_adding_dhcp6_ns(resolver_config, &dhcp6_map);
                }
                _ => {}
            }
        }
    }
}

fn try_adding_dhcp6_ns(
    resolver_config: &mut ResolverConfig,
    dhcp6_map: &HashMap<String, Variant<Box<dyn RefArg>>>,
) {
    let ipv6_ns_opt = dhcp6_map.get("dhcp6_name_servers");
    if let Some(variant) = ipv6_ns_opt {
        let ipv6_ns = Ipv6Addr::from_str(variant.as_str().unwrap());
        if let Ok(ipv6_ns) = ipv6_ns {
            resolver_config.add_name_server(NameServerConfig::new(
                SocketAddr::V6(SocketAddrV6::new(ipv6_ns, 53, 0, 0)),
                Protocol::Udp,
            ))
        } else {
            warn!("Your dhcp server is cooked and supplied a garbage ipv6 address");
        }
    }
}

fn try_adding_dhcp4_ns(
    resolver_config: &mut ResolverConfig,
    dhcp4_map: HashMap<String, Variant<Box<dyn RefArg>>>,
) {
    let ipv4_ns_opt = dhcp4_map.get("domain_name_servers");
    if let Some(variant) = ipv4_ns_opt {
        let ipv4_ns = Ipv4Addr::from_str(variant.as_str().unwrap());
        if let Ok(ipv4_ns) = ipv4_ns {
            resolver_config.add_name_server(NameServerConfig::new(
                SocketAddr::V4(SocketAddrV4::new(ipv4_ns, 53)),
                Protocol::Udp,
            ))
        } else {
            warn!("Your dhcp server is cooked and supplied a garbage ipv4 address");
        }
    }
}
