use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use futures_lite::future;
use noise::{NoiseFn, Perlin};

use crate::plugins::camera::camera::FlyCamera;

const CHUNK_SIZE: usize = 16;
const MAX_HEIGHT: usize = 100;
const TERRAIN_HEIGHT: usize = 40;
const RENDER_DISTANCE: usize = 30;

pub struct Chunk {
    pub position: Vec3,
    pub blocks: Vec<Block>,
    pub mesh: Mesh,
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, |mut commands: Commands| {
            commands.insert_resource(LoadedChunks {
                chunks: HashMap::new(),
            });
        })
        .add_systems(Update, prepare_chunks)
        .add_systems(Update, generate_chunk);
    }
}

#[derive(Debug)]
struct Visibility {
    top: bool,
    bottom: bool,
    right: bool,
    left: bool,
    front: bool,
    back: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self {
            top: true,
            bottom: true,
            right: true,
            left: true,
            front: true,
            back: true,
        }
    }
}

#[derive(Debug)]
pub struct Block {
    position: Vec3,
    visibility: Visibility,
}

impl Block {
    pub fn get_position_key(&self, add: Vec3) -> String {
        format!(
            "{}-{}-{}",
            self.position.x + add.x,
            self.position.y + add.y,
            self.position.z + add.z
        )
    }
}

#[derive(Component)]
struct ComputeChunk(Task<Chunk>);

#[derive(Resource)]
struct LoadedChunks {
    chunks: HashMap<String, Option<Chunk>>,
}

fn spawn_prepare_chunk_if_needed(
    x: i32,
    z: i32,
    loaded_chunks: &mut LoadedChunks,
    tasks: &mut Vec<ComputeChunk>,
    thread_pool: &AsyncComputeTaskPool,
    perlin: Perlin,
) {
    let chunk_id = format!("X{}Z{}", x, z);
    if !(loaded_chunks.chunks.contains_key(&chunk_id)) {
        loaded_chunks.chunks.insert(chunk_id, None);
        tasks.push(ComputeChunk(
            thread_pool.spawn(async move { prepare_chunk(x, z, perlin) }),
        ));
    }
}

fn prepare_chunks(
    mut commands: Commands,
    query: Query<&Transform, &FlyCamera>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    let camera = query.get_single().unwrap();
    let thread_pool = AsyncComputeTaskPool::get();
    let perlin = Perlin::new(10);

    let x = ((camera.translation.x / CHUNK_SIZE as f32).ceil() * CHUNK_SIZE as f32) as i32;
    let z = ((camera.translation.z / CHUNK_SIZE as f32).ceil() * CHUNK_SIZE as f32) as i32;

    for i_x in (x..).step_by(CHUNK_SIZE).take(RENDER_DISTANCE) {
        for i_z in (z..).step_by(CHUNK_SIZE).take(RENDER_DISTANCE) {
            let i_x_mirror = x - i_x;
            let i_z_mirror = z - i_z;
            let mut tasks = Vec::new();

            spawn_prepare_chunk_if_needed(
                i_x,
                i_z,
                &mut loaded_chunks,
                &mut tasks,
                &thread_pool,
                perlin,
            );
            spawn_prepare_chunk_if_needed(
                i_x_mirror,
                i_z,
                &mut loaded_chunks,
                &mut tasks,
                &thread_pool,
                perlin,
            );
            spawn_prepare_chunk_if_needed(
                i_x,
                i_z_mirror,
                &mut loaded_chunks,
                &mut tasks,
                &thread_pool,
                perlin,
            );
            spawn_prepare_chunk_if_needed(
                i_x_mirror,
                i_z_mirror,
                &mut loaded_chunks,
                &mut tasks,
                &thread_pool,
                perlin,
            );

            commands.spawn_batch(tasks);
        }
    }
}

fn get_perlin_value(
    perlin: Perlin,
    x: f32,
    y: f32,
    mut amplitude: f32,
    mut frequency: f32,
    octaves: u32,
    persistence: f32,
    lunacrity: f32,
) -> f32 {
    let mut value = 0.0;

    for _ in 0..octaves {
        value += amplitude * perlin.get([(x * frequency) as f64, (y * frequency) as f64]) as f32;
        amplitude *= persistence;
        frequency *= lunacrity;
    }

    value
}

fn is_block_at(position: Vec3, perlin: Perlin) -> bool {
    get_perlin_heigth(position.x, position.z, perlin) as f32 > position.y
}

fn get_perlin_heigth(x: f32, z: f32, perlin: Perlin) -> u32 {
    let perlin_value = get_perlin_value(perlin, x * 0.01, z * 0.01, 0.4, 1.0, 4, 0.5, 2.0);
    let height_value = perlin_value * 90.0;
    let height = TERRAIN_HEIGHT as u32
        + height_value as u32
        + HEIGTH_MAP
            .iter()
            .find(|h| h[0] > height_value as u32)
            .unwrap_or_else(|| &HEIGTH_MAP[2])[1];

    return height;
}

