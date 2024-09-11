//          IMPORTS

use std::collections::HashMap;
use bevy::ecs::world;
use bevy::{
    input::mouse::MouseButtonInput, math::vec3, prelude::*, utils::HashSet,
    color::palettes::basic::*,
};
use noise::{NoiseFn, Perlin};
use rand::Rng;
use bevy::window::PrimaryWindow;
use bevy_pancam::{PanCamPlugin, PanCam};
use rand_distr::{Normal, Distribution};

//bevy ecs tilemap stuff
use bevy::{color::palettes, math::Vec4Swizzles};
use bevy::{ecs::system::Resource, prelude::*};

mod tile_data;
use tile_data::*;


//          CONSTANTS

//Spritesheet
const TERRAIN_SHEET_PATH: &str = "map.png";
const TILE_WIDTH: usize = 8;
const TILE_HEIGHT: usize = 8;
const SPRITE_SCALE_FACTOR: usize = 6;

//Map
pub const GRID_COLS:usize = 500;
pub const GRID_ROWS:usize = 500;
const PERLIN_NOISE_SCALE: f64 = 65.; //was 10

//Camera
const CAM_LERP_FACTOR: f32 = 4.0;
const CAM_SPEED_MIN: f32 = 300.;
const CAM_SPEED_MAX: f32 = 1500.;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_systems(Startup, setup) //Map loading/setup system
        .add_systems(FixedUpdate, animate_sprite) //Fixedupdate runs 60 FPS
        .add_systems(FixedUpdate, update_camera) //Camera control
        .add_systems(FixedUpdate, reload_on_r) //Reload map on 'r'
        .add_systems(FixedUpdate, mouse_input_handler) //mouse input handler
        .init_resource::<CursorWorldCoords>()
        .add_plugins(PanCamPlugin) //Adds zoom and mouse-pan
        .insert_resource(Msaa::Off) //Removes lines between assets
        .run();
}

//          COMPONENTS

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

#[derive(Component)]
struct CameraSpeed {
    speed: f32,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);


fn setup( //Sets up the map
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load(TERRAIN_SHEET_PATH);

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_WIDTH as u32, TILE_HEIGHT as u32), 12, 16, None, None);

    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn(CameraSpeed {speed: CAM_SPEED_MIN}); //instantiate the cameraspeed structure with a default speed
    
    //let animation_indices = AnimationIndices { first: 1, last: 6 }; //Unused currently
    commands.spawn(Camera2dBundle::default()).insert(PanCam::default());

    generate_new_map(commands, texture, texture_atlas_layout,None);
}

