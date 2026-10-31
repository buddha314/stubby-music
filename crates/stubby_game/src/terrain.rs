use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

pub const SPACING: f32 = 1.8;
pub fn hash(x: i32, z: i32, seed: u64) -> f32 {
    let mut n = seed
        ^ (x as u64).wrapping_mul(0x9e3779b97f4a7c15)
        ^ (z as u64).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((n ^ (n >> 31)) >> 40) as f32 / 16777216.0
}
pub fn site(x: i32, z: i32, seed: u64) -> Vec2 {
    Vec2::new(
        x as f32 + 0.15 + hash(x, z, seed) * 0.7,
        z as f32 + 0.15 + hash(x, z, seed ^ 431) * 0.7,
    ) * SPACING
}
pub fn plate(x: i32, z: i32, seed: u64) -> Vec<Vec2> {
    let c = site(x, z, seed);
    let mut poly = vec![
        c + Vec2::new(-4., -4.),
        c + Vec2::new(4., -4.),
        c + Vec2::new(4., 4.),
        c + Vec2::new(-4., 4.),
    ];
    for dz in -2..=2 {
        for dx in -2..=2 {
            if dx == 0 && dz == 0 {
                continue;
            }
            let other = site(x + dx, z + dz, seed);
            let n = other - c;
            let middle = (other + c) * 0.5;
            let mut clipped = Vec::new();
            for i in 0..poly.len() {
                let a = poly[i];
                let b = poly[(i + 1) % poly.len()];
                let da = (a - middle).dot(n);
                let db = (b - middle).dot(n);
                if da <= 0. {
                    clipped.push(a);
                }
                if (da <= 0.) != (db <= 0.) {
                    clipped.push(a + (b - a) * (da / (da - db)));
                }
            }
            poly = clipped;
        }
    }
    poly.into_iter().map(|p| c + (p - c) * 0.965).collect()
}
pub fn height(p: Vec2) -> f32 {
    0.045 * (p.x * 0.14).sin() * (p.y * 0.11).cos()
}
#[derive(Default)]
pub struct Surface {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    tangents: Vec<[f32; 4]>,
}
impl Surface {
    pub fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3, color: [f32; 4]) {
        let n = (b - a).cross(c - a).normalize_or_zero();
        for p in [a, b, c] {
            self.positions.push(p.to_array());
            self.normals.push(n.to_array());
            let tangent = (Vec3::X - n * n.x).normalize_or_zero();
            self.tangents.push([tangent.x, tangent.y, tangent.z, -1.]);
            self.uvs.push([p.x / 16., p.z / 16.]);
            self.colors.push(color);
        }
    }
    pub fn mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, self.tangents)
    }
}
pub fn ground(seed: u64, radius: i32) -> Mesh {
    let mut s = Surface::default();
    for z in -radius..radius {
        for x in -radius..radius {
            let center = site(x, z, seed);
            let poly = plate(x, z, seed);
            let shade = 0.76 + hash(x, z, seed ^ 17) * 0.24;
            let color = [shade, shade * 0.95, shade * 0.86, 1.];
            let at = |p: Vec2, h: f32| Vec3::new(p.x, height(p) + h, p.y);
            for i in 0..poly.len() {
                let a = poly[i];
                let b = poly[(i + 1) % poly.len()];
                let ia = center + (a - center) * 0.76;
                let ib = center + (b - center) * 0.76;
                s.triangle(at(center, 0.), at(ib, 0.006), at(ia, 0.006), color);
                s.triangle(at(ia, 0.006), at(ib, 0.006), at(b, 0.035), color);
                s.triangle(at(ia, 0.006), at(b, 0.035), at(a, 0.035), color);
                let dark = [0.30, 0.25, 0.20, 1.];
                s.triangle(at(a, 0.035), at(b, 0.035), at(b, -0.13), dark);
                s.triangle(at(a, 0.035), at(b, -0.13), at(a, -0.13), dark);
            }
        }
    }
    s.mesh()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seeded_layout_is_repeatable_and_varies() {
        assert_eq!(plate(3, -2, 42), plate(3, -2, 42));
        assert_ne!(plate(3, -2, 42), plate(3, -2, 43));
    }
    #[test]
    fn plates_are_bounded_and_near_flat() {
        for z in -12..12 {
            for x in -12..12 {
                let p = plate(x, z, 42);
                assert!((3..=10).contains(&p.len()));
                for v in p {
                    assert!(v.distance(site(x, z, 42)) < 3.);
                    assert!(height(v).abs() + 0.13 < 0.5);
                }
            }
        }
    }
}
