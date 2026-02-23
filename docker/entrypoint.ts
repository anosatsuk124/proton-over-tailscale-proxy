import { $, type ShellOutput } from "bun";
import { Subprocess } from "bun";

// ProtonVPN + Tailscale Exit Node Entrypoint
// Routes all Tailscale client traffic through ProtonVPN via WireGuard

// --- Config ---

interface Config {
  proton: {
    privateKey: string;
    publicKey: string;
    endpoint: string;
    dns: string;
    address: string;
    allowedIps: string;
  };
  tailscale: {
    authKey: string;
    hostname: string;
    advertiseRoutes: string;
    acceptDns: boolean;
    ssh: boolean;
    userspaceNetworking: boolean;
  };
  killSwitch: boolean;
  healthCheckUrl: string;
  serveFrontend: boolean;
}

function stripNewlines(s: string): string {
  return s.replace(/[\n\r]/g, "");
}

function loadConfig(): Config {
  return {
    proton: {
      privateKey: stripNewlines(process.env.PROTON_WG_PRIVATE_KEY ?? ""),
      publicKey: stripNewlines(process.env.PROTON_WG_PUBLIC_KEY ?? ""),
      endpoint:
        process.env.PROTON_WG_ENDPOINT ?? "nl-free-01.protonvpn.net:51820",
      dns: process.env.PROTON_WG_DNS ?? "10.8.0.1",
      address: process.env.PROTON_WG_ADDRESS ?? "10.8.0.2/32",
      allowedIps: process.env.PROTON_WG_ALLOWED_IPS ?? "0.0.0.0/0,::/0",
    },
    tailscale: {
      authKey: stripNewlines(process.env.TAILSCALE_AUTH_KEY ?? ""),
      hostname: process.env.TAILSCALE_HOSTNAME ?? "proton-exit-node",
      advertiseRoutes: process.env.TAILSCALE_ADVERTISE_ROUTES ?? "",
      acceptDns: process.env.TAILSCALE_ACCEPT_DNS === "true",
      ssh: process.env.TAILSCALE_SSH !== "false",
      userspaceNetworking:
        process.env.TAILSCALE_USERSPACE_NETWORKING !== "false",
    },
    killSwitch: process.env.KILL_SWITCH !== "false",
    healthCheckUrl: process.env.HEALTH_CHECK_URL ?? "https://ipinfo.io",
    serveFrontend: process.env.TAILSCALE_SERVE_FRONTEND !== "false",
  };
}

function validateConfig(config: Config): void {
  log("Validating configuration...");

  if (!config.proton.privateKey) {
    error(
      "PROTON_WG_PRIVATE_KEY is required. Get it from your ProtonVPN WireGuard configuration."
    );
  }
  if (!config.proton.publicKey) {
    error(
      "PROTON_WG_PUBLIC_KEY is required. Get it from your ProtonVPN WireGuard configuration."
    );
  }
  if (!config.tailscale.authKey) {
    error(
      "TAILSCALE_AUTH_KEY is required. Generate one at https://login.tailscale.com/admin/settings/keys"
    );
  }

  log("Configuration validation passed");
}

// --- Logging ---

function log(msg: string): void {
  const ts = new Date().toISOString().replace("T", " ").replace(/\.\d+Z/, "");
  console.log(`[${ts}] ${msg}`);
}

function error(msg: string): never {
  log(`ERROR: ${msg}`);
  process.exit(1);
}

// --- DNS ---

async function resolveHost(host: string): Promise<string | null> {
  // Check if already an IP address
  if (/^\d+\.\d+\.\d+\.\d+$/.test(host)) {
    return host;
  }

  // Try getent first (most reliable), prefer IPv4
  const getent = await $`getent ahostsv4 ${host} 2>/dev/null`
    .nothrow()
    .quiet();
  if (getent.exitCode === 0) {
    const ip = getent.text().split("\n")[0]?.split(/\s+/)[0];
    if (ip) return ip;
  }

  // Fallback to nslookup
  const nslookup = await $`nslookup ${host} 2>/dev/null`.nothrow().quiet();
  if (nslookup.exitCode === 0) {
    const lines = nslookup.text().split("\n");
    for (const line of lines) {
      const match = line.match(/^Address:\s+(\S+)/);
      if (match && match[1]) return match[1];
    }
  }

  // Last resort: dig
  const dig = await $`dig +short ${host} 2>/dev/null`.nothrow().quiet();
  if (dig.exitCode === 0) {
    const lines = dig.text().split("\n");
    for (const line of lines) {
      if (/^\d+\./.test(line.trim())) return line.trim();
    }
  }

  return null;
}

