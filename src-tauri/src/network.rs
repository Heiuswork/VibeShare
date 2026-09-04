use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct NetworkInfo {
    pub ip: String,
    pub interface: String,
    pub label: String,
    pub interfaces: Vec<NetworkInterface>,
    pub reachable: bool,
}

fn label_for(name: &str) -> String {
    match name {
        "en0" => "Wi-Fi".into(),
        "en1" => "以太网".into(),
        other => other.into(),
    }
}

fn is_lan_ipv4(ip: &std::net::Ipv4Addr) -> bool {
    !ip.is_loopback() && !ip.is_link_local() && !ip.is_multicast() && !ip.is_unspecified()
}

pub fn list_interfaces() -> Vec<NetworkInterface> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|iface| {
            let std::net::IpAddr::V4(ip) = iface.ip() else {
                return None;
            };
            if !is_lan_ipv4(&ip) {
                return None;
            }
            Some(NetworkInterface {
                label: label_for(&iface.name),
                name: iface.name,
                ip: ip.to_string(),
            })
        })
        .collect()
}

pub fn current_network(preferred_ip: Option<&str>) -> NetworkInfo {
    let interfaces = list_interfaces();
    let selected = preferred_ip
        .and_then(|ip| interfaces.iter().find(|item| item.ip == ip))
        .or_else(|| interfaces.first());

    match selected {
        Some(item) => NetworkInfo {
            ip: item.ip.clone(),
            interface: item.name.clone(),
            label: item.label.clone(),
            reachable: true,
            interfaces,
        },
        None => NetworkInfo {
            ip: "127.0.0.1".into(),
            interface: "".into(),
            label: "本机".into(),
            reachable: false,
            interfaces,
        },
    }
}