fn assign_visibility(block: &mut Block, perlin: Perlin) {
    if is_block_at(
        Vec3::new(block.position.x - 1.0, block.position.y, block.position.z),
        perlin,
    ) {
        block.visibility.left = false;
    }
    if is_block_at(
        Vec3::new(block.position.x + 1.0, block.position.y, block.position.z),
        perlin,
    ) {
        block.visibility.right = false;
    }
    if is_block_at(
        Vec3::new(block.position.x, block.position.y + 1.0, block.position.z),
        perlin,
    ) {
        block.visibility.top = false;
    }
    if is_block_at(
        Vec3::new(block.position.x, block.position.y - 1.0, block.position.z),
        perlin,
    ) {
        block.visibility.bottom = false;
    }
    if is_block_at(
        Vec3::new(block.position.x, block.position.y, block.position.z - 1.0),
        perlin,
    ) {
        block.visibility.front = false;
    }
    if is_block_at(
        Vec3::new(block.position.x, block.position.y, block.position.z + 1.0),
        perlin,
    ) {
        block.visibility.back = false;
    }
}

fn create_block_vertices(block: &Block) -> Vec<[f32; 3]> {
    let mut vertices = Vec::new();

    if block.visibility.top {
        vertices.extend_from_slice(&[
            [
                block.position.x - 0.5,
                block.position.y + 0.5,
                block.position.z - 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y + 0.5,
                block.position.z - 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y + 0.5,
                block.position.z + 0.5,
            ],
            [
                block.position.x - 0.5,
                block.position.y + 0.5,
                block.position.z + 0.5,
            ],
        ])
    }
    if block.visibility.bottom {
        vertices.extend_from_slice(&[
            [
                block.position.x - 0.5,
                block.position.y - 0.5,
                block.position.z - 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y - 0.5,
                block.position.z - 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y - 0.5,
                block.position.z + 0.5,
            ],
            [
                block.position.x - 0.5,
                block.position.y - 0.5,
                block.position.z + 0.5,
            ],
        ])
    }
    if block.visibility.right {
        vertices.extend_from_slice(&[
            [
                block.position.x + 0.5,
                block.position.y - 0.5,
                block.position.z - 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y - 0.5,
                block.position.z + 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y + 0.5,
                block.position.z + 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y + 0.5,
                block.position.z - 0.5,
            ],
        ])
    }
    if block.visibility.left {
        vertices.extend_from_slice(&[
            [
                block.position.x - 0.5,
                block.position.y - 0.5,
                block.position.z - 0.5,
            ],
            [
                block.position.x - 0.5,
                block.position.y - 0.5,
                block.position.z + 0.5,
            ],
            [
                block.position.x - 0.5,
                block.position.y + 0.5,
                block.position.z + 0.5,
            ],
            [
                block.position.x - 0.5,
                block.position.y + 0.5,
                block.position.z - 0.5,
            ],
        ])
    }
    if block.visibility.back {
        vertices.extend_from_slice(&[
            [
                block.position.x - 0.5,
                block.position.y - 0.5,
                block.position.z + 0.5,
            ],
            [
                block.position.x - 0.5,
                block.position.y + 0.5,
                block.position.z + 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y + 0.5,
                block.position.z + 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y - 0.5,
                block.position.z + 0.5,
            ],
        ])
    }
    if block.visibility.front {
        vertices.extend_from_slice(&[
            [
                block.position.x - 0.5,
                block.position.y - 0.5,
                block.position.z - 0.5,
            ],
            [
                block.position.x - 0.5,
                block.position.y + 0.5,
                block.position.z - 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y + 0.5,
                block.position.z - 0.5,
            ],
            [
                block.position.x + 0.5,
                block.position.y - 0.5,
                block.position.z - 0.5,
            ],
        ])
    }

    vertices
}

fn create_block_indices(block: &Block, mut skip: i32) -> (Vec<u32>, i32) {
    let mut indices = Vec::new();

    if block.visibility.top {
        indices.extend_from_slice(&[0 + skip, 3 + skip, 1 + skip, 1 + skip, 3 + skip, 2 + skip])
    } else {
        skip -= 4;
    }
    if block.visibility.bottom {
        indices.extend_from_slice(&[
            4 + skip, // 0,
            5 + skip, // 1,
            7 + skip, // 3,
            5 + skip, // 1,
            6 + skip, // 2,
            7 + skip, // 3,
        ])
    } else {
        skip -= 4
    }
    if block.visibility.right {
        indices.extend_from_slice(&[
            8 + skip,
            11 + skip,
            9 + skip,
            9 + skip,
            11 + skip,
            10 + skip,
        ])
    } else {
        skip -= 4;
    }
    if block.visibility.left {
        indices.extend_from_slice(&[
            12 + skip,
            13 + skip,
            15 + skip,
            13 + skip,
            14 + skip,
            15 + skip,
        ])
    } else {
        skip -= 4;
    }
    if block.visibility.back {
        indices.extend_from_slice(&[
            16 + skip,
            19 + skip,
            17 + skip,
            17 + skip,
            19 + skip,
            18 + skip,
        ])
    } else {
        skip -= 4;
    }
    if block.visibility.front {
        indices.extend_from_slice(&[
            20 + skip,
            21 + skip,
            23 + skip,
            21 + skip,
            22 + skip,
            23 + skip,
        ])
    } else {
        skip -= 4;
    };
    skip += 24;

    (indices.iter().map(|i| *i as u32).collect::<Vec<_>>(), skip)
}

