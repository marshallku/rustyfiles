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
-   **Format Detection**: Automatically detect and handle various image formats (PNG, JPG, JPEG, GIF, WebP, SVG)
-   **Quality Optimization**: Serve optimized images based on request parameters

### Host-Based Routing

The CDN now supports serving content from multiple hosts with intelligent path parsing:

-   **URL-based requests**: `https://example.com/images/path/to/image.jpg`
-   **Absolute paths**: `/images/path/to/image.jpg`
-   **Relative paths**: `images/path/to/image.jpg`

## Project Structure

```
rustycdn/
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

#### Multi-Host Support

-   **Full URL**: `http://localhost:41890/images/https://example.com/images/logo.png`
-   **Host-based**: `http://localhost:41890/images/example.com/logo.png`

## Configuration

Environment variables for customization:

-   `BIND_ADDRESS`: Server bind address (default: `127.0.0.1`)
-   `PORT`: Server port (default: `41890`)
-   `HOST`: Default origin host (default: `http://localhost/`)

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
00 4 * * * /usr/bin/find /path/to/cdn_root -mindepth 2 -atime +365 -type f \( -iname \*.png -o -iname \*.jpg -o -iname \*.jpeg -o -iname \*.gif -o -iname \*.webp -o -iname \*.mp4 -o -iname \*.webm -o -iname \*.svg -o -iname \*.css -o -iname \*.js \) | xargs rm 1>/dev/null 2>/dev/null
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
