use crate::authority::mushroom::Mushroom;
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
use networkmanager::{Error, NetworkManager};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;
use sysctl::{CtlValue, Sysctl};
use tracing::{error, info, warn};

pub(crate) async fn hickory_lookup(
    mushroom: &Mushroom,
    x0: &String,
    record_type: RecordType,
) -> (Result<Lookup, ResolveError>, bool) {
    let mut resolver_opts = ResolverOpts::default();
    resolver_opts.try_tcp_on_error = false;

    let ipv6_support = is_ipv6_enabled();

    let final_resolver = if x0.ends_with("nordvpn.com.") {
        let mut resolver_config = ResolverConfig::new();
        try_adding_ns_from_dhcp(&mut resolver_config, false);

        if resolver_config.name_servers().is_empty() {
            for ns in NameServerConfigGroup::google().iter() {
                if ns.socket_addr.is_ipv4() {
                    resolver_config.add_name_server(ns.clone())
                }
            }
        }

        info!(
            "Forwarding {} lookup via {:?}",
            x0,
            resolver_config.name_servers()
        );

        let mut resolver_opts = ResolverOpts::default();
        resolver_opts.shuffle_dns_servers = true;
        resolver_opts.try_tcp_on_error = false;
        &TokioResolver::tokio(resolver_config, resolver_opts)
    } else {
        if ipv6_support {
            info!("Looking up {} via all nameservers", x0);
            &mushroom.resolver
        } else {
            info!("Looking up {} via ipv4 nameservers", x0);
            &mushroom.ipv4_resolver
        }
    };
    (final_resolver.lookup(x0, record_type).await, ipv6_support)
}

fn is_ipv6_enabled() -> bool {
    return false; // fuck nordvpn,
    let disabled: CtlValue = sysctl::Ctl::new("net.ipv6.conf.all.disable_ipv6")
        .map(|v|
            {
                let err = v.value();
                err.unwrap_or(CtlValue::String("1".to_string()))
            }
        )
        .unwrap_or(CtlValue::String("1".to_string()));
    disabled == CtlValue::String("0".to_string())

}

// #[test]
// fn test_ipv6() {
//     assert!(!is_ipv6_enabled());
// }

fn try_adding_ns_from_dhcp(resolver_config: &mut ResolverConfig, ipv6_support: bool) {
    let dbus_connection = Connection::new_system();

    if let Ok(dbus_connection) = dbus_connection {
        let nm = NetworkManager::new(&dbus_connection);

        for dev in nm.get_devices().unwrap_or_default() {
            match dev {
                Device::Ethernet(x) => {
                    let dhcp4_map = x.dhcp4_config().map(|it| it.options());
                    try_adding_dhcp4_ns(resolver_config, dhcp4_map);

                    if ipv6_support {
                        let dhcp6_map = x.dhcp6_config().map(|it| it.options());
                        try_adding_dhcp6_ns(resolver_config, dhcp6_map);
                    }
                }
                Device::WiFi(x) => {
                    let dhcp4_map = x.dhcp4_config().map(|it| it.options());
                    try_adding_dhcp4_ns(resolver_config, dhcp4_map);

                    if ipv6_support {
                        let dhcp6_map = x.dhcp6_config().map(|it| it.options());
                        try_adding_dhcp6_ns(resolver_config, dhcp6_map);
                    }
                }
                _ => {}
            }
        }
    }
}

fn try_adding_dhcp6_ns(
    resolver_config: &mut ResolverConfig,
    dhcp6_map: Result<Result<HashMap<String, Variant<Box<dyn RefArg>>>, Error>, Error>,
) {
    if let Ok(Ok(dhcp6_map)) = dhcp6_map {
        let ipv6_ns_opt = dhcp6_map.get("dhcp6_name_servers").map(|it| it.as_str());
        if let Some(Some(dhcp_ipv6s)) = ipv6_ns_opt {
            dhcp_ipv6s.split(" ")
                .filter_map(|ipv6_str| {
                    if let Ok(ipv6_ns) = Ipv6Addr::from_str(ipv6_str) {
                        Some(ipv6_ns)
                    } else {
                        warn!("Your dhcp server is cooked and supplied a garbage ipv6 address {ipv6_str}");
                        None
                    }
                }).for_each(|ipv6_ns| {
                resolver_config.add_name_server(NameServerConfig::new(
                    SocketAddr::V6(SocketAddrV6::new(ipv6_ns, 53, 0, 0)),
                    Protocol::Udp,
                ))
            });
        }
    }
}

fn try_adding_dhcp4_ns(
    resolver_config: &mut ResolverConfig,
    dhcp4_map: Result<Result<HashMap<String, Variant<Box<dyn RefArg>>>, Error>, Error>,
) {
    if let Ok(Ok(dhcp4_map)) = dhcp4_map {
        let ipv4_ns_opt = dhcp4_map.get("domain_name_servers").map(|it| it.as_str());
        if let Some(Some(dhcp_ipv4s)) = ipv4_ns_opt {
            dhcp_ipv4s.split(" ")
                .filter_map(|ipv4_str| {
                    if let Ok(ipv4_ns) = Ipv4Addr::from_str(ipv4_str) {
                        Some(ipv4_ns)
                    } else {
                        warn!("Your dhcp server is cooked and supplied a garbage ipv4 address {ipv4_str}");
                        None
                    }
                }).for_each(|ipv4_ns| {
                resolver_config.add_name_server(NameServerConfig::new(
                    SocketAddr::V4(SocketAddrV4::new(ipv4_ns, 53)),
                    Protocol::Udp,
                ))
            });
        }
    }
}
