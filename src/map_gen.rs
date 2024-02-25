use bevy::{
    math::vec2,
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    utils::petgraph::matrix_graph::Zero,
};
use macros::error_return;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Plane {
    n: Vec3,
    n_prime: Vec3,
    d: Vec3,
}
impl Plane {
    pub fn from_data(plane: map_parser::parser::Plane) -> Self {
        let p1 = Vec3::new(plane.p1.0, plane.p1.1, plane.p1.2);
        let p2 = Vec3::new(plane.p2.0, plane.p2.1, plane.p2.2);
        let p3 = Vec3::new(plane.p3.0, plane.p3.1, plane.p3.2);

        // calculate the normal vector
        let n_prime = (p2 - p1).cross(p3 - p1);
        // normalize it
        let n = n_prime
            / ((n_prime.x * n_prime.x) + (n_prime.y * n_prime.y) + (n_prime.z * n_prime.z)).sqrt();
        // calculate the parameter
        let d = (-p1) * n;
        Self { n, n_prime, d }
    }
}

fn get_intersection(i: Plane, j: Plane, k: Plane) -> Option<Vec3> {
    let denom = i.n.dot(j.n.cross(k.n));
    if denom < f32::EPSILON || denom.is_nan() {
        return None;
    }
    let p = -i.d * (j.n.cross(k.n)) - j.d * (k.n.cross(i.n)) - k.d * (i.n.cross(j.n)) / denom;
    Some(p)
}

fn vec2vec(vec: Vec3) -> nalgebra::Vector3<f32> {
    nalgebra::Vector3::new(vec.x, vec.y, vec.z)
}

pub fn test_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let map = error_return!(std::fs::read_to_string(
        "crates/map_parser/tests/simple.map"
    ));
    let map = error_return!(map_parser::parse(&map));

    for entity in map {
        for brush in entity.brushes {
            let planes = brush
                .into_iter()
                .map(|d| {
                    bevy::math::prelude::Plane3d::from_points(
                        Vec3::new(d.p1.0, d.p1.1, d.p1.2),
                        Vec3::new(d.p2.0, d.p2.1, d.p2.2),
                        Vec3::new(d.p3.0, d.p3.1, d.p3.2),
                    )
                })
                .map(|p| {
                    println!("{p:?}");
                    p
                })
                .map(|(p, vec)| {
                    println!("{p:?}");
                    implicit3d::NormalPlane::from_normal_and_p(
                        vec2vec(p.normal.into()),
                        vec.distance(Vec3::ZERO),
                    )
                })
                .map(|p| {
                    let b: Box<dyn implicit3d::Object<f32>> = Box::new(p);
                    b
                })
                .collect::<Vec<_>>();
            let mesh = implicit3d::Intersection::from_vec(planes, 0.0).unwrap();
            println!("{mesh:?}");

            // let mut new_mesh = Mesh::new(
            //     PrimitiveTopology::TriangleList,
            //     RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            // )
            // .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts);
            // new_mesh.compute_flat_normals();
            //
            // commands.spawn(PbrBundle {
            //     mesh: meshes.add(new_mesh),
            //     material: materials.add(Color::rgb_u8(255, 0, 0)),
            //     transform: Transform::default(),
            //     ..default()
            // });
        }
    }
}