// --- WireGuard ---

async function setupWireguard(config: Config): Promise<void> {
  log("Setting up WireGuard configuration...");

  // Validate key lengths (WireGuard keys are base64 encoded, should be ~44 chars)
  if (config.proton.privateKey.length < 40) {
    error(
      `PROTON_WG_PRIVATE_KEY appears to be invalid (length: ${config.proton.privateKey.length}, expected ~44 chars). Check your .env file.`
    );
  }
  if (config.proton.publicKey.length < 40) {
    error(
      `PROTON_WG_PUBLIC_KEY appears to be invalid (length: ${config.proton.publicKey.length}, expected ~44 chars). Check your .env file.`
    );
  }

  // Create config file for reference/debugging purposes
  log(`Creating WireGuard config with Address=${config.proton.address}`);

  const wgConf = `[Interface]
PrivateKey = ${config.proton.privateKey}
Address = ${config.proton.address}
DNS = ${config.proton.dns}

[Peer]
PublicKey = ${config.proton.publicKey}
AllowedIPs = ${config.proton.allowedIps}
Endpoint = ${config.proton.endpoint}
PersistentKeepalive = 25
`;

  await Bun.write("/etc/wireguard/wg0.conf", wgConf, { mode: 0o600 });
  log("WireGuard configuration saved to /etc/wireguard/wg0.conf (for reference only)");

  // Debug: Show config preview (hiding keys)
  log("Config file preview:");
  const previewLines = wgConf.split("\n").slice(0, 3);
  for (const line of previewLines) {
    log(`  ${line.replace(/PrivateKey = .*/, "PrivateKey = [HIDDEN]")}`);
  }
}

async function startWireguard(config: Config): Promise<void> {
  log("Starting WireGuard...");

  // Create WireGuard interface
  await $`ip link add wg0 type wireguard`.nothrow().quiet();

  // Write private key to a temporary file (bash process substitution alternative)
  const keyFile = "/tmp/wg-private-key";
  await Bun.write(keyFile, config.proton.privateKey, { mode: 0o600 });

  log("Setting WireGuard private key...");
  await $`wg set wg0 private-key ${keyFile}`;

  // Clean up key file
  await $`rm -f ${keyFile}`.nothrow().quiet();

  log("Adding WireGuard peer...");
  await $`wg set wg0 peer ${config.proton.publicKey} allowed-ips ${config.proton.allowedIps} endpoint ${config.proton.endpoint} persistent-keepalive 25`;

  // Bring up interface
  await $`ip link set up wg0`;

  // Add IP address
  log(`Adding IP address ${config.proton.address} to wg0...`);
  await $`ip address add ${config.proton.address} dev wg0`;

  // Resolve ProtonVPN endpoint hostname for routing
  const protonHost = config.proton.endpoint.split(":")[0];
  const protonEndpointIp = await resolveHost(protonHost);

  if (protonEndpointIp) {
    log(`Resolved ProtonVPN endpoint ${protonHost} -> ${protonEndpointIp}`);
    log(`Adding route to ${protonEndpointIp} through eth0...`);
    const addRoute =
      await $`ip route add ${protonEndpointIp}/32 via 172.20.0.1 dev eth0`
        .nothrow()
        .quiet();
    if (addRoute.exitCode !== 0) {
      const replaceRoute =
        await $`ip route replace ${protonEndpointIp}/32 via 172.20.0.1 dev eth0`
          .nothrow()
          .quiet();
      if (replaceRoute.exitCode !== 0) {
        log(`WARNING: Could not add route to ${protonEndpointIp}`);
      }
    }
  } else {
    log(`WARNING: Could not resolve ${protonHost} to an IP address`);
    log("Adding hostname-based route as fallback...");
    const addRoute =
      await $`ip route add ${protonHost} via 172.20.0.1 dev eth0`
        .nothrow()
        .quiet();
    if (addRoute.exitCode !== 0) {
      const replaceRoute =
        await $`ip route replace ${protonHost} via 172.20.0.1 dev eth0`
          .nothrow()
          .quiet();
      if (replaceRoute.exitCode !== 0) {
        log(`WARNING: Could not add route to ${protonHost}`);
      }
    }
  }

  // NOTE: Default route through wg0 is set later, after Tailscale connects.

  // Show WireGuard status for debugging
  log("WireGuard status:");
  const wgStatus = await $`wg show wg0`.nothrow().quiet();
  if (wgStatus.exitCode === 0) {
    const lines = wgStatus.text().split("\n").slice(0, 5);
    for (const line of lines) {
      log(`  ${line}`);
    }
  }

  log("WireGuard started successfully (default route NOT yet changed)");
}

