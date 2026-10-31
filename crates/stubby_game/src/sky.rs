use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
};

pub fn desert_sky() -> Image {
    let size = 128;
    // Bevy samples cubemaps in left-handed space (world Z is negated).
    let sun = Vec3::new(-0.48, 0.075, 0.87).normalize();
    let mut data = Vec::with_capacity(size * size * 6 * 8);
    for face in 0..6 {
        for y in 0..size {
            for x in 0..size {
                let u = 2. * (x as f32 + 0.5) / size as f32 - 1.;
                let v = 2. * (y as f32 + 0.5) / size as f32 - 1.;
                let d = match face {
                    0 => Vec3::new(1., -v, -u),
                    1 => Vec3::new(-1., -v, u),
                    2 => Vec3::new(u, 1., v),
                    3 => Vec3::new(u, -1., -v),
                    4 => Vec3::new(u, -v, 1.),
                    _ => Vec3::new(-u, -v, -1.),
                }
                .normalize();
                let horizon = Vec3::new(0.72, 0.43, 0.22);
                let zenith = Vec3::new(0.14, 0.19, 0.25);
                let mut rgb = horizon.lerp(zenith, d.y.max(0.).sqrt());
                if d.y < 0. {
                    rgb = horizon.lerp(Vec3::new(0.13, 0.09, 0.06), (-d.y * 3.).min(1.));
                }
                let alignment = d.dot(sun).max(0.);
                rgb += Vec3::new(1.0, 0.56, 0.19)
                    * (alignment.powf(24.) * 0.65 + alignment.powf(1500.) * 12.);
                for c in [rgb.x, rgb.y, rgb.z, 1.] {
                    data.extend_from_slice(&half::f16::from_f32(c).to_le_bytes());
                }
            }
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba16Float,
        RenderAssetUsages::default(),
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image
}
