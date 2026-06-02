# Rusty Files

[![CI](https://github.com/marshallku/rustyfiles/actions/workflows/ci.yml/badge.svg)](https://github.com/marshallku/rustyfiles/actions/workflows/ci.yml)

<!-- ![Quality Gate Status](https://badge.marshallku.dev?metric=alert_status&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Bugs](https://badge.marshallku.dev?metric=bugs&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Code Smells](https://badge.marshallku.dev?metric=code_smells&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Maintainability Rating](https://badge.marshallku.dev?metric=sqale_rating&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Reliability Rating](https://badge.marshallku.dev?metric=reliability_rating&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Security Rating](https://badge.marshallku.dev?metric=security_rating&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Vulnerabilities](https://badge.marshallku.dev?metric=vulnerabilities&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Coverage](https://badge.marshallku.dev?metric=coverage&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c) -->

A high-performance static file server built with Rust, designed to efficiently serve static files and images with advanced caching and optimization features.

## Features

### Core Functionality

-   **Static File Serving**: Serves CSS, JS, and other static files with intelligent caching
-   **Dynamic Image Processing**: On-demand image resizing and WebP conversion
-   **Multi-Host Support**: Serve content from multiple origin hosts with automatic host detection
-   **Smart Caching**: Fetches files from origin servers and caches them locally for improved performance

### Image Processing Capabilities

-   **Automatic Resizing**: Resize images on-the-fly using width parameters (e.g., `w100`, `w300`)
-   **WebP Conversion**: Convert images to WebP format for better compression
-   **AVIF Conversion**: Convert images to AVIF format for even better compression and quality
-   **Format Detection**: Automatically detect and handle various image formats (PNG, JPG, JPEG, GIF, WebP, AVIF, SVG)
-   **Quality Optimization**: Serve optimized images based on request parameters

### Host-Based Routing

The CDN now supports serving content from multiple hosts with intelligent path parsing:

-   **URL-based requests**: `https://example.com/images/path/to/image.jpg`
-   **Absolute paths**: `/images/path/to/image.jpg`
-   **Relative paths**: `images/path/to/image.jpg`

## Project Structure

```
/
├── src/
│   ├── controllers/     # Request handlers
│   ├── services/        # Business logic
│   ├── utils/           # Utility functions
│   ├── env/             # Environment configuration
│   └── constants/       # Application constants
├── cdn_root/            # Root directory for cached files, customizable by environment variable `CDN_ROOT`
│   ├── files/           # Cached static files
│   └── images/          # Cached and processed images
├── config/
│   └── nginx.conf       # Nginx configuration example
└── docker-compose.yml   # Docker deployment
```

## Prerequisites

-   Rust (latest stable)
-   Docker (for containerized deployment)
-   Basic understanding of Nginx (for production setup)

### Additional packages

```bash
sudo apt install pkg-config libssl-dev
```

The `reqwest` library requires `pkg-config` and `libssl-dev` packages for HTTP client functionality.

## Usage

### Starting the Server

```bash
cargo run
```

The server will start listening on the configured address (default: `127.0.0.1:41890`).

### Accessing Content

#### Static Files

-   **Endpoint**: `/files/*path`
-   **Example**: `http://localhost:41890/files/css/style.css`

#### Images

-   **Endpoint**: `/images/*path`
-   **Resize**: `http://localhost:41890/images/w100/photo.jpg` (resizes to 100px width)
-   **WebP conversion**: `http://localhost:41890/images/photo.jpg.webp`
-   **AVIF conversion**: `http://localhost:41890/images/photo.jpg.avif`

#### Multi-Host Support

-   **Full URL**: `http://localhost:41890/images/https://example.com/images/logo.png`
-   **Host-based**: `http://localhost:41890/images/example.com/logo.png`

## Configuration

Environment variables for customization:

-   `BIND_ADDRESS`: Server bind address (default: `127.0.0.1`)
-   `PORT`: Server port (default: `41890`)
-   `HOST`: Default origin host (default: `http://localhost/`)
-   `ALLOWED_HOSTS`: Comma-separated list of allowed hosts for whitelisting (e.g., `example.com,cdn.example.org`). If empty or not set, all hosts are allowed

### Storage backends

Rusty Files can cache fetched files either on the local filesystem or in an S3-compatible object store (AWS S3, Cloudflare R2, MinIO, etc.). The backend is selected at startup via `STORAGE_BACKEND` and cannot be mixed — all reads and writes go through a single backend.

-   `STORAGE_BACKEND`: `local` (default) or `s3` (alias: `r2`)

When `STORAGE_BACKEND=s3`:

-   `S3_ENDPOINT`: Custom endpoint URL. Leave empty for AWS S3. For Cloudflare R2 use `https://<account_id>.r2.cloudflarestorage.com`.
-   `S3_REGION`: AWS region (e.g. `us-east-1`) or `auto` for R2.
-   `S3_BUCKET`: Bucket name. Required.
-   `S3_ACCESS_KEY_ID` / `S3_SECRET_ACCESS_KEY`: Credentials. Required.
-   `S3_FORCE_PATH_STYLE`: `true` (default) uses path-style URLs (`endpoint/bucket/key`), required for R2 and most S3-compatible services. Set to `false` to use AWS virtual-hosted style.

Both S3 and R2 use the same protocol — AWS Signature V4 over HTTPS. R2 is drop-in S3-compatible, so only the endpoint and region differ.

### Origin mode

`ORIGIN_MODE` controls where originals come from when they are not already in storage:

-   `remote` (default): on a miss, the original is fetched from `HOST` over HTTP and cached in storage (read-through cache — storage is just a cache, the upstream site is the source of truth).
-   `bucket`: storage **is** the source of truth. Originals are expected to be uploaded out-of-band, so a miss returns `404` instead of an upstream fetch. Derived images (resize/WebP/AVIF) are still generated on demand and written back to storage.

This is independent of `STORAGE_BACKEND` — `bucket` mode is most useful with `s3`, where originals are uploaded directly to the bucket and Rusty Files reads them, generates derivatives, and stores those derivatives alongside.

#### Object key layout

The request path maps to a storage object key. The host segment is only used in `remote` mode, where it disambiguates objects cached from different upstreams:

| Mode | Key for `GET /images/photo.png` | Key for `GET /files/doc.pdf` |
| --- | --- | --- |
| `remote` | `images/<host>/photo.png` | `files/<host>/doc.pdf` |
| `bucket` | `images/photo.png` | `files/doc.pdf` |

(`<host>` is the request URL's host, or the `HOST` domain when none is given.) In `bucket` mode, upload your originals under `images/<path>` / `files/<path>`. Derived images are keyed the same way with the size/format suffix preserved (e.g. `images/photo.w300.png`), so the original for `images/photo.w300.png.webp` is read from `images/photo.png`.

## Production Deployment

### Docker Deployment

```bash
docker-compose up -d
```

### Nginx Reverse Proxy

For optimal performance in production, deploy behind Nginx using the provided configuration in `config/nginx.conf`.

## File Management

### Automatic Cleanup

Set up cron jobs to automatically clean up unused files:

```bash
# Remove CSS/JS files older than 5 days
00 4 * * * /usr/bin/find /path/to/cdn_root -mindepth 2 -atime +5 -type f \( -o -iname \*.css -o -iname \*.js \) | xargs rm 1>/dev/null 2>/dev/null

# Remove media files older than 1 year
00 4 * * * /usr/bin/find /path/to/cdn_root -mindepth 2 -atime +365 -type f \( -iname \*.png -o -iname \*.jpg -o -iname \*.jpeg -o -iname \*.gif -o -iname \*.webp -o -iname \*.avif -o -iname \*.mp4 -o -iname \*.webm -o -iname \*.svg -o -iname \*.css -o -iname \*.js \) | xargs rm 1>/dev/null 2>/dev/null
```

## Development

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

## License

This project is licensed under the MIT License.