async function stopWireguard(): Promise<void> {
  log("Stopping WireGuard...");
  await $`ip link del wg0`.nothrow().quiet();
}

// --- Networking ---

async function setupNetworking(): Promise<void> {
  log("Configuring networking for exit node functionality...");

  // Enable IP forwarding (immediate) - ignore errors as these may be read-only
  const sysctls: Array<[string, string, string]> = [
    ["/proc/sys/net/ipv4/ip_forward", "1", "ip_forward"],
    ["/proc/sys/net/ipv6/conf/all/forwarding", "1", "IPv6 forwarding"],
    ["/proc/sys/net/ipv4/conf/all/src_valid_mark", "1", "src_valid_mark"],
    ["/proc/sys/net/ipv4/conf/all/accept_redirects", "0", "accept_redirects"],
    ["/proc/sys/net/ipv4/conf/all/send_redirects", "0", "send_redirects"],
  ];

  for (const [path, value, name] of sysctls) {
    try {
      await Bun.write(path, value);
    } catch {
      log(`Note: ${name} sysctl is read-only (set via docker sysctls)`);
    }
  }

  // Configure sysctl.conf for persistence
  const sysctlConf = `
# Tailscale Exit Node Configuration
net.ipv4.ip_forward = 1
net.ipv6.conf.all.forwarding = 1
net.ipv4.conf.all.src_valid_mark = 1
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.conf.all.send_redirects = 0
`;
  const existing = await Bun.file("/etc/sysctl.conf").text().catch(() => "");
  await Bun.write("/etc/sysctl.conf", existing + sysctlConf);

  log("Networking configured for IP forwarding");
}

async function setupNat(): Promise<void> {
  log("Setting up NAT and masquerading for exit node...");

  // Only flush POSTROUTING, preserve Docker's internal DNS NAT rules
  await $`iptables -t nat -F POSTROUTING`.nothrow().quiet();

  // Enable masquerading for WireGuard interface
  await $`iptables -t nat -A POSTROUTING -o wg0 -j MASQUERADE`;

  // Also masquerade for tailscale0 interface if it exists
  await $`iptables -t nat -A POSTROUTING -o tailscale0 -j MASQUERADE`
    .nothrow()
    .quiet();

  log("NAT/masquerading configured");
}

