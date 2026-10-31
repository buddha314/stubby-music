//! Reproducible edit-viewport proxy; Play always uses the procedural generator.
use bevy::{mesh::VertexAttributeValues, prelude::*};
use serde_json::json;
use std::{fs, io::Write, path::Path};

pub fn write_preview(directory: &Path, seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    write_preview_with_radius(directory, seed, 60)
}

fn write_preview_with_radius(
    directory: &Path,
    seed: u64,
    radius: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(directory)?;
    let (albedo, normal) = super::mud_texture(seed);
    for (name, image) in [("mud.png", albedo), ("mud-normal.png", normal)] {
        let file = fs::File::create(directory.join(name))?;
        let mut encoder = png::Encoder::new(file, image.width(), image.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(image.data.as_deref().ok_or("missing texture pixels")?)?;
    }
    let mesh = super::terrain::ground(seed, radius);
    let mut buffer = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();
    let attributes = [
        Mesh::ATTRIBUTE_POSITION,
        Mesh::ATTRIBUTE_NORMAL,
        Mesh::ATTRIBUTE_UV_0,
        Mesh::ATTRIBUTE_COLOR,
        Mesh::ATTRIBUTE_TANGENT,
    ];
    for attribute in attributes {
        let data = mesh.attribute(attribute).ok_or("missing mesh attribute")?;
        let start = buffer.len();
        let (count, kind) = match data {
            VertexAttributeValues::Float32x2(values) => {
                for value in values {
                    for f in value {
                        buffer.extend(f.to_le_bytes());
                    }
                }
                (values.len(), "VEC2")
            }
            VertexAttributeValues::Float32x3(values) => {
                for value in values {
                    for f in value {
                        buffer.extend(f.to_le_bytes());
                    }
                }
                (values.len(), "VEC3")
            }
            VertexAttributeValues::Float32x4(values) => {
                for value in values {
                    for f in value {
                        buffer.extend(f.to_le_bytes());
                    }
                }
                (values.len(), "VEC4")
            }
            _ => return Err("unsupported vertex format".into()),
        };
        views.push(
            json!({"buffer":0,"byteOffset":start,"byteLength":buffer.len()-start,"target":34962}),
        );
        let mut accessor =
            json!({"bufferView":accessors.len(),"componentType":5126,"count":count,"type":kind});
        if attribute == Mesh::ATTRIBUTE_POSITION {
            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(VertexAttributeValues::as_float3)
                .ok_or("missing positions")?;
            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);
            for p in positions {
                min = min.min(Vec3::from_array(*p));
                max = max.max(Vec3::from_array(*p));
            }
            accessor["min"] = json!(min.to_array());
            accessor["max"] = json!(max.to_array());
        }
        accessors.push(accessor);
    }
    let document = json!({
        "asset":{"version":"2.0","generator":"stubby-music procedural preview"},
        "scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0,"name":format!("Mudflat seed {seed}")}],
        "meshes":[{"primitives":[{"attributes":{"POSITION":0,"NORMAL":1,"TEXCOORD_0":2,"COLOR_0":3,"TANGENT":4},"material":0}]}],
        "materials":[{"name":"Procedural dry mud","pbrMetallicRoughness":{"baseColorTexture":{"index":0},"metallicFactor":0.0,"roughnessFactor":0.96},"normalTexture":{"index":1}}],
        "textures":[{"source":0,"sampler":0},{"source":1,"sampler":0}],
        "images":[{"uri":"mud.png"},{"uri":"mud-normal.png"}],
        "samplers":[{"magFilter":9729,"minFilter":9987,"wrapS":10497,"wrapT":10497}],
        "buffers":[{"byteLength":buffer.len()}],"bufferViews":views,"accessors":accessors
    });
    let mut text = serde_json::to_vec(&document)?;
    while !text.len().is_multiple_of(4) {
        text.push(b' ');
    }
    while !buffer.len().is_multiple_of(4) {
        buffer.push(0);
    }
    let mut out = fs::File::create(directory.join("mudflat.glb"))?;
    out.write_all(b"glTF")?;
    out.write_all(&2_u32.to_le_bytes())?;
    out.write_all(&((12 + 8 + text.len() + 8 + buffer.len()) as u32).to_le_bytes())?;
    out.write_all(&(text.len() as u32).to_le_bytes())?;
    out.write_all(b"JSON")?;
    out.write_all(&text)?;
    out.write_all(&(buffer.len() as u32).to_le_bytes())?;
    out.write_all(b"BIN\0")?;
    out.write_all(&buffer)?;
    fs::write(directory.join("seed.txt"), format!("{seed}\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_loads_as_gltf_with_both_textures_and_bounded_vertices() {
        let directory = tempfile::tempdir().expect("temporary export directory");
        write_preview_with_radius(directory.path(), 42, 2).expect("export preview");
        let (document, buffers, images) = gltf::import(directory.path().join("mudflat.glb"))
            .expect("valid glTF and texture references");
        assert_eq!(images.len(), 2);
        assert!(
            images
                .iter()
                .all(|image| image.width == 1024 && image.height == 1024)
        );
        let primitive = document
            .meshes()
            .next()
            .unwrap()
            .primitives()
            .next()
            .unwrap();
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let positions: Vec<_> = reader.read_positions().unwrap().collect();
        assert!(!positions.is_empty());
        assert_eq!(positions.len() % 3, 0);
        assert!(
            positions
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite()) && p[1].abs() < 0.5)
        );
        assert_eq!(reader.read_normals().unwrap().count(), positions.len());
        assert_eq!(reader.read_tangents().unwrap().count(), positions.len());
        assert_eq!(
            reader.read_tex_coords(0).unwrap().into_f32().count(),
            positions.len()
        );
        assert!(primitive.material().normal_texture().is_some());
    }
}
