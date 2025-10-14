use crate::core::state::AppState;
use crate::core::types::{NetworkConfigMode, NetworkInterfaceConfig};

#[derive(Clone, Debug)]
pub struct NetworkPlan {
    pub commands: Vec<String>,
}

impl NetworkPlan {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

pub struct NetworkService;

impl NetworkService {
    /// Build network configuration plan based on the selected mode
    pub fn build_plan(state: &AppState) -> NetworkPlan {
        let mut cmds: Vec<String> = Vec::new();

        match state.network_mode_index {
            0 => {
                // Copy ISO network configuration
                Self::build_copy_iso_plan(state, &mut cmds);
            }
            1 => {
                // Manual configuration
                Self::build_manual_plan(state, &mut cmds);
            }
            2 => {
                // NetworkManager (already handled in sysconfig.rs)
                // No additional commands needed here
            }
            _ => {
                // Default to NetworkManager
            }
        }

        NetworkPlan::new(cmds)
    }

    /// Build plan for copying ISO network configuration
    fn build_copy_iso_plan(_state: &AppState, cmds: &mut Vec<String>) {
        // Helper: wrap a command to run inside the target system via arch-chroot
        fn chroot_cmd(inner: &str) -> String {
            let escaped = inner.replace("'", "'\\''");
            format!("arch-chroot /mnt bash -lc '{escaped}'")
        }

        // Enable systemd-networkd and systemd-resolved (default in Arch ISO)
        cmds.push("systemctl --root=/mnt enable systemd-networkd".into());
        cmds.push("systemctl --root=/mnt enable systemd-resolved".into());

        // Ensure networkd configuration directory exists on target
        cmds.push("install -d /mnt/etc/systemd/network".into());

        // Copy current network configuration from the ISO
        if let Ok(iso_configs) = Self::detect_iso_network_config() {
            for config in iso_configs {
                Self::create_networkd_config(&config, cmds);
            }
        } else {
            // Fallback: create a basic DHCP configuration for the first non-loopback interface
            if let Ok(interface) = Self::get_first_network_interface() {
                let fallback_config = NetworkInterfaceConfig {
                    interface,
                    mode: NetworkConfigMode::Dhcp,
                    ip_cidr: None,
                    gateway: None,
                    dns: None,
                };
                Self::create_networkd_config(&fallback_config, cmds);
            }
        }

        // Ensure resolv.conf is properly linked to systemd-resolved (idempotent)
        cmds.push(chroot_cmd(
            "target=/run/systemd/resolve/stub-resolv.conf; cur=$(readlink -f /etc/resolv.conf 2>/dev/null || true); if [ \"$cur\" = \"$target\" ]; then :; else rm -f /etc/resolv.conf && ln -s \"$target\" /etc/resolv.conf; fi"
        ));
    }

    /// Build plan for manual network configuration
    fn build_manual_plan(state: &AppState, cmds: &mut Vec<String>) {
        // Helper: wrap a command to run inside the target system via arch-chroot
        fn chroot_cmd(inner: &str) -> String {
            let escaped = inner.replace("'", "'\\''");
            format!("arch-chroot /mnt bash -lc '{escaped}'")
        }

        // Enable systemd-networkd and systemd-resolved
        cmds.push("systemctl --root=/mnt enable systemd-networkd".into());
        cmds.push("systemctl --root=/mnt enable systemd-resolved".into());

        // Ensure networkd configuration directory exists on target
        cmds.push("install -d /mnt/etc/systemd/network".into());

        // Create configuration files for each manually configured interface
        for config in &state.network_configs {
            Self::create_networkd_config(config, cmds);
        }

        // Ensure resolv.conf is properly linked to systemd-resolved (idempotent)
        cmds.push(chroot_cmd(
            "target=/run/systemd/resolve/stub-resolv.conf; cur=$(readlink -f /etc/resolv.conf 2>/dev/null || true); if [ \"$cur\" = \"$target\" ]; then :; else rm -f /etc/resolv.conf && ln -s \"$target\" /etc/resolv.conf; fi"
        ));
    }

    /// Detect current network configuration from the ISO environment
    fn detect_iso_network_config() -> Result<Vec<NetworkInterfaceConfig>, Box<dyn std::error::Error>>
    {
        let mut configs = Vec::new();

        // Get list of network interfaces (excluding loopback)
        let interfaces = Self::get_network_interfaces()?;

        for interface in interfaces {
            // Check if interface has an IP address
            if let Ok(ip_output) = std::process::Command::new("ip")
                .args(["addr", "show", &interface])
                .output()
            {
                let ip_text = String::from_utf8_lossy(&ip_output.stdout);

                // Check if interface has an IP address (not just link-local)
                let has_ip = ip_text.contains("inet ") && !ip_text.contains("127.0.0.1");

                if has_ip {
                    // Try to determine if it's DHCP or static
                    let mode = if Self::is_dhcp_configured(&interface)? {
                        NetworkConfigMode::Dhcp
                    } else {
                        NetworkConfigMode::Static
                    };

                    // Extract IP address and gateway if static
                    let (ip_cidr, gateway) = if matches!(mode, NetworkConfigMode::Static) {
                        let ip = Self::extract_ip_address(&ip_text)?;
                        let gw = Self::get_gateway(&interface)?;
                        (Some(ip), gw)
                    } else {
                        (None, None)
                    };

                    // Get DNS servers
                    let dns = Self::get_dns_servers()?;

                    configs.push(NetworkInterfaceConfig {
                        interface,
                        mode,
                        ip_cidr,
                        gateway,
                        dns,
                    });
                }
            }
        }

        Ok(configs)
    }