async function applyKillSwitch(config: Config): Promise<void> {
  if (!config.killSwitch) {
    log("Kill switch disabled");
    await setupNat();
    return;
  }

  log("Applying kill switch rules...");

  // Flush existing filter rules (but NOT nat - Docker's internal DNS uses nat rules)
  await $`iptables -F`.nothrow().quiet();
  // Only flush user-defined nat chains
  await $`iptables -t nat -F POSTROUTING`.nothrow().quiet();

  // Default drop policy
  await $`iptables -P INPUT DROP`;
  await $`iptables -P FORWARD DROP`;
  await $`iptables -P OUTPUT DROP`;

  // Allow loopback
  await $`iptables -A INPUT -i lo -j ACCEPT`;
  await $`iptables -A OUTPUT -o lo -j ACCEPT`;

  // Allow established connections
  await $`iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT`;
  await $`iptables -A OUTPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT`;

  // Allow WireGuard traffic
  await $`iptables -A OUTPUT -p udp --dport 51820 -j ACCEPT`;

  // Allow Tailscale traffic (UDP and TCP for DERP relays)
  await $`iptables -A OUTPUT -p udp --dport 41641 -j ACCEPT`;
  await $`iptables -A OUTPUT -p udp --dport 3478 -j ACCEPT`;
  await $`iptables -A OUTPUT -p tcp --dport 443 -j ACCEPT`;

  // Allow incoming Tailscale connections
  await $`iptables -A INPUT -p udp --dport 41641 -j ACCEPT`;

  // Allow DNS
  await $`iptables -A OUTPUT -p udp --dport 53 -j ACCEPT`;
  await $`iptables -A OUTPUT -p tcp --dport 53 -j ACCEPT`;

  // Allow Docker internal DNS
  await $`iptables -A OUTPUT -d 127.0.0.11 -p udp --dport 53 -j ACCEPT`;
  await $`iptables -A OUTPUT -d 127.0.0.11 -p tcp --dport 53 -j ACCEPT`;
  await $`iptables -A INPUT -s 127.0.0.11 -p udp --sport 53 -j ACCEPT`;
  await $`iptables -A INPUT -s 127.0.0.11 -p tcp --sport 53 -j ACCEPT`;

  // Allow ICMP (ping)
  await $`iptables -A OUTPUT -p icmp -j ACCEPT`;
  await $`iptables -A INPUT -p icmp -j ACCEPT`;

  // Allow traffic through WireGuard interface
  await $`iptables -A INPUT -i wg0 -j ACCEPT`;
  await $`iptables -A OUTPUT -o wg0 -j ACCEPT`;
  await $`iptables -A FORWARD -i wg0 -j ACCEPT`;
  await $`iptables -A FORWARD -o wg0 -j ACCEPT`;

  // Allow traffic through Tailscale interface
  await $`iptables -A INPUT -i tailscale0 -j ACCEPT`;
  await $`iptables -A OUTPUT -o tailscale0 -j ACCEPT`;
  await $`iptables -A FORWARD -i tailscale0 -j ACCEPT`;
  await $`iptables -A FORWARD -o tailscale0 -j ACCEPT`;

  // Allow Tailscale control plane bypass routes through eth0
  await $`iptables -A OUTPUT -o eth0 -p tcp --dport 443 -j ACCEPT`;
  await $`iptables -A OUTPUT -o eth0 -p udp --dport 443 -j ACCEPT`;
  await $`iptables -A INPUT -i eth0 -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT`;

  // Re-apply NAT after flushing
  await setupNat();

  log("Kill switch applied");
}

// --- VPN Routing ---

async function activateVpnRouting(config: Config): Promise<void> {
  log("Activating VPN routing...");

  // Add routes for Tailscale control plane through eth0 (bypass ProtonVPN)
  const tsHosts = [
    "controlplane.tailscale.com",
    "log.tailscale.io",
    "login.tailscale.com",
  ];

  for (const host of tsHosts) {
    const ip = await resolveHost(host);
    if (ip) {
      log(`Adding bypass route for ${host} (${ip}) through eth0`);
      const add =
        await $`ip route add ${ip}/32 via 172.20.0.1 dev eth0`
          .nothrow()
          .quiet();
      if (add.exitCode !== 0) {
        await $`ip route replace ${ip}/32 via 172.20.0.1 dev eth0`
          .nothrow()
          .quiet();
      }
    } else {
      log(`WARNING: Could not resolve ${host}`);
    }
  }

  // Add bypass routes for all Tailscale DERP relay servers
  log("Adding bypass routes for Tailscale DERP servers...");
  const derpMapResult = await $`tailscale debug derp-map`.nothrow().quiet();

  if (derpMapResult.exitCode === 0) {
    const derpText = derpMapResult.text();
    const derpIps = [
      ...new Set(
        [...derpText.matchAll(/"IPv4"\s*:\s*"([0-9.]+)"/g)].map((m) => m[1])
      ),
    ];

    if (derpIps.length > 0) {
      for (const ip of derpIps) {
        const add =
          await $`ip route add ${ip}/32 via 172.20.0.1 dev eth0`
            .nothrow()
            .quiet();
        if (add.exitCode !== 0) {
          await $`ip route replace ${ip}/32 via 172.20.0.1 dev eth0`
            .nothrow()
            .quiet();
        }
      }
      log(`Added bypass routes for ${derpIps.length} DERP server IPs`);
    } else {
      await addFallbackDerpRoutes();
    }
  } else {
    log(
      "WARNING: Could not get DERP map, adding known DERP hostnames as fallback"
    );
    await addFallbackDerpRoutes();
  }

  // Replace default route: wg0 gets priority, eth0 becomes fallback
  log("Setting default route through WireGuard...");
  const routeOutput = await $`ip route`.nothrow().quiet();
  const routes = routeOutput.text();
  const defaultMatch = routes.match(
    /default via (\S+).*eth0/
  );

  if (defaultMatch) {
    const eth0Gw = defaultMatch[1];
    await $`ip route del default via ${eth0Gw} dev eth0`.nothrow().quiet();
    await $`ip route add default via ${eth0Gw} dev eth0 metric 100`
      .nothrow()
      .quiet();
  }

  // Add wg0 default route with low metric (highest priority)
  const addDefault = await $`ip route add default dev wg0 metric 10`
    .nothrow()
    .quiet();
  if (addDefault.exitCode !== 0) {
    await $`ip route replace default dev wg0 metric 10`;
  }

  log("VPN routing activated - traffic now flows through ProtonVPN");
}

