use bevy::math::{DVec2, IVec2};
use bevy_ecs_tilemap::prelude::TileTextureIndex;
use noise::{HybridMulti, MultiFractal, NoiseFn, OpenSimplex, Worley};
use rand::prelude::StdRng;
use rand::{random_range, Rng, SeedableRng};

const DIRT: TileTextureIndex = TileTextureIndex(3);
const SAND: TileTextureIndex = TileTextureIndex(9);
const GRASS: TileTextureIndex = TileTextureIndex(40);
const WATER: TileTextureIndex = TileTextureIndex(110);
const TULIPS: TileTextureIndex = TileTextureIndex(44);
const TULIP_BUSH: TileTextureIndex = TileTextureIndex(45);
const LEAFY_FLOWERS: TileTextureIndex = TileTextureIndex(46);
const FLOWERS: TileTextureIndex = TileTextureIndex(47);
const LARGE_ROCK: TileTextureIndex = TileTextureIndex(64);
const MEDIUM_ROCK: TileTextureIndex = TileTextureIndex(65);
const SCULPTED_ROCK: TileTextureIndex = TileTextureIndex(66);
const SMALL_ROCK: TileTextureIndex = TileTextureIndex(67);
const ROCK: TileTextureIndex = TileTextureIndex(63);
const CRACKED_ROCK: TileTextureIndex = TileTextureIndex(61);
const LOG: TileTextureIndex = TileTextureIndex(48);
const LOGS: TileTextureIndex = TileTextureIndex(49);
const FALLEN_LOG: TileTextureIndex = TileTextureIndex(50);
const GRASSY_FALLEN_LOG: TileTextureIndex = TileTextureIndex(51);
const DEAD_LOG: TileTextureIndex = TileTextureIndex(52);

pub fn generate_tiles(tile_pos: IVec2, seed: u32) -> Vec<TileTextureIndex> {
    let tile_position_seed =
        ((tile_pos.x.cast_unsigned() << 16) | (tile_pos.y.cast_unsigned() & 0xFFFF)) as u64;
    let mut rng = StdRng::seed_from_u64(seed as u64 + (tile_position_seed << 32));
    let chance = rng.random_range(0.0..1.0);
    let point = *(tile_pos.as_dvec2() / 128.).as_ref();

    let elevation = HybridMulti::<OpenSimplex>::new(seed).get(point);
    let moisture = HybridMulti::<OpenSimplex>::new(seed + 1).get(point);

    if elevation < -0.3 {
        return vec![WATER];
    }

    if elevation < -0.2 {
        if moisture < -0.2 {
            return vec![SAND];
        }

        return vec![GRASS];
    }

    if elevation > 0.5 {
        if elevation > 0.55 {
            if chance < 0.03 {
                return vec![ROCK, ROCK, ROCK, LARGE_ROCK];
            }
        }
        return vec![ROCK, ROCK, ROCK];
    }

    if moisture < -0.4 {
        return vec![DIRT, SAND];
    }

    if moisture > 0.3 {
        if moisture > 0.2 {
            if chance < 0.006 {
                return vec![DIRT, GRASS, TULIPS];
            } else if chance < 0.01 {
                return vec![DIRT, GRASS, LOG];
            } else if chance < 0.02 {
                return vec![DIRT, GRASS, FALLEN_LOG];
            }
        }
        return vec![DIRT, GRASS];
    }

    return vec![DIRT, DIRT];
}
