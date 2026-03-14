# ip.nosqd.dev

> A blazingly fast IP information service written in Rust.
Inspired by ipinfo.io, but designed to be minimal, dependency-light, and easily self-hostable via Nix.

## Usage

### Browser
Simply visit `https://ip.nosqd.dev` in your browser.

### CLI (JSON)
```bash
$ curl ip.nosqd.dev
{
  "ip": "1.1.1.1",
  "city": "Brisbane",
  "country": "Australia",
  "asn": 13335,
  "asn_org": "CLOUDFLARENET",
  "flag": "🇦🇺"
}
```

## Self-Hosting

### Using docker
```
docker pull ghcr.io/nosqd/ip.nosqd.dev:latest
docker run -p 3000:3000 ghcr.io/nosqd/ip.nosqd.dev:latest
```

### Using Nix
```bash
nix run github:nosqd/ip-nosqd
```

### Environment Variables
- `PORT`: Port to listen on (default: 3000)
- `CITY_DB_PATH`: Path to `GeoLite2-City.mmdb` (if you are using docker container, that will be packaged automatically)
- `ASN_DB_PATH`: Path to `GeoLite2-ASN.mmdb` (if you are using docker container, that will be packaged automatically)

## Acknowledgements
- Data provided by [MaxMind](https://www.maxmind.com) (GeoLite2), fetched from [github:P3TERX/GeoLite.mmdb](https://github.com/P3TERX/GeoLite.mmdb)
- Built with [Axum](https://github.com/tokio-rs/axum) and [Crane](https://github.com/ipetkov/crane)