async function addFallbackDerpRoutes(): Promise<void> {
  for (let i = 1; i <= 29; i++) {
    const derpIp = await resolveHost(`derp${i}.tailscale.com`);
    if (derpIp) {
      const add =
        await $`ip route add ${derpIp}/32 via 172.20.0.1 dev eth0`
          .nothrow()
          .quiet();
      if (add.exitCode !== 0) {
        await $`ip route replace ${derpIp}/32 via 172.20.0.1 dev eth0`
          .nothrow()
          .quiet();
      }
    }
  }
}

// --- Tailscale ---

async function startTailscale(config: Config): Promise<void> {
  log("Starting Tailscale with userspace networking...");

  // Build tailscaled arguments
  const tailscaledArgs = [
    "tailscaled",
    "--state=/var/lib/tailscale/tailscaled.state",
    "--socket=/var/run/tailscale/tailscaled.sock",
  ];

  if (config.tailscale.userspaceNetworking) {
    log("Using userspace networking mode (no TUN device required)");
    tailscaledArgs.push("--tun=userspace-networking");
  } else {
    log("Using kernel TUN mode");
  }

  // Start tailscaled daemon in background
  Bun.spawn(tailscaledArgs, {
    stdout: "inherit",
    stderr: "inherit",
  });

  // Wait for daemon to start
  await Bun.sleep(2_000);

  // Build tailscale up arguments
  const tsUpArgs = [
    "tailscale",
    "up",
    `--authkey=${config.tailscale.authKey}`,
    `--hostname=${config.tailscale.hostname}`,
    "--advertise-exit-node",
    `--accept-dns=${config.tailscale.acceptDns}`,
  ];

  if (config.tailscale.ssh) {
    tsUpArgs.push("--ssh");
  }

  if (config.tailscale.advertiseRoutes) {
    tsUpArgs.push(`--advertise-routes=${config.tailscale.advertiseRoutes}`);
  }

  // Bring up Tailscale
  await $`${tsUpArgs}`;

  log("Tailscale started successfully with exit node enabled");

  const tsIp = await $`tailscale ip -4`.nothrow().quiet();
  log(`Tailscale IP: ${tsIp.exitCode === 0 ? tsIp.text().trim() : "N/A"}`);

  // Display exit node status
  log("Exit node status:");
  const status = await $`tailscale status`.nothrow().quiet();
  if (status.exitCode === 0) {
    const lines = status.text().split("\n").slice(0, 5);
    for (const line of lines) {
      log(line);
    }
  } else {
    log("Status not available yet");
  }
}

async function stopTailscale(): Promise<void> {
  log("Stopping Tailscale...");
  await $`tailscale down`.nothrow().quiet();
  await $`pkill tailscaled`.nothrow().quiet();
}

async function waitForTailscaleConnection(): Promise<boolean> {
  log("Waiting for Tailscale to establish connection...");
  const maxAttempts = 30;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    const status = await $`tailscale status --json`.nothrow().quiet();
    if (status.exitCode === 0) {
      try {
        const json = JSON.parse(status.text());
        if (json.Self?.Online === true) {
          log("Tailscale is connected!");
          return true;
        }
      } catch {
        // JSON parse failed, try again
      }
    }
    log(`Waiting for Tailscale connection... (${attempt}/${maxAttempts})`);
    await Bun.sleep(2_000);
  }

  return false;
}

async function setupTailscaleServe(config: Config): Promise<void> {
  if (!config.serveFrontend) return;

  log("Configuring tailscale serve for frontend (HTTPS 443 -> localhost:80)...");
  await $`tailscale serve --bg --https=443 http://localhost:80`;
  log("Frontend is now accessible via Tailscale HTTPS (port 443)");
}