fn generate_new_map(mut commands: Commands, texture: Handle<Image>, texture_atlas_layout: Handle<TextureAtlasLayout>, query: Option<Query<Entity, With<TileBaseType>>>) { //Generate the map
    //Generates perlin noise for the map
    if let Some(query) = query {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }

    
    let mut perlin_rand = rand::thread_rng();
    let base_perlin = Perlin::new(perlin_rand.gen());
    let forest_perlin = Perlin::new(perlin_rand.gen());
    let dark_forest_perlin = Perlin::new(perlin_rand.gen());
    let jungle_perlin = Perlin::new(perlin_rand.gen());
    let dirt_perlin = Perlin::new(perlin_rand.gen());
    let mud_perlin = Perlin::new(perlin_rand.gen());
    let pond_perlin = Perlin::new(perlin_rand.gen());
    let islands_perlin = Perlin::new(perlin_rand.gen());


    //Create tiles and use perlin noise to fill in the tilebasetype.
    let mut tiles = HashSet::new();
    for x in 0..GRID_COLS {
        for y in 0..GRID_ROWS {
            let mut value = base_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let tile_base_type = if value <= 0.10 {
                let normalized_rand = Normal::new(0.0,0.02).unwrap(); //mean 0, std deviation 0.04
                let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
                value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &0.14);

                    match value {
                        v if v < 0.01 => TileBaseType::DeepWater,
                        _ => TileBaseType::Water,
                    }
            } else {
                let normalized_rand = Normal::new(0.0,0.05).unwrap(); //mean 0, std deviation 0.10
                let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
                value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
    
                match value {
                    v if v < 0.25 => TileBaseType::Sand,
                    v if v < 0.65 => TileBaseType::Plains,
                    v if v < 0.85 => TileBaseType::DarkMountain,
                    _ => TileBaseType::Mountain,
                }
            };

            //Generate forests
            let mut value = forest_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap(); //mean 0, std deviation 0.10
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && tile_base_type == TileBaseType::Plains) => TileBaseType::Forest,
                _ => {tile_base_type}
            };
            
            //Generate dark forests
            let mut value = dark_forest_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap(); //mean 0, std deviation 0.10
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && tile_base_type == TileBaseType::Plains) => TileBaseType::DarkForest,
                _ => {tile_base_type}
            };
            
            //Generate jungles
            let mut value = jungle_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap(); //mean 0, std deviation 0.10
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && (tile_base_type == TileBaseType::Plains || tile_base_type == TileBaseType::DarkMountain)) => TileBaseType::Jungle,
                _ => {tile_base_type}
            };
            
            //Generate dirt
            let mut value = dirt_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap(); //mean 0, std deviation 0.10
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && tile_base_type == TileBaseType::Plains) => TileBaseType::DarkDirt,
                v if (v > 0.8 && tile_base_type == TileBaseType::Plains) => TileBaseType::Dirt,
                _ => {tile_base_type}
            };
            
            //Generate mud
            let mut value = mud_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap(); //mean 0, std deviation 0.10
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && tile_base_type == TileBaseType::Sand) => TileBaseType::Mud,
                _ => {tile_base_type}
            };

            //Generate ponds
            let mut value = pond_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap(); //mean 0, std deviation 0.10
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && (tile_base_type == TileBaseType::Plains || tile_base_type == TileBaseType::DarkMountain || tile_base_type == TileBaseType::Mountain)) => TileBaseType::Pond,
                v if (v > 0.9 && tile_base_type == TileBaseType::Sand) => TileBaseType::Mud,
                v if (v > 0.8 && (tile_base_type == TileBaseType::Plains || tile_base_type == TileBaseType::DarkMountain || tile_base_type == TileBaseType::Mountain)) => TileBaseType::Mud,
                _ => {tile_base_type}
            };

            //Generate islands
            let value = islands_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let tile_base_type = match value {
                v if (v > 0.95 && (tile_base_type == TileBaseType::DeepWater || tile_base_type == TileBaseType::Water)) => TileBaseType::Sand,
                v if (v > 0.85 && (tile_base_type == TileBaseType::DeepWater || tile_base_type == TileBaseType::Water)) => TileBaseType::Water,
                _ => {tile_base_type}
            };


            //Generate resources
            let resource_value: f64 = perlin_rand.gen();
            let resource_type = match resource_value {
                v if v < 0.993 => "none",
                v if v < 0.995 => "outpost",
                v if v < 0.997 => "enemy",
                _ => "harvest",
            };

            let mut resource_data = (
                OutpostTile{ outpost_type: OutpostType::None, sales_mod: 0 },
                HarvestableTile{yields: ResourceItemType::None, amount_mod: 0},
                EnemyTile { enemy_type: EnemyTileType::None, yields: [ResourceItemType::None,ResourceItemType::None,ResourceItemType::None,ResourceItemType::None,ResourceItemType::None], amount_mod: 0, health: 0, damage: 0 });


            if resource_type == "outpost" {
                let sales_mod_rand = rand::thread_rng().gen_range(0..=100);
                resource_data.0 = match tile_base_type {
                    v if (v == TileBaseType::Water || v == TileBaseType::DeepWater || v == TileBaseType::Pond || v == TileBaseType::River) => OutpostTile{ outpost_type: OutpostType::ShipOutpost, sales_mod: sales_mod_rand },
                    v if (v == TileBaseType::DarkMountain || v == TileBaseType::Mountain) => OutpostTile{ outpost_type: OutpostType::MountainOutpost, sales_mod: sales_mod_rand },
                    v if (v == TileBaseType::Forest || v == TileBaseType::Sand || v == TileBaseType::Jungle || v == TileBaseType::DarkForest) => OutpostTile{ outpost_type: OutpostType::ForestOutpost, sales_mod: sales_mod_rand },
                    _ => {OutpostTile{ outpost_type: OutpostType::Outpost, sales_mod: sales_mod_rand }}
                };
            } else if  resource_type == "enemy" {
                let amount_mod_rand = rand::thread_rng().gen_range(0..=100);
                let health_mod_rand = rand::thread_rng().gen_range(0..=100);
                let damage_mod_rand = rand::thread_rng().gen_range(0..=100);
                resource_data.2 = match tile_base_type {
                    v if (v == TileBaseType::Water || v == TileBaseType::DeepWater || v == TileBaseType::Pond || v == TileBaseType::River) => EnemyTile { enemy_type: EnemyTileType::PirateShip, yields: ([ResourceItemType::None,ResourceItemType::None,ResourceItemType::None,ResourceItemType::None,ResourceItemType::None]), amount_mod: amount_mod_rand, health: health_mod_rand, damage: damage_mod_rand },
                    _ => EnemyTile { enemy_type: EnemyTileType::BeepleBillage, yields: ([ResourceItemType::None,ResourceItemType::None,ResourceItemType::None,ResourceItemType::None,ResourceItemType::None]), amount_mod: amount_mod_rand, health: health_mod_rand, damage: damage_mod_rand },
                };
            } else if resource_type == "harvest" {
                //let amount_mod_rand = rand::thread_rng().gen_range(0..=100);
                let rand_item: ResourceItemType = rand::random();
                let rand_item_type = match rand_item {
                    ResourceItemType::Nut(_) => ResourceItemType::Nut(rand::random::<NutType>()),
                    ResourceItemType::Fruit(_) => ResourceItemType::Fruit(rand::random::<FruitType>()),
                    ResourceItemType::Bush(_) => ResourceItemType::Bush(rand::random::<BushType>()),
                    ResourceItemType::Spice(_) => ResourceItemType::Spice(rand::random::<SpiceType>()),
                    _ => ResourceItemType::Mine(rand::random::<MineType>()), //mine
                };
                let rand_amount = rand::random();
                if !(tile_base_type == TileBaseType::Water || tile_base_type == TileBaseType::DeepWater || tile_base_type == TileBaseType::Pond || 
                    tile_base_type == TileBaseType::River) {
                    resource_data.1 = HarvestableTile {yields: rand_item_type, amount_mod: rand_amount};
                } else {
                    resource_data.1 = HarvestableTile {yields: ResourceItemType::Spice(SpiceType::Seamint), amount_mod: rand_amount};
                }
                
            };
            // Insert into the tiles collection as a point and a tile type enum
            tiles.insert(((x, y), tile_base_type, resource_data.0, resource_data.1, resource_data.2));
        }
    }

    //Go through each tile, scale the grid locations to the world location using tile width and scale factor, then get the sprite based on the tilebasetype enum.
    for ((x,y), tile_base_type, outpost_data, harvest_data, enemy_data) in tiles.iter() {
        let (x, y) = grid_to_world(*x as f32, *y as f32);

        let tile_randomizer = rand::thread_rng().gen_range(0..=3); //4 possible max states, most tiles use less hence the clamp.

        let texture_index = match tile_base_type {
            TileBaseType::Plains => {0 + *clamp(&tile_randomizer, &0, &2) as usize},
            TileBaseType::Forest => {3 + *clamp(&tile_randomizer, &0, &2) as usize},
            TileBaseType::DarkForest => {6 + *clamp(&tile_randomizer, &0, &2) as usize},
            TileBaseType::Jungle => {9 + *clamp(&tile_randomizer, &0, &2) as usize},
            TileBaseType::Mountain => {12 + *clamp(&tile_randomizer, &0, &2) as usize},
            TileBaseType::DarkMountain => {15 + *clamp(&tile_randomizer, &0, &2) as usize},
            TileBaseType::Sand => {18 + *clamp(&tile_randomizer, &0, &2) as usize},
            TileBaseType::Dirt => {21 + *clamp(&tile_randomizer, &0, &2) as usize},
            TileBaseType::DarkDirt => {24 + *clamp(&tile_randomizer, &0, &1) as usize},
            TileBaseType::Mud => {26 + *clamp(&tile_randomizer, &0, &1) as usize},
            TileBaseType::Water => {28 + *clamp(&tile_randomizer, &0, &1) as usize},
            TileBaseType::DeepWater => {30 + *clamp(&tile_randomizer, &0, &1) as usize},
            TileBaseType::River => {32 + *clamp(&tile_randomizer, &0, &1) as usize},
            TileBaseType::Pond => {34 + *clamp(&tile_randomizer, &0, &1) as usize},
        };

        let resource_index = if outpost_data.outpost_type != OutpostType::None {
            let tile_resource_type = outpost_data.outpost_type;
            let rand_colorval = rand::thread_rng().gen_range(0..=11); //0 to 11 inclusive
            let resource_index = match tile_resource_type {
                OutpostType::ForestOutpost => {48 + rand_colorval},
                OutpostType::MountainOutpost => {60 + rand_colorval},
                OutpostType::ShipOutpost => {72 + rand_colorval},
                _ => {36 + rand_colorval},
            };
            resource_index
        } else if harvest_data.yields != ResourceItemType::None {
            let tile_resource_type = &harvest_data.yields;
            //let rand_colorval = rand::thread_rng().gen_range(0..=11); //0 to 11 inclusive
            let resource_index = match tile_resource_type {
                ResourceItemType::Nut(_) => {120 + rand::thread_rng().gen_range(0..=2)},
                ResourceItemType::Bush(_) => {132 + rand::thread_rng().gen_range(0..=8)},
                ResourceItemType::Fruit(_) => {108 + rand::thread_rng().gen_range(0..=11)},
                ResourceItemType::Spice(_) => {144 + rand::thread_rng().gen_range(0..=4)},
                _ => {96 + rand::thread_rng().gen_range(0..=4)}, //Mine
            };
            resource_index
        } else if enemy_data.enemy_type != EnemyTileType::None {
            let tile_resource_type = &enemy_data.enemy_type;
            let resource_index = match tile_resource_type {
                EnemyTileType::BeepleBillage => {85},
                _ => {84}, //pirate ship
            };
            resource_index
        } else {
            192 //default = none
        };

        
        
        commands.spawn(( //Spawns the texture with the given texture index calculated previously
            SpriteBundle {
                transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR as f32)).with_translation(vec3(x as f32, y as f32, 0.0)),
                texture: texture.clone(), //cloning is performance hit -- necessary?
                ..default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(), //cloning is performance hit -- necessary?
                index: texture_index,
            },
            Tile {
                location :(x,y), 
                base_type: *tile_base_type, 
                outpost: *outpost_data,
                harvest: *harvest_data,
                enemy: *enemy_data ,
            },
            // animation_indices,
            // AnimationTimer(Timer::from_seconds(0.4, TimerMode::Repeating)),
        ));

        if resource_index < 191 { //if it has a valid resource

            commands.spawn(( //Spawns the resource texture with the given resource index calculated previously
                SpriteBundle {
                    transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR as f32)).with_translation(vec3(x as f32, y as f32, 0.0)),
                    texture: texture.clone(), //cloning is performance hit -- necessary?
                    ..default()
                },
                TextureAtlas {
                    layout: texture_atlas_layout.clone(), //cloning is performance hit -- necessary?
                    index: resource_index,
                },
                // animation_indices,
                // AnimationTimer(Timer::from_seconds(0.4, TimerMode::Repeating)),
            ));
        }
    }
}