    /// Get list of network interfaces (excluding loopback)
    fn get_network_interfaces() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let output = std::process::Command::new("ip")
            .args(["-o", "link"])
            .output()?;

        let text = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();

        for line in text.lines() {
            if let Some(colon) = line.find(": ") {
                let rest = &line[colon + 2..];
                if let Some(name_end) = rest.find(":") {
                    let name = &rest[..name_end];
                    if name != "lo" {
                        interfaces.push(name.to_string());
                    }
                }
            }
        }

        Ok(interfaces)
    }

    /// Get the first non-loopback network interface
    fn get_first_network_interface() -> Result<String, Box<dyn std::error::Error>> {
        let interfaces = Self::get_network_interfaces()?;
        interfaces
            .into_iter()
            .next()
            .ok_or_else(|| "No network interfaces found".into())
    }

    /// Check if an interface is configured with DHCP
    fn is_dhcp_configured(interface: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if systemd-networkd is managing the interface
        let output = std::process::Command::new("systemctl")
            .args(["is-active", "systemd-networkd"])
            .output()?;

        if output.status.success() {
            // Check for DHCP configuration in systemd-networkd
            let config_path = "/etc/systemd/network/".to_string();
            if std::path::Path::new(&config_path).exists() {
                for entry in std::fs::read_dir(&config_path)? {
                    let entry = entry?;
                    if let Some(name) = entry.file_name().to_str()
                        && name.ends_with(".network")
                        && let Ok(content) = std::fs::read_to_string(entry.path())
                        && content.contains(&format!("Name={}", interface))
                        && content.contains("DHCP=yes")
                    {
                        return Ok(true);
                    }
                }
            }
        }

        // Check if dhcpcd is running
        let dhcpcd_output = std::process::Command::new("systemctl")
            .args(["is-active", "dhcpcd"])
            .output()?;

        if dhcpcd_output.status.success() {
            return Ok(true);
        }

        // Default to static if we can't determine
        Ok(false)
    }

    /// Extract IP address from ip addr output
    fn extract_ip_address(ip_text: &str) -> Result<String, Box<dyn std::error::Error>> {
        for line in ip_text.lines() {
            if line.contains("inet ") && !line.contains("127.0.0.1") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Ok(parts[1].to_string());
                }
            }
        }
        Err("No IP address found".into())
    }

    /// Get gateway for an interface
    fn get_gateway(interface: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let output = std::process::Command::new("ip")
            .args(["route", "show", "default", "dev", interface])
            .output()?;

        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("default via") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(via_idx) = parts.iter().position(|&x| x == "via")
                    && via_idx + 1 < parts.len()
                {
                    return Ok(Some(parts[via_idx + 1].to_string()));
                }
            }
        }
        Ok(None)
    }

    /// Get DNS servers from resolv.conf
    fn get_dns_servers() -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
            let mut dns_servers = Vec::new();
            for line in content.lines() {
                if line.starts_with("nameserver ") {
                    let server = line.strip_prefix("nameserver ").unwrap_or("");
                    if !server.is_empty() {
                        dns_servers.push(server);
                    }
                }
            }
            if !dns_servers.is_empty() {
                return Ok(Some(dns_servers.join(",")));
            }
        }
        Ok(None)
    }

    /// Create systemd-networkd configuration file for an interface
    fn create_networkd_config(config: &NetworkInterfaceConfig, cmds: &mut Vec<String>) {
        let config_content = match config.mode {
            NetworkConfigMode::Dhcp => {
                format!(
                    "[Match]\nName={}\n\n[Network]\nDHCP=yes\n",
                    config.interface
                )
            }
            NetworkConfigMode::Static => {
                let mut content = format!("[Match]\nName={}\n\n[Network]\n", config.interface);

                if let Some(ip_cidr) = &config.ip_cidr {
                    content.push_str(&format!("Address={}\n", ip_cidr));
                }

                if let Some(gateway) = &config.gateway {
                    content.push_str(&format!("Gateway={}\n", gateway));
                }

                if let Some(dns) = &config.dns {
                    for dns_server in dns.split(',') {
                        content.push_str(&format!("DNS={}\n", dns_server.trim()));
                    }
                }

                content
            }
        };

        let config_file = format!("/mnt/etc/systemd/network/20-{}.network", config.interface);
        cmds.push(format!(
            "printf '{}' > {}",
            config_content.replace('\n', "\\n"),
            config_file
        ));
    }
}
