use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use noise::{NoiseFn, Perlin};

const CHUNK_SIZE: usize = 100;

pub struct Chunk {
    pub blocks: Vec<Block>,
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, generate_chunk);
    }
}

pub struct Block {
    position: Vec3,
}

fn generate_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    server: Res<AssetServer>,
) {
    let handle: Handle<Image> = server.load("grass.png");
    let perlin = Perlin::new(200);

    let mut chunk = Chunk { blocks: Vec::new() };

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let y_perlin = perlin.get([x as f64, 1.64 as f64, z as f64]) as f32;
            let mut block = Block {
                position: Vec3::new(x as f32, y_perlin.ceil(), z as f32),
            };

            chunk.blocks.push(block);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let vertices = chunk
        .blocks
        .iter()
        .map(|b| {
            vec![
                // top
                [b.position.x - 0.5, b.position.y + 0.5, b.position.z - 0.5],
                [b.position.x + 0.5, b.position.y + 0.5, b.position.z - 0.5],
                [b.position.x + 0.5, b.position.y + 0.5, b.position.z + 0.5],
                [b.position.x - 0.5, b.position.y + 0.5, b.position.z + 0.5],
                // bottom
                [b.position.x - 0.5, b.position.y - 0.5, b.position.z - 0.5],
                [b.position.x + 0.5, b.position.y - 0.5, b.position.z - 0.5],
                [b.position.x + 0.5, b.position.y - 0.5, b.position.z + 0.5],
                [b.position.x - 0.5, b.position.y - 0.5, b.position.z + 0.5],
                // right
                [b.position.x + 0.5, b.position.y - 0.5, b.position.z - 0.5],
                [b.position.x + 0.5, b.position.y - 0.5, b.position.z + 0.5],
                [b.position.x + 0.5, b.position.y + 0.5, b.position.z + 0.5],
                [b.position.x + 0.5, b.position.y + 0.5, b.position.z - 0.5],
                // left
                [b.position.x - 0.5, b.position.y - 0.5, b.position.z - 0.5],
                [b.position.x - 0.5, b.position.y - 0.5, b.position.z + 0.5],
                [b.position.x - 0.5, b.position.y + 0.5, b.position.z + 0.5],
                [b.position.x - 0.5, b.position.y + 0.5, b.position.z - 0.5],
                // back
                [b.position.x - 0.5, b.position.y - 0.5, b.position.z + 0.5],
                [b.position.x - 0.5, b.position.y + 0.5, b.position.z + 0.5],
                [b.position.x + 0.5, b.position.y + 0.5, b.position.z + 0.5],
                [b.position.x + 0.5, b.position.y - 0.5, b.position.z + 0.5],
                // forward
                [b.position.x - 0.5, b.position.y - 0.5, b.position.z - 0.5],
                [b.position.x - 0.5, b.position.y + 0.5, b.position.z - 0.5],
                [b.position.x + 0.5, b.position.y + 0.5, b.position.z - 0.5],
                [b.position.x + 0.5, b.position.y - 0.5, b.position.z - 0.5],
            ]
        })
        .flatten()
        .collect::<Vec<_>>();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    let indices = Indices::U32(
        chunk
            .blocks
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let m = (i * 24) as u32;
                vec![
                    // top
                    0 + m,
                    3 + m,
                    1 + m,
                    1 + m,
                    3 + m,
                    2 + m,
                    // bottom
                    4 + m,
                    5 + m,
                    7 + m,
                    5 + m,
                    6 + m,
                    7 + m,
                    // right (+x)
                    8 + m,
                    11 + m,
                    9 + m,
                    9 + m,
                    11 + m,
                    10 + m,
                    // left (-x)
                    12 + m,
                    13 + m,
                    15 + m,
                    13 + m,
                    14 + m,
                    15 + m,
                    // back (+z)
                    16 + m,
                    19 + m,
                    17 + m,
                    17 + m,
                    19 + m,
                    18 + m,
                    // forward (-z)
                    20 + m,
                    21 + m,
                    23 + m,
                    21 + m,
                    22 + m,
                    23 + m,
                ]
            })
            .flatten()
            .collect::<Vec<_>>(),
    );
    mesh.set_indices(Some(indices));

    let uvs = chunk
        .blocks
        .iter()
        .map(|b| {
            vec![
                // Assigning the UV coords for the top side.
                [0.66, 0.25],
                [0.33, 0.25],
                [0.33, 0.5],
                [0.66, 0.5],
                // Assigning the UV coords for the bottom side.
                [0.66, 0.75],
                [0.33, 0.75],
                [0.33, 1.0],
                [0.66, 1.0],
                // Assigning the UV coords for the right side.
                [0.66, 0.5],
                [0.33, 0.5],
                [0.33, 0.75],
                [0.66, 0.75],
                // Assigning the UV coords for the left side.
                [0.66, 0.0],
                [0.33, 0.0],
                [0.33, 0.25],
                [0.66, 0.25],
                // Assigning the UV coords for the back side.
                [0.33, 0.25],
                [0.0, 0.25],
                [0.0, 0.5],
                [0.33, 0.5],
                // Assigning the UV coords for the forward side.
                [1.0, 0.25],
                [0.66, 0.25],
                [0.66, 0.5],
                [1.0, 0.5],
            ]
        })
        .flatten()
        .collect::<Vec<_>>();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    let normals = chunk
        .blocks
        .iter()
        .map(|_| {
            vec![
                // Normals for the top side (towards +y)
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                // Normals for the bottom side (towards -y)
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                // Normals for the right side (towards +x)
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                // Normals for the left side (towards -x)
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                // Normals for the back side (towards +z)
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                // Normals for the forward side (towards -z)
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
            ]
        })
        .flatten()
        .collect::<Vec<_>>();
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    let mesh_handle = meshes.add(mesh);

    let material = PbrBundle {
        mesh: mesh_handle,
        material: materials.add(StandardMaterial {
            base_color_texture: Some(handle),
            ..default()
        }),
        ..default()
    };

    commands.spawn(material);
}
