use crate::*;
use bevy::asset::io::ErasedAssetWriter;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::prelude::*;
use rand::Rng;
use crate::procgen;

#[derive(Component)]
struct TileComponent;

#[derive(Resource)]
pub struct GroundTiles(pub HashSet<(i32, i32)>);

#[derive(Resource)]
struct GenerationSeed(u32);

#[derive(Event)]
pub struct ResetTerrainEvent;

#[derive(Resource)]
struct SpriteSheetTexture(pub Handle<Image>);

#[derive(Component)]
struct Chunk(IVec2);

#[derive(Resource)]
struct SpawnedChunks(pub HashSet<IVec2>);

const TILE_SIZE: UVec2 = UVec2 { x: 32, y: 32 };
const GRID_SIZE: UVec2 = UVec2 { x: 32, y: 16 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 16, y: 16 };
const GRID_PROJECTION: Vec2 = Vec2 {
    x: 1.,
    y: GRID_SIZE.y as f32 / GRID_SIZE.x as f32,
};

const MAX_CHUNK_SPAWN: usize = 3;
const MAX_CHUNK_DESPAWN: usize = 1;

const MAX_ELEVATION: u32 = 4;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        let mut rng = rand::rng();
        app.add_plugins(TilemapPlugin)
            .insert_resource(GenerationSeed(rng.random()))
            .insert_resource(SpawnedChunks(HashSet::new()))
            .insert_resource(GroundTiles(HashSet::new()))
            .add_systems(Startup, setup)
            .add_systems(Update, spawn_chunks_around_camera)
            .add_systems(Update, despawn_off_camera_chunks)
            .add_systems(Startup, setup)
            .add_event::<ResetTerrainEvent>();
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("terrain.png");
    commands.insert_resource(SpriteSheetTexture(texture_handle.clone()));
}

fn spawn_chunk(
    commands: &mut Commands,
    chunk_pos: IVec2,
    sprite_sheet_texture: &SpriteSheetTexture,
    seed: &GenerationSeed,
) {
    let mut layers = vec![vec![]; MAX_ELEVATION as usize];
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let global_pos = chunk_pos * CHUNK_SIZE.as_ivec2() + IVec2::new(x as i32, y as i32);
            let tile_pos = TilePos { x, y };
            let tiles = procgen::generate_tiles(global_pos, seed.0);
            for (h, tile) in tiles.into_iter().enumerate() {
                layers[h].push((tile_pos, tile));
            }
        }
    }

    for (h, tiles) in layers.into_iter().enumerate() {
        if (tiles.is_empty()) {
            continue;
        }

        let chunk_entity = commands.spawn(Chunk(chunk_pos)).id();

        let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());

        for (tile_pos, tile) in tiles {
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    texture_index: tile,
                    tilemap_id: TilemapId(chunk_entity),
                    ..default()
                })
                .id();
            commands.entity(chunk_entity).add_child(tile_entity);
            tile_storage.set(&tile_pos, tile_entity);
        }

        let mut translation = chunk_to_world(chunk_pos);
        translation.y += h as f32 * GRID_SIZE.y as f32 / 2.;
        let transform = Transform::from_translation(translation.extend(h as f32));

        commands.entity(chunk_entity).insert(TilemapBundle {
            grid_size: TilemapGridSize::from(GRID_SIZE.as_vec2()),
            size: CHUNK_SIZE.into(),
            storage: tile_storage,
            texture: TilemapTexture::Single(sprite_sheet_texture.0.clone()),
            tile_size: TilemapTileSize::from(TILE_SIZE.as_vec2()),
            map_type: TilemapType::Isometric(IsoCoordSystem::Staggered),
            anchor: TilemapAnchor::Center,
            transform,
            render_settings: TilemapRenderSettings {
                y_sort: true,
                render_chunk_size: UVec2::new(CHUNK_SIZE.x, 1),
            },
            ..default()
        });
    }
}

fn chunk_to_world(chunk_pos: IVec2) -> Vec2 {
    chunk_pos.as_vec2() * GRID_SIZE.as_vec2() * CHUNK_SIZE.as_vec2() * GRID_PROJECTION
}

fn world_to_chunk(world_pos: Vec2) -> Vec2 {
    world_pos / GRID_SIZE.as_vec2() / CHUNK_SIZE.as_vec2() / GRID_PROJECTION
}

fn compute_visible_chunks(
    camera: &Camera,
    transform: &GlobalTransform,
    padding: f32,
) -> HashSet<IVec2> {
    let edge = 1. + padding;

    let Some(min) = camera.ndc_to_world(transform, Vec3::new(-edge, -edge, 1.0)) else {
        return HashSet::new();
    };

    let Some(max) = camera.ndc_to_world(transform, Vec3::new(edge, edge, 1.0)) else {
        return HashSet::new();
    };

    let min_chunk = world_to_chunk(min.xy() - padding);
    let max_chunk = world_to_chunk(max.xy() + padding);

    let mut visible_chunks = HashSet::new();
    for y in (min_chunk.y.floor() as i32)..=(max_chunk.y.ceil() as i32) {
        for x in (min_chunk.x.floor() as i32)..=(max_chunk.x.ceil() as i32) {
            visible_chunks.insert(IVec2::new(x, y));
        }
    }

    visible_chunks
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut spawned_chunks: ResMut<SpawnedChunks>,
    sprite_sheet_texture: Res<SpriteSheetTexture>,
    seed: Res<GenerationSeed>,
) {
    for (camera, transform) in camera_query.iter() {
        let visible_chunks = compute_visible_chunks(camera, transform, 0.1);

        let new_chunks = visible_chunks
            .difference(&spawned_chunks.0)
            .take(MAX_CHUNK_SPAWN)
            .cloned()
            .collect::<Vec<_>>();

        for chunk in new_chunks {
            spawned_chunks.0.insert(chunk);
            spawn_chunk(&mut commands, chunk, &sprite_sheet_texture, &seed);
        }
    }
}

fn despawn_off_camera_chunks(
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    chunks_query: Query<(Entity, &Chunk)>,
    mut spawned_chunks: ResMut<SpawnedChunks>,
) {
    let mut visible_chunks = HashSet::new();
    for (camera, transform) in camera_query.iter() {
        visible_chunks.extend(compute_visible_chunks(camera, transform, 0.5));
    }

    let mut count = 0;
    for (entity, chunk) in chunks_query.iter() {
        if visible_chunks.contains(&chunk.0) {
            continue;
        }

        spawned_chunks.0.remove(&chunk.0);
        commands.entity(entity).despawn();
        count += 1;
        if count >= MAX_CHUNK_DESPAWN {
            break;
        }
    }
}

fn handle_terrain_reset_event(
    mut commands: Commands,
    mut reader: EventReader<ResetTerrainEvent>,
    mut spawned_chunks: ResMut<SpawnedChunks>,
    mut ground_tiles: ResMut<GroundTiles>,
    mut seed: ResMut<GenerationSeed>,
    chunks_query: Query<Entity, With<Chunk>>,
) {
    if reader.is_empty() {
        return;
    }

    reader.clear();
    for e in chunks_query.iter() {
        commands.entity(e).despawn();
    }

    // Reset res
    spawned_chunks.0.clear();
    ground_tiles.0.clear();

    let mut rng = rand::rng();
    seed.0 = rng.random();
}