// --- Health Check ---

async function healthcheck(config: Config): Promise<boolean> {
  // Check if WireGuard interface exists and is up
  const wgCheck = await $`ip link show wg0`.nothrow().quiet();
  if (wgCheck.exitCode !== 0) {
    console.log("FAIL: WireGuard interface not found");
    return false;
  }

  // Check if Tailscale is running
  const tsCheck = await $`pgrep tailscaled`.nothrow().quiet();
  if (tsCheck.exitCode !== 0) {
    console.log("FAIL: Tailscale daemon not running");
    return false;
  }

  // Check if Tailscale is connected
  const tsStatus = await $`tailscale status --json`.nothrow().quiet();
  if (tsStatus.exitCode === 0) {
    try {
      const json = JSON.parse(tsStatus.text());
      if (json.Self?.Online !== true) {
        console.log("FAIL: Tailscale not connected");
        return false;
      }
    } catch {
      console.log("FAIL: Tailscale not connected");
      return false;
    }
  } else {
    console.log("FAIL: Tailscale not connected");
    return false;
  }

  // Check internet connectivity through VPN
  const curlCheck =
    await $`curl -s --max-time 10 -o /dev/null -w "%{http_code}" ${config.healthCheckUrl}`
      .nothrow()
      .quiet();
  if (curlCheck.exitCode !== 0 || curlCheck.text().trim() !== "200") {
    console.log("FAIL: Cannot reach internet through VPN");
    return false;
  }

  console.log("OK: All services healthy");
  return true;
}

// --- Service Monitor ---

async function monitorServices(config: Config): Promise<never> {
  while (true) {
    // Check WireGuard
    const wgCheck = await $`ip link show wg0`.nothrow().quiet();
    if (wgCheck.exitCode !== 0) {
      log("WireGuard interface down, attempting to restart...");
      await startWireguard(config);
    }

    // Check Tailscale
    const tsCheck = await $`pgrep tailscaled`.nothrow().quiet();
    if (tsCheck.exitCode !== 0) {
      log("Tailscale daemon down, attempting to restart...");
      await startTailscale(config);
    }

    await Bun.sleep(30_000);
  }
}

// --- Cleanup ---

async function cleanup(config: Config): Promise<void> {
  log("Received shutdown signal, cleaning up...");
  await stopTailscale();
  await stopWireguard();
  log("Cleanup complete");
  process.exit(0);
}

// --- Main ---

async function main(): Promise<void> {
  const config = loadConfig();

  // Set up signal handlers
  process.on("SIGTERM", () => cleanup(config));
  process.on("SIGINT", () => cleanup(config));

  // Check if healthcheck mode
  if (process.argv[2] === "healthcheck") {
    const ok = await healthcheck(config);
    process.exit(ok ? 0 : 1);
  }

  log("Starting ProtonVPN + Tailscale Exit Node...");
  log(
    "Traffic flow: Tailscale clients -> Tailscale daemon -> WireGuard -> ProtonVPN"
  );

  // Validate and setup
  validateConfig(config);
  await setupWireguard(config);
  await setupNetworking();

  // Start services WITHOUT kill switch and WITHOUT changing default route
  // This allows Tailscale to reach its control plane via direct internet
  log("Starting services (default route stays on eth0 until Tailscale connects)...");
  await startWireguard(config);
  await startTailscale(config);

  // Wait for Tailscale to be ready
  const connected = await waitForTailscaleConnection();

  if (connected) {
    // Tailscale is connected - safe to route traffic through ProtonVPN
    await activateVpnRouting(config);

    // Configure tailscale serve for frontend HTTPS access
    await setupTailscaleServe(config);

    if (config.killSwitch) {
      log("Services are ready, applying kill switch...");
      await applyKillSwitch(config);
    }
  } else {
    log("WARNING: Tailscale failed to connect within timeout");
    log("NOT activating VPN routing or kill switch to allow debugging");
    log("Check your .env configuration and network connectivity");
  }

  log("All services started successfully!");
  log(
    "This node is now advertising as an exit node on your Tailscale network"
  );
  log(
    "Enable it in Tailscale admin panel: https://login.tailscale.com/admin/machines"
  );

  // Keep running and monitor services
  await monitorServices(config);
}

main().catch((err) => {
  error(`Unhandled error: ${err}`);
});
