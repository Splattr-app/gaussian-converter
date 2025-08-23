# SG Flux Desktop

A desktop application for converting between different Gaussian Splatting formats.
This is built with [Tauri](https://tauri.app/) and [Nuxt](https://nuxt.com/).

## Setup

Make sure to install dependencies:

```bash
# pnpm
pnpm install
```

## Development Mode

Start the development app with

```bash
# pnpm
pnpm dev
```

## Production

Build the application for production:

```bash
# pnpm
pnpm build
```

## Note

If on linux you are getting an error like `Failed to create GBM buffer of size 800x600: Invalid argument` you can try setting `WEBKIT_DISABLE_DMABUF_RENDERER` to `1` in your environment.