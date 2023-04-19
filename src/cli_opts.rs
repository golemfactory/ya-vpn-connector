use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct CliOptions {
    #[structopt(
        short = "w",
        long = "websocket-address",
        help = "Bind websocket address",
        default_value = "ws://host.docker.internal:7465/net-api/v2/vpn/net/dd45782a49374df98c9f6b94fd26702f/raw/from/192.168.8.1/to/192.168.8.7"
    )]
    pub websocket_address: String,

    #[structopt(
        short = "v",
        long = "vpn-network-addr",
        help = "Bind address to the vpn network",
        default_value = "192.168.8.1"
    )]
    pub vpn_network_addr: String,

    #[structopt(
        short = "m",
        long = "vpn-network-mask",
        help = "Vpn network mask",
        default_value = "255.255.255.0"
    )]
    pub vpn_network_mask: String,

    #[structopt(
        short = "i",
        long = "vpn-interface-name",
        help = "Name of the vpn interface",
        default_value = "vpn0"
    )]
    pub vpn_interface_name: String,

    #[structopt(
        short = "u",
        long = "vpn-mtu",
        help = "Mtu of the vpn interface",
        default_value = "1220"
    )]
    pub vpn_mtu: u16,

    #[structopt(
    long = "vpn-layer",
    help = "Name of the vpn interface. Tap support is only experimental",
    default_value = "tun",
    possible_values = &["tun", "tap"]
    )]
    pub vpn_layer: String,
}
