# ryndns: Multi-provider Dynamic DNS (bunny.net support)

## Summary

Extend the existing `cloudflare-dyndns` tool to support bunny.net as a second DNS provider. The crate and binary are renamed to `ryndns`. The config format is a clean break. The architecture uses a `DnsProvider` trait so both providers share one `handle_zone` implementation.

---

## Naming

- Crate name: `cloudflare-dyndns` → `ryndns`
- Binary name: `cloudflare-dyndns` → `ryndns`
- No backward-compatible aliases.

---

## Module Structure

```
src/
  provider/
    mod.rs         ← DnsProvider trait + shared Zone, DnsRecord, DnsRecordType
    cloudflare.rs  ← CloudflareProvider implements DnsProvider
    bunny.rs       ← BunnyProvider implements DnsProvider
  cloudflare_api/  ← raw serde types + HTTP calls for Cloudflare (unchanged)
  bunny_api/       ← raw serde types + HTTP calls for Bunny
  config/          ← updated ApplicationConfig, ProviderConfig, ZoneConfig, RecordConfig
  ip_cache/        ← unchanged
  cli/
    dyndns.rs      ← updated main loop
    list_zones.rs  ← updated to list zones from all configured providers
    ...
  lib.rs
  state.rs         ← ApplicationState loses client and zone_name_to_id_map
```

---

## Shared Domain Types (`src/provider/mod.rs`)

```rust
pub struct Zone {
    pub id: String,
    pub name: String,
}

pub struct DnsRecord {
    pub id: String,
    pub name: String,
    pub record_type: DnsRecordType,
    pub content: String,
}

pub enum DnsRecordType { A, AAAA, MX, TXT, SRV, CNAME, MISC }
```

`DnsRecordType` moves here from `cloudflare_api/record.rs`. The raw Cloudflare serde types stay in `cloudflare_api/` and are mapped to these shared types inside `CloudflareProvider`. Bunny's raw types live in `bunny_api/` and map the same way.

---

## `DnsProvider` Trait

```rust
pub trait DnsProvider {
    async fn list_zones(&self) -> Result<Vec<Zone>>;
    async fn list_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>>;
    async fn update_record(&self, zone_id: &str, record_id: &str, ip: &str) -> Result<()>;
}
```

`CloudflareProvider` and `BunnyProvider` each hold a `reqwest::Client` built with the provider's token. Cloudflare uses `Authorization: Bearer {token}`; Bunny uses `AccessKey: {token}`. Each provider constructs its own client.

---

## Config Format (clean break)

```toml
public_ip_url = "https://example.ip"

[cloudflare]
token = "cf_token"

[cloudflare."example.com"]
records = ["example.com", "*", { type = "AAAA", name = "mail" }]

[bunny]
token = "bunny_token"

[bunny."example.nl"]
records = ["example.nl", "*"]
```

Rust structs:

```rust
struct ApplicationConfig {
    public_ip_url: String,
    cloudflare: Option<ProviderConfig>,
    bunny: Option<ProviderConfig>,
}

struct ProviderConfig {
    token: String,
    #[serde(flatten)]
    zones: IndexMap<String, ZoneConfig>,
}
```

`ZoneConfig` and `RecordConfig` are unchanged. The tool errors at startup if neither `[cloudflare]` nor `[bunny]` is present.

Default config file paths change as a consequence of the rename (derived from `PKG_NAME`):
- User: `~/.config/ryndns/ryndns.toml` (was `~/.config/cloudflare-dyndns/cloudflare-dyndns.toml`)
- Root: `/etc/ryndns/ryndns.toml` (was `/etc/cloudflare-dyndns/cloudflare-dyndns.toml`)

Users must move their config file when upgrading.

---

## Main Loop (`cli/dyndns.rs`)

The `zone_name_to_id_map` is built per-provider (each provider lists its own zones). `handle_zone` is generic over `T: DnsProvider`.

```rust
if let Some(cf_config) = config.cloudflare() {
    let provider = CloudflareProvider::new(cf_config.token())?;
    let zone_map = build_zone_map(&provider).await?;
    for (zone, zone_config) in cf_config.zones() {
        handle_zone(&provider, &zone_map, &mut state, zone, zone_config).await?;
    }
}
if let Some(bunny_config) = config.bunny() {
    let provider = BunnyProvider::new(bunny_config.token())?;
    let zone_map = build_zone_map(&provider).await?;
    for (zone, zone_config) in bunny_config.zones() {
        handle_zone(&provider, &zone_map, &mut state, zone, zone_config).await?;
    }
}
```

`ApplicationState` loses `client` and `zone_name_to_id_map`. It retains `ip_cache`, `ip_cache_path`, `config_path`, `public_ip_address`, `preview`, and `force`.

---

## `list_zones` Binary

Reads the config file and lists zones from all configured providers, labelled by provider:

```
cloudflare:
  example.com  (id: abc123)

bunny:
  example.nl   (id: 456)
```

No `--provider` flag. Lists everything present in the config.

---

## Error Handling

- Startup error if neither provider section is configured.
- Per-provider errors propagate as before (fail fast with `?`).
- No change to IP cache behaviour.

---

## Supporting Files

The following files are renamed and updated to reflect the new name and config format:

- `cloudflare-dyndns.example.toml` → `ryndns.example.toml` (updated to new provider-section config format)
- `cloudflare-dyndns.service` → `ryndns.service` (binary name updated)
- `cloudflare-dyndns.timer` → `ryndns.timer` (service name reference updated)
- `install.sh` updated to reference new file names

---

## Out of Scope

- IPv6 / AAAA record logic (unchanged from current implementation).
- systemd service/timer files (rename of binary is the only change needed there).
- Adding further providers beyond Cloudflare and bunny.net.
