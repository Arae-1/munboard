# Install Oxibooru

## Prerequisites

This guide assumes that you have Docker (version 19.03 or greater) and the Docker Compose CLI (version 1.27.0 or greater) already installed.

## Installing

1. **Download the `oxibooru` source**

    To get started either clone the repository with

    ```sh
    git clone https://github.com/liamw1/oxibooru
    ```

    or download the latest release from the [releases page](https://github.com/liamw1/oxibooru/releases). By default, a freshly cloned repo will use the `latest` images, whereas a release package will be fixed to a particular version. **Be warned that by using `latest` images, you are opting out of stability**. See [Versioning](#versioning) for details.

    Enter the `oxibooru` directory:

    ```sh
    cd oxibooru
    ```

2. **Configure the application**

    ```sh
    cp server/config.toml.dist server/config.toml
    edit server/config.toml
    ```
    It's *strongly recommended* to at least change these fields:

    - password_secret
    - content_secret
    - token_encryption_secret

    The server refuses to start when `password_secret`, `content_secret`, or
    `token_encryption_secret` is empty or still set to the placeholder value
    `"change"`. `token_encryption_secret` must be 64 hexadecimal characters
    (32 bytes). It is the AES-256-GCM key that the server uses to encrypt user
    tokens at rest.

    Any fields not present will default to their corresponding value in the original `config.toml.dist`, so feel free to remove fields that are uneeded or irrelevant. The three secrets above are an exception: keep them in your config and set each to a unique value.

3. **Configure Docker Compose**

    ```sh
    cp example.env .env
    edit .env
    ```

    Change the values of the variables in `.env` as needed. Read the comments to guide you. Note that `.env` should be in the root directory of this repository.

4. **Pull the containers**
    
    This pulls the latest containers from docker.io:

    ```sh
    docker compose pull
    ```

    If you have modified the application's source and would like to manually build it, follow the instructions in [Building](#building) instead, then read here once you're done.

5. **Give mount directories permissions**

    Set owner of mount directories (MOUNT_DATA and MOUNT_SQL in the .env) to the user with id 1000:

    ```sh
    sudo chown -R 1000:1000 "$MOUNT_DATA"
    sudo chown -R 1000:1000 "$MOUNT_SQL"
    ```

6. **Run it!**

    To start all containers:

    ```sh
    docker compose up -d
    ```

    To view/monitor the application logs:

    ```sh
    docker compose logs -f
    # (CTRL+C to exit)
    ```

## Configuration

The server reads `server/config.toml`. The important settings are:

| Key | Default | Description |
| --- | ------- | ----------- |
| `password_secret` | `"change"` (refused) | Salt for the users' password hashes. The server refuses to start when this is empty or `"change"`. |
| `content_secret` | `"change"` (refused) | Secret used to generate filenames for static content. The server refuses to start when this is empty or `"change"`. |
| `token_encryption_secret` | `"change"` (refused) | AES-256-GCM key used to encrypt user tokens at rest. Must be 64 hexadecimal characters (32 bytes). The server refuses to start when this is empty, `"change"`, or not 64 hexadecimal characters. |
| `max_upload_size` | `4294967296` (4 GiB) | Maximum size in bytes of files uploaded directly via multipart. |
| `max_url_download_size` | `4294967296` (4 GiB) | Maximum size in bytes of files downloaded from a URL. |
| `rate_limit_max_failures` | `20` | Maximum number of failed basic-auth attempts per username within `rate_limit_window_secs` before further attempts are rejected with `429`. |
| `rate_limit_window_secs` | `60` | Length of the failed-login sliding window, in seconds. |
| `secure_cookie` | `true` | Set the auth cookie's `secure` flag so the cookie is only sent over HTTPS. |
| `password_regex` | `^.{8,}$` | Regular expression that a new password must match. Set under `[public_info]`. |

## Reverse proxy (nginx)

The example configuration in `client/nginx.conf.docker` applies the following security defaults. The server itself does not serve HTTP; an nginx (or similar) reverse proxy in front of it must implement them.

- Security headers are sent on every response, including error responses: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`, a starting-point `Content-Security-Policy`, and `Strict-Transport-Security` when the request arrived over HTTPS.
- The API documentation at `/docs` and `/apidoc` is private by default. Only the loopback interface and private network ranges (`127.0.0.1`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`) may access it; all other clients receive `403`.
- Requests to `/data/temporary-uploads/` always receive `403`. The temporary upload staging files are internal only and must never be public.
- CORS is same-origin only. nginx mirrors the request's `Origin` header back only when the origin matches the `Host` header the browser used. There is no wildcard `Access-Control-Allow-Origin`. The bundled client is served from the same origin and does not need the header.

## Versioning

Oxibooru uses a semantic versioning system to distinguish between backwards compatible and backwards incompatbile versions. An increase in the leading non-zero version number means that the change is breaking, meaning that once you upgrade it may be difficult or impossible to revert to a previous version (typically due to changes in the database schema or data store). For example, it's fine to swap back and forth between versions `0.6.0` and `0.6.2`, but an upgrade from `0.6.2` and `0.7.0` is irreversible. A breaking change may also necesitate downtime due to expensive database migrations.

To change the version of an image, edit the image field in `docker-compose.yml`. The field follows the format `oxibooru/[server|client]:<version>`. There are a few different options for image versioning depending on your appetite for volatility. For maximum stability, you can use the fixed version images, e.g. `0.6.1`. These are never modified after creation, so you can pretty much guarantee that their behavior will stay the same forever. If you like being on the cutting edge, use the `latest` images. They are frequently (but not recklessly) updated with the latest fixes and features, but may introduce breaking changes or time-consuming migrations at any time. For a middle ground, there's also the major and minor version images, e.g. `0` and `0.7`, which are updated with the latest changes up to that major or minor version number.

## Building

To build the client and server containers, run

```sh
docker compose build
```

*Note: If your changes are not taking effect in your builds, consider building
with `--no-cache`.*

#### Performance tip

If you're already building yourself, consider setting `TARGET_CPU` to `native` in the `docker-compose.yml`. This instructions the Rust compiler to target your exact CPU architecture, resulting in better codegen and may result in measurable performance improvements to image decoding and reverse search operations.

## OpenGraph embeds

Oxibooru serves an OpenGraph embed page for each post link (`/post/{id}`). Chat applications such as Discord and Telegram use these pages to render a rich preview when a post link is shared.

To enable this feature, the deployment's nginx must proxy `^/post/[0-9]+/?$` to the backend route `/embed/post/{id}`. See `client/nginx.conf.docker` for an example.

Set `domain` in `config.toml` to the site's absolute URL. The embed page uses this value to generate absolute URLs for the post and its media. The `enable_embeds` option can disable the feature entirely.

The embed appearance is configured under `[embed]` in `config.toml`. The `site_name`, `title`, and `description` entries are text templates. Each template can contain placeholders in any order. The valid placeholders are `{site_name}`, `{post_id}`, `{width}`, `{height}`, `{dimensions}`, `{type}`, `{file_size}`, `{creation_date}`, `{launch_date}`, `{uploader}`, `{tags}`, `{tags:<category>}`, and `{source}`. The `image` entry selects the card preview: `post` (full image for image posts, thumbnail otherwise), `thumbnail`, `logo`, or `none`. Set `logo_url` to use a custom logo as the site icon and card image. The server does not start when a template contains an invalid placeholder or when `image` has an invalid value.