fn mouse_input_handler(
    mut cursor_coords: ResMut<CursorWorldCoords>,
    tile_query: Query<&Tile>, 
    buttons: Res<ButtonInput<MouseButton>>,
    windows_query: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {

    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = windows_query.single();

    

    if buttons.just_pressed(MouseButton::Left) {


        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
        {
            cursor_coords.0 = world_position;
            for tile in tile_query.iter() {
                    //Shift value x by -24, -24 and value 2 (outer value) by +24, +24
                if world_position.x > tile.location.0 - 24. && world_position.x < (tile.location.0 + TILE_WIDTH as f32 * 3.) && 
                    world_position.y > tile.location.1 - 24. && world_position.y < (tile.location.1 + TILE_HEIGHT as f32 * 3.) {
                    
                    println!("\n->cursor coords: {}/{}", world_position.x, world_position.y);
                    println!("Tile base type: {:?}\nTile resources:{:?},{:?},{:?}\nTile location: {},{}", tile.base_type,tile.enemy.enemy_type,tile.harvest.yields,tile.outpost.outpost_type, tile.location.0, tile.location.1);
                }
            }
        };
    }
}


fn reload_on_r( //Reload map textures on 'r' press
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    keys: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, With<TileBaseType>>,
) {
    for entities in &query {
        commands.entity(entities).despawn(); //despawn all entities
    };
    let texture: Handle<Image> = asset_server.load(TERRAIN_SHEET_PATH);

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_WIDTH as u32, TILE_HEIGHT as u32), 12, 16, None, None);

    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    if keys.pressed(KeyCode::KeyR) {
        generate_new_map(commands, texture, texture_atlas_layout, Some(query));
    }
}

