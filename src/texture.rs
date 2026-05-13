// Texture loading helper for chapter 5.
//
// The renderer needs three GPU objects to sample an image in a shader:
// 1. `wgpu::Texture`     - the actual image memory on the GPU.
// 2. `wgpu::TextureView` - the shader/render-pass view into that texture.
// 3. `wgpu::Sampler`     - the rules for reading pixels from the texture.
//
// Extra reading: what is a texture, conceptually?
// - Computer Graphics from Scratch: Textures
//   Explains texels, UV coordinates, texture lookup, filtering, and mipmapping.
//   https://gabrielgambetta.com/computer-graphics-from-scratch/14-textures.html
//
// - Scratchapixel: Introduction to Texturing
//   Conceptual overview of texture mapping: using images to add surface detail.
//   https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-texturing/introduction-to-texturing.html
//
// - Scratchapixel: Texture Mapping Basic Implementation
//   Shows how UV coordinates are used to fetch image colors during rendering.
//   https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-texturing/introduction-to-texturing-basic-implementation.html
//
// - Scratchapixel: Perspective-Correct Interpolation
//   Explains how rasterization interpolates vertex attributes like UVs across triangles.
//   https://www.scratchapixel.com/lessons/3d-basic-rendering/rasterization-practical-implementation/perspective-correct-interpolation-vertex-attributes
//
// - OpenGL Tutorial: How Mipmap Selection Works
//   Deeper explanation of how GPUs choose mip levels while rendering.
//   https://paroj.github.io/gltut/Texturing/Tut15%20How%20Mipmapping%20Works.html

// Brings the `dimensions()` method into scope for `image::DynamicImage`.
// Without this trait import, `image.dimensions()` below will not compile.
use image::GenericImageView;

// A small wrapper that keeps the texture-related GPU objects together.
//
// We store all three because the shader does not read a `Texture` directly:
// it samples from a `TextureView` using a `Sampler`.
pub struct Texture {
    #[allow(unused)]  // Do not warn me if texture is currently unused
    pub texture: wgpu::Texture, // wgpu::Texture is the actual GPU-side image storage: a block of GPU memory that can hold pixels, like a loaded PNG or a render target.
    pub view: wgpu::TextureView, // GPU-facing “view” of a wgpu::Texture. It describes how the texture should be accessed by shaders or render passes
    pub sampler: wgpu::Sampler, // GPU object that tells the shader how to read pixels from a texture
}

impl Texture {
    // Build a GPU texture from raw image-file bytes.
    //
    // `include_bytes!(...)` will give us a `&[u8]` containing a PNG/JPEG file.
    // This function decodes those bytes into an `image::DynamicImage`, then hands
    // the decoded image to `from_image`, which does the actual GPU upload work.
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, image::ImageError> {
        // Decode PNG/JPEG/etc. bytes into an image object on the CPU.
        // The `?` returns the image error to the caller if decoding fails.
        let image = image::load_from_memory(bytes)?;

        // Reuse the image-upload path below so all texture creation logic lives
        // in one place.
        Ok(Self::from_image(device, queue, &image, Some(label)))
    }

    // Build a GPU texture from an already-decoded CPU image.
    //
    // This is the main texture creation pipeline:
    // decode/convert pixels -> allocate GPU texture -> upload pixels ->
    // create view -> create sampler -> return wrapper.
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &image::DynamicImage,
        label: Option<&str>,
    ) -> Self {
        // Convert the image to 8-bit RGBA pixels.
        // `wgpu::TextureFormat::Rgba8UnormSrgb` expects 4 bytes per pixel:
        // red, green, blue, and alpha.
        let rgba = image.to_rgba8();

        // Read the image width and height so the GPU texture has the same size
        // as the source image.
        let dimensions = image.dimensions();

        // Describe the 2D texture's dimensions for wgpu.
        // Textures are described as 3D extents; a normal 2D image uses depth 1.
        let size = wgpu::Extent3d {
            width:dimensions.0,
            height:dimensions.1,
            depth_or_array_layers:1, // For a normal 2D texture, it usually means: how many texture layers. Most beginner 2D images use 1.
        };

        // Allocate the actual texture memory on the GPU.
        // At this point the texture exists, but it does not contain our image
        // pixels yet; the upload happens in `queue.write_texture` below.
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1, // how many mipmap levels this texture contains
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // TODO store it in a global
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[]
        });

        // Copy the CPU-side RGBA pixels into the GPU texture.
        //
        // `TexelCopyTextureInfo` says where on the GPU to write.
        // `TexelCopyBufferLayout` says how the CPU byte slice is arranged.
        // `size` says how many texels to copy.
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        // Create a default view of the whole texture.
        // The shader will bind this view, not the raw texture object.
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create the sampler that controls how texture coordinates turn into pixels.
        //
        // ClampToEdge prevents sampling outside [0, 1] from wrapping around.
        // Linear magnification smooths the image when it is enlarged.
        // Nearest minification/mipmap filtering keeps the setup simple for now.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
               address_mode_u: wgpu::AddressMode::Repeat,
               address_mode_v: wgpu::AddressMode::Repeat,
               address_mode_w: wgpu::AddressMode::ClampToEdge,
               mag_filter: wgpu::FilterMode::Linear,
               min_filter: wgpu::FilterMode::Nearest,
               mipmap_filter: wgpu::FilterMode::Nearest,
               ..Default::default()
           });

        // Return the three GPU objects as one logical texture resource.
        Self {
            texture,
            view,
            sampler,
        }
    }
}