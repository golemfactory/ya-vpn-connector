use std::process::{Command, Stdio};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use rand::Rng;
use std::net::Ipv4Addr;
use std::str::FromStr;

pub struct IpTablesRule {
    table: String,
    rule: String,
}

fn iptables_remove_rule(rule: &IpTablesRule) -> std::io::Result<()> {
    let table = &rule.table;
    let rule = &rule.rule;
    match Command::new("iptables")
        .arg("-t")
        .arg(table)
        .arg("-D")
        .args(rule.split(' '))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map(|proc| proc.wait_with_output())
    {
        Ok(Ok(output)) => {
            if output.status.success() {
                log::info!("Deleted rule: iptables -t {table} -D {rule}");
                Ok(())
            } else {
                log::error!("Error deleting rule: iptables -t {table} -D {rule}");
                log::error!("{}", String::from_utf8_lossy(&output.stdout));
                log::error!("{}", String::from_utf8_lossy(&output.stderr));
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error deleting rule",
                ));
            }
        }
        Ok(Err(err)) => {
            log::error!("Error deleting rule: iptables -t {table} -A {rule} {err:?}");
            return Err(err);
        }
        Err(err) => {
            log::error!("Error deleting rule: iptables -t {table} -A {rule} {err:?}");
            return Err(err);
        }
    }
}

fn iptables_add_rule(rule: &IpTablesRule) -> std::io::Result<()> {
    let table = &rule.table;
    let rule = &rule.rule;
    match Command::new("iptables")
        .arg("-t")
        .arg(table)
        .arg("-C")
        .args(rule.split(" "))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|proc| proc.wait_with_output())
    {
        Ok(Ok(output)) => {
            if output.status.success() {
                log::info!("Rule already exists: iptables -t {table} -C {rule}");
                return Ok(());
            } else {
                //command returns error if rule does not exist
            }
        }
        Ok(Err(err)) => {
            log::error!("Error checking rule: iptables -t {table} -C {rule} {err:?}");
            return Err(err);
        }
        Err(err) => {
            log::error!("Error checking rule: iptables -t {table} -C {rule} {err:?}");
            return Err(err);
        }
    }

    match Command::new("iptables")
        .arg("-t")
        .arg(table)
        .arg("-A")
        .args(rule.split(" "))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map(|proc| proc.wait_with_output())
    {
        Ok(Ok(output)) => {
            if output.status.success() {
                log::info!("Added rule: iptables -t {table} -A {rule}");
                Ok(())
            } else {
                log::error!("Error adding rule: iptables -t {table} -A {rule}");
                log::error!("{}", String::from_utf8_lossy(&output.stdout));
                log::error!("{}", String::from_utf8_lossy(&output.stderr));
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error adding rule",
                ));
            }
        }
        Ok(Err(err)) => {
            log::error!("Error adding rule: iptables -t {table} -A {rule} {err:?}");
            return Err(err);
        }
        Err(err) => {
            log::error!("Error adding rule: iptables -t {table} -A {rule} {err:?}");
            return Err(err);
        }
    }
}

/// This function removes rules that can be deleted after removing vpn_interface
/// Not that at least one rule will be left
pub fn iptables_cleanup(rules: Vec<IpTablesRule>) -> std::io::Result<()> {
    for rule in rules {
        iptables_remove_rule(&rule)?;
    }
    Ok(())
}

/// This function creates routing between vpn interface and main interface
/// It returns a list of rules that should be deleted later
pub fn iptables_route_to_interface(
    main_interface: &str,
    vpn_interface: &str,
) -> std::io::Result<Vec<IpTablesRule>> {
    let rules_to_delete_later = vec![
        IpTablesRule {
            table: "filter".to_string(),
            rule: format!("FORWARD -i {vpn_interface} -o eth0 -j ACCEPT"),
        },
        IpTablesRule {
            table: "filter".to_string(),
            rule: format!("FORWARD -i {main_interface} -o {vpn_interface} -m state --state RELATED,ESTABLISHED -j ACCEPT"),
        }];

    let rules_to_add = vec![IpTablesRule {
        table: "nat".to_string(),
        rule: format!("POSTROUTING -o {main_interface} -j MASQUERADE"),
    }];
    for rule in rules_to_delete_later.iter().chain(rules_to_add.iter()) {
        iptables_add_rule(rule)?;
    }
    Ok(rules_to_delete_later)
}

pub struct SubnetIpv4Info {
    subnet: Ipv4Addr,
    mask: Ipv4Addr,
    node_ip: Ipv4Addr,
    interface_name: String,
}

fn generate_interface_subnet_and_name(ip_suffix: u8) -> std::io::Result<SubnetIpv4Info> {
    let mut rng = rand::thread_rng();
    const VPN_BASE: &str = "10.94";
    const VPN_INTERFACE_NAME_BASE: &str = "vpn_10_94";

    let network_interfaces = match NetworkInterface::show() {
        Ok(interfaces) => interfaces,
        Err(err) => {
            log::error!("Error getting network interfaces: {err:?}");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error getting network interfaces {err:?}",
            )));
        }
    };

    let mut tries = 0;
    let si = loop {
        tries += 1;

        if tries > 100 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not find free subnet",
            ));
        }

        //10.8.X.X and 10.66.X.X is used in openvpn
        //selected 10.94.X.X for our vpn
        let number = rng.gen_range(0..255);


        let device_name = format!("{VPN_INTERFACE_NAME_BASE}_{number}");
        if let Some(ni) = network_interfaces.iter().find(|ni| ni.name == device_name) {
            continue;
        }
        let subnet_part = format!(
            "{}.{}",
            VPN_BASE,
            number,
        );
        let mask = "255.255.255.0";
        let subnet = format!("{}.0", subnet_part);
        let node_ip = format!("{}.{}", subnet_part, ip_suffix);

        let si = SubnetIpv4Info {
            subnet: Ipv4Addr::from_str(&subnet).expect("Cannot fail, invalid ip address"),
            mask: Ipv4Addr::from_str(&mask).expect("Cannot fail, invalid ip address"),
            interface_name: device_name,
            node_ip: Ipv4Addr::from_str(&node_ip).expect("Cannot fail, invalid ip address")
        };
        break si
    };
    Ok(si)
}