fn create_block_uvs(block: &Block) -> Vec<[f32; 2]> {
    let mut uvs = Vec::new();

    if block.visibility.top {
        uvs.extend_from_slice(&[[0.66, 0.25], [0.33, 0.25], [0.33, 0.5], [0.66, 0.5]])
    }
    if block.visibility.bottom {
        uvs.extend_from_slice(&[[0.66, 0.75], [0.33, 0.75], [0.33, 1.0], [0.66, 1.0]])
    }
    if block.visibility.right {
        uvs.extend_from_slice(&[[0.66, 0.5], [0.33, 0.5], [0.33, 0.75], [0.66, 0.75]])
    }
    if block.visibility.left {
        uvs.extend_from_slice(&[[0.66, 0.0], [0.33, 0.0], [0.33, 0.25], [0.66, 0.25]])
    }
    if block.visibility.back {
        uvs.extend_from_slice(&[[0.33, 0.25], [0.0, 0.25], [0.0, 0.5], [0.33, 0.5]])
    }
    if block.visibility.front {
        uvs.extend_from_slice(&[[1.0, 0.25], [0.66, 0.25], [0.66, 0.5], [1.0, 0.5]])
    }
    uvs
}

fn create_block_normals(block: &Block) -> Vec<[f32; 3]> {
    let mut normals = Vec::new();

    if block.visibility.top {
        normals.extend_from_slice(&[
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ])
    }
    if block.visibility.bottom {
        normals.extend_from_slice(&[
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
        ])
    }
    if block.visibility.right {
        normals.extend_from_slice(&[
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ])
    }
    if block.visibility.left {
        normals.extend_from_slice(&[
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
        ])
    }
    if block.visibility.back {
        normals.extend_from_slice(&[
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ])
    }
    if block.visibility.front {
        normals.extend_from_slice(&[
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ])
    }
    normals
}

const HEIGTH_MAP: [[u32; 2]; 3] = [[20, 10], [25, 15], [30, 20]];
fn prepare_chunk(start_x: i32, start_z: i32, perlin: Perlin) -> Chunk {
    let mut chunk = Chunk {
        position: Vec3::new(start_x as f32, 0.0, start_z as f32),
        blocks: Vec::new(),
        mesh: Mesh::new(PrimitiveTopology::TriangleList),
    };

    for x in (start_x..).step_by(1).take(CHUNK_SIZE) {
        for y in 0..MAX_HEIGHT {
            for z in (start_z..).step_by(1).take(CHUNK_SIZE) {
                if is_block_at(Vec3::new(x as f32, y as f32, z as f32), perlin) {
                    let block = Block {
                        position: Vec3::new(x as f32, y as f32, z as f32),
                        visibility: Visibility::default(),
                    };

                    chunk.blocks.push(block);
                }
            }
        }
    }
    chunk
        .blocks
        .iter_mut()
        .for_each(|mut b| assign_visibility(&mut b, perlin));

    let vertices = chunk
        .blocks
        .iter()
        .map(|b| create_block_vertices(&b))
        .flatten()
        .collect::<Vec<_>>();
    chunk
        .mesh
        .insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    let mut skip: i32 = 0;
    let indices = Indices::U32(
        chunk
            .blocks
            .iter()
            .map(|b| {
                let (indices, new_skip) = create_block_indices(&b, skip);
                skip = new_skip;
                indices
            })
            .flatten()
            .collect::<Vec<_>>(),
    );
    chunk.mesh.set_indices(Some(indices));

    let uvs = chunk
        .blocks
        .iter()
        .map(|b| create_block_uvs(&b))
        .flatten()
        .collect::<Vec<_>>();
    chunk.mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    let normals = chunk
        .blocks
        .iter()
        .map(|b| create_block_normals(&b))
        .flatten()
        .collect::<Vec<_>>();
    chunk.mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    chunk
}

fn generate_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    server: Res<AssetServer>,
    mut chunk_tasks: Query<(Entity, &mut ComputeChunk)>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    let handle: Handle<Image> = server.load("grass.png");

    for (e, mut task) in &mut chunk_tasks {
        if let Some(chunk) = future::block_on(future::poll_once(&mut task.0)) {
            let mesh_handle = meshes.add(chunk.mesh.clone());

            let chunk_data = PbrBundle {
                mesh: mesh_handle,
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(handle.clone()),
                    ..default()
                }),
                ..default()
            };

            let id = format!("X{}Z{}", chunk.position.x, chunk.position.z);
            loaded_chunks.chunks.insert(id, Some(chunk));
            commands.spawn(chunk_data);
            commands.entity(e).remove::<ComputeChunk>();
        }
    }
}