fn update_camera( //Allows for movement-key control for the camera
    mut camera: Query<&mut Transform, With<Camera2d>>,
    mut query: Query<&mut CameraSpeed>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut camera) = camera.get_single_mut() else { //Make sure there's just 1 camera
        return;
    };
    let Ok(mut cam_speed) = query.get_single_mut() else {return;}; //And camera speed

    //let Vec3 { x, y, .. } = Vec3::new(0.0,0.0,0.0);

    if keys.pressed(KeyCode::Space) { //Space resets camera to 0,0,0
        let direction = Vec3::new(0.0,0.0,0.0);
        camera.translation = camera.translation.lerp(direction, time.delta_seconds() * CAM_LERP_FACTOR);
    }
    if keys.pressed(KeyCode::KeyD) {
        cam_speed.speed += 35.;
        camera.translation.x += cam_speed.speed * time.delta_seconds();
    }
    if keys.pressed(KeyCode::KeyA) {
        cam_speed.speed += 35.;
        camera.translation.x -= cam_speed.speed * time.delta_seconds();
    }
    if keys.pressed(KeyCode::KeyW) {
        cam_speed.speed += 35.;
        camera.translation.y += cam_speed.speed * time.delta_seconds();
    }
    if keys.pressed(KeyCode::KeyS) {
        cam_speed.speed += 35.;
        camera.translation.y -= cam_speed.speed * time.delta_seconds();
    }

    if cam_speed.speed > CAM_SPEED_MIN { //Cam speed falls constantly unless a key is being pressed, and can't go higher than the max
        cam_speed.speed -= 25.;
    }
    if cam_speed.speed >= CAM_SPEED_MAX {
        cam_speed.speed = CAM_SPEED_MAX;
    }
}


fn animate_sprite( //Currently not used but will probably use later
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}


fn grid_to_world(x:f32,y:f32) -> (f32,f32) { //Returns the new x,y coordinates as scaled based on sprite data
    (x * TILE_WIDTH as f32 * SPRITE_SCALE_FACTOR as f32, y * TILE_HEIGHT as f32 * SPRITE_SCALE_FACTOR as f32)
}
fn clamp<'a, T: PartialOrd>(x: &'a T, min: &'a T, max: &'a T) -> &'a T { //Clamps a partially ordered value between a min and max value, inclusive.
    if x >= max {
        return max;
    } else if x <= min {
        return min;
    }
    x
}