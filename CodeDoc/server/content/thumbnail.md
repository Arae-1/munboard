# Thumbnail Generation (`content/thumbnail.rs`)

## Summary

This module generates thumbnails from decoded images. It resizes images to fit within configured dimensions while preserving aspect ratio, then composites transparent pixels onto a white background for JPEG compatibility.

## Imports

- `config::Config`: Application configuration.
- `image::imageops::FilterType`: Image resampling filter.
- `image::{DynamicImage, GenericImageView, Pixel, Rgba, RgbaImage}`: Image types and pixel operations.

## Enum: `ThumbnailType`

```rust
#[derive(Clone, Copy)]
pub enum ThumbnailType {
    Post,
    Avatar,
}
```

Specifies whether to use post or avatar thumbnail dimensions from config.

## Enum: `ThumbnailCategory`

```rust
#[derive(Clone, Copy)]
pub enum ThumbnailCategory {
    Generated,
    Custom,
}
```

Distinguishes between auto-generated thumbnails and user-uploaded custom thumbnails.

## Function: `create(config, image, thumbnail_type) -> DynamicImage`

```rust
pub fn create(config: &Config, image: &DynamicImage, thumbnail_type: ThumbnailType) -> DynamicImage
```

Creates a thumbnail from an image.

**Algorithm:**

### Step 1: Read target dimensions from config
- `ThumbnailType::Post`: `config.thumbnails.post_dimensions()` -> `(config_width, config_height)`.
- `ThumbnailType::Avatar`: `config.thumbnails.avatar_dimensions()` -> `(config_width, config_height)`.

### Step 2: Compute resize dimensions
The resize uses the smaller dimension as the constraint to preserve aspect ratio. The `DynamicImage::resize()` method constrains the image to fit within `(thumbnail_width, thumbnail_height)` while maintaining the aspect ratio.

- If `image_width > image_height`: the image is wider than tall. Constrain by height: `(u32::MAX, config_height)`. The height is limited, width scales proportionally.
- Otherwise: the image is taller than wide. Constrain by width: `(config_width, u32::MAX)`.

### Step 3: Resize
```rust
let resized_image = image.resize(thumbnail_width, thumbnail_height, FilterType::Gaussian);
```
Uses Gaussian filtering for smooth downscaling.

### Step 4: Handle alpha channel (transparency)
- If the resized image has an alpha channel (`resized_image.color().has_alpha()`):
  - Create a white RGBA canvas of the same dimensions (`Rgba([255, 255, 255, 255])`).
  - Convert the resized image to RGBA8.
  - For each pixel pair (canvas, image): blend the image pixel onto the canvas pixel using alpha blending (`canvas_pixel.blend(image_pixel)`).
  - Return `DynamicImage::ImageRgba8(canvas)`.
- If no alpha: return the resized image as-is.

The alpha blending step ensures transparent pixels become white rather than corrupting the JPEG output (JPEG does not support alpha).
