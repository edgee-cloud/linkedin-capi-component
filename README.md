<div align="center">
<p align="center">
  <a href="https://www.edgee.cloud">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://cdn.edgee.cloud/img/component-dark.svg">
      <img src="https://cdn.edgee.cloud/img/component.svg" height="100" alt="Edgee">
    </picture>
  </a>
</p>
</div>


<h1 align="center">LinkedIn CAPI Component for Edgee</h1>

This component implements the data collection protocol between [Edgee](https://www.edgee.cloud) and [LinkedIn CAPI](https://learn.microsoft.com/en-us/linkedin/marketing/integrations/ads-reporting/conversions-api).

## Quick Start

1. Download the latest component version from our [releases page](../../releases)
2. Place the `linkedin_capi.wasm` file in your server (e.g., `/var/edgee/components`)
3. Add the following configuration to your `edgee.toml`:

```toml
[[destinations.data_collection]]
name = "linkedin_capi"
component = "/var/edgee/components/linkedin_capi.wasm"
credentials.linkedin_access_token = "YOUR_ACCESS_TOKEN"
```

## Event Handling

### Event Mapping
The component maps Edgee events to LinkedIn CAPI events as follows:

| Edgee event | LinkedIn CAPI Event  | Description |
|-------------|-----------|-------------|
| Page   | NONE     | LinkedIn CAPI doesn't have Page event |
| Track  | URN of the conversion rule created through API. | Uses the provided conversion rule name directly |
| User   | NONE   | LinkedIn CAPI doesn't have User event |


Here is an example of a track call:
```javascript
edgee.track({
  name: "urn:lla:llaPartnerConversion:123",
});
```

## Configuration Options

### Basic Configuration
```toml
[[destinations.data_collection]]
name = "linkedin_capi"
component = "/var/edgee/components/linkedin_capi.wasm"
credentials.linkedin_access_token = "YOUR_ACCESS_TOKEN"

# Optional configurations
config.default_consent = "pending" # Set default consent status
```

### Event Controls
Control which events are forwarded to LinkedIn CAPI:
```toml
config.page_event_enabled = true   # Enable/disable page view tracking
config.track_event_enabled = true  # Enable/disable custom event tracking
config.user_event_enabled = true   # Enable/disable user identification
```

### Consent Management
Before sending events to LinkedIn CAPI, you can set the user consent using the Edgee SDK: 
```javascript
edgee.consent("granted");
```

Or using the Data Layer:
```html
<script id="__EDGEE_DATA_LAYER__" type="application/json">
  {
    "data_collection": {
      "consent": "granted"
    }
  }
</script>
```

If the consent is not set, the component will use the default consent status.
**Important:** LinkedIn CAPI requires the consent status to be set to `granted`. If not, the events will be ignored.

| Consent | Events |
|---------|--------|
| pending | ignored |
| denied  | ignored |
| granted | forwarded |

## Development

### Building from Source
Prerequisites:
- [Rust](https://www.rust-lang.org/tools/install)
- WASM target: `rustup target add wasm32-wasip2`
- wit-deps: `cargo install wit-deps`

Build command:
```bash
make wit-deps
make build
```

### Contributing
Interested in contributing? Read our [contribution guidelines](./CONTRIBUTING.md)

### Security
Report security vulnerabilities to [security@edgee.cloud](mailto:security@edgee.cloud)
```