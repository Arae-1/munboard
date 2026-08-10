---
title: Main Entry Point
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, entry, main, bootstrap]
sources: [main.js]
---

# Main Entry Point

The `main.js` file starts the client application. It initializes all core modules: configuration, live-reload, router, API, tags, pools, settings, and controllers.

## Startup Sequence

The startup sequence runs in these steps:

1. Load the auto-generated configuration from `./config.js`.
2. If the environment is `development`, connect a live-reload WebSocket on port 8081.
3. Load the polyfill module (`./util/polyfill.js`).
4. Import `misc`, `views`, and the `router`.
5. Set `history.scrollRestoration = "manual"`.
6. Register a router exit handler to save scroll position and confirm page exit.
7. Register a router enter handler to reset keyboard shortcuts via Mousetrap.
8. Apply the dark theme if the user setting `darkTheme` is enabled.
9. Call `api.fetchConfig()` to get the remote configuration.
10. On success, require and register all controllers on the router.
11. Re-apply the dark theme setting.
12. Set video autoplay based on the `autoplayVideos` setting.
13. Call `api.loginFromCookies()` to restore the user session from cookies.
14. On login success, refresh tag and pool category color maps, then start the router.
15. On login failure:
    - If the current URL contains "login", forget auth cookies and start the router.
    - Otherwise, navigate to "/" and show the login error.

## Registered Controllers

The file requires and registers these controllers on the router:

| Controller                          | Purpose                        |
|-------------------------------------|--------------------------------|
| `home_controller.js`                | Home page                      |
| `help_controller.js`                | Help page                      |
| `auth_controller.js`                | Login/logout                   |
| `password_reset_controller.js`      | Password reset                 |
| `comments_controller.js`            | Comments listing               |
| `snapshots_controller.js`           | Snapshots listing              |
| `post_detail_controller.js`         | Single post detail             |
| `post_main_controller.js`           | Post main content              |
| `post_list_controller.js`           | Post listing/search            |
| `post_upload_controller.js`         | Post upload                    |
| `tag_create_controller.js`          | Tag creation                   |
| `tag_controller.js`                 | Single tag view                |
| `tag_list_controller.js`            | Tag listing                    |
| `tag_categories_controller.js`      | Tag category management        |
| `pool_create_controller.js`         | Pool creation                  |
| `pool_controller.js`                | Single pool view               |
| `pool_list_controller.js`           | Pool listing                   |
| `pool_categories_controller.js`     | Pool category management       |
| `custom_field_create_controller.js` | Custom field creation          |
| `custom_field_controller.js`        | Single custom field view       |
| `custom_field_list_controller.js`   | Custom field listing           |
| `settings_controller.js`            | User settings                  |
| `user_controller.js`                | Single user view               |
| `user_list_controller.js`           | User listing                   |
| `user_registration_controller.js`   | User registration              |
| `not_found_controller.js`           | 404 (registered last)          |

## Dependencies

- `./config.js` — auto-generated configuration (build meta and environment).
- `./util/polyfill.js` — browser polyfills for NodeList, Promise, Number, Date.
- `./util/misc.js` — general utility functions.
- `./util/views.js` — DOM view helpers.
- `./router.js` — client-side router (singleton).
- `mousetrap` — keyboard shortcut library.
- `./tags.js` — tag category color map refresh.
- `./pools.js` — pool category color map refresh.
- `./api.js` — API client (singleton).
- `./models/settings.js` — user settings (singleton).

## Exports

This file does not export anything. It is the startup script.
