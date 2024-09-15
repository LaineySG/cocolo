//          IMPORTS

use std::collections::HashMap;
use bevy::ecs::world;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::{
    color::palettes::css::*,
    input::mouse::MouseButtonInput, math::vec3, prelude::*, utils::HashSet,
    color::palettes::basic::*,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use noise::{NoiseFn, Perlin};
use rand::seq::SliceRandom;
use rand::Rng;
use bevy::window::PrimaryWindow;
use bevy_pancam::{PanCamPlugin, PanCam};
use rand_distr::{Normal, Distribution};
use bevy_simple_scroll_view::*;

//bevy ecs tilemap stuff
use bevy::{color::palettes, math::Vec4Swizzles};
use bevy::{ecs::system::Resource};

mod tile_data;
use tile_data::*;

//bevy egui
use bevy_egui::{egui, EguiContexts, EguiPlugin};



//          CONSTANTS

///sprite sheet image path.
const SPRITE_SHEET_PATH: &str = "map.png";
///Pixel width of a tile in the spritesheet.
const TILE_WIDTH: usize = 8;
///Pixel height of a tile in the spritesheet.
const TILE_HEIGHT: usize = 8;
///Sprite scale factor for a tile in the spritesheet, converted to the worldmap.
const SPRITE_SCALE_FACTOR: usize = 6;
///Number of columns on the spritesheet.
const MAP_COLS: u32 = 12;
///Number of rows on the spritesheet.
const MAP_ROWS: u32 = 24;

///Map grid number of columns.
pub const GRID_COLS:usize = 500;
///Map grid number of rows.
pub const GRID_ROWS:usize = 500;
///Map grid perlin noise scale.
const PERLIN_NOISE_SCALE: f64 = 65.; //was 10

///Camera lerp smoothing factor.
const CAM_LERP_FACTOR: f32 = 4.0;
///Camera minimum speed from arrowkeys.
const CAM_SPEED_MIN: f32 = 300.;
///Camera maximum speed from arrowkeys.
const CAM_SPEED_MAX: f32 = 1500.;

///Standard font
const TEXT_FONT:&str = "fonts/Starborn.ttf";
///Standard text size
const TEXT_SIZE_STANDARD:f32 = 16.;
///Standard text color
const TEXT_COLOR_STANDARD:Color = Color::WHITE;
///Highlight text size
const TEXT_SIZE_HIGHLIGHT:f32 = 20.;
///Highlight text color
const TEXT_COLOR_HIGHLIGHT:Color = Color::srgba(0.82, 0.06, 0.23, 1.);


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(ScrollViewPlugin)
        .add_plugins((FrameTimeDiagnosticsPlugin,))
        .add_systems(Startup, init_msg_ui)
        .add_systems(Startup, setup) //Map loading/setup system
        .add_systems(FixedUpdate, update_camera) //Camera control
        .add_systems(FixedUpdate, mouse_input_handler) //mouse input handler
        .add_systems(FixedUpdate,keyboard_handler)
        .add_systems(FixedUpdate, gui_buttons)
        .init_resource::<CursorWorldCoords>()
        .add_plugins(PanCamPlugin) //Adds zoom and mouse-pan
        .insert_resource(Msaa::Off) //Removes lines between assets

        .add_plugins(EguiPlugin)

        .run();
}

//          COMPONENTS

/// World coordinates of the mouse position
#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

/// Current speed of the camera.
#[derive(Component)]
struct CameraSpeed {
    speed: f32,
}
/// Player information storage.
#[derive(Component)]
struct PlayerStats {
    gold: i32, //currency traded for with outposts
    knowledge:i32, //research unlocked by hiring researchers to research ingredient tiles.
    loonkas: Vec<Loonka>,
}
#[derive(Component)]
///UI Node storage for comparisons.
struct UINode {
    name: String,
    id: Entity,
}
///Name component for comparisons.
#[derive(Component)]
struct Name(String);


///Sets up the game and map.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    //create the map size, scale, and grid.
    let texture: Handle<Image> = asset_server.load(SPRITE_SHEET_PATH);
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_WIDTH as u32, TILE_HEIGHT as u32), MAP_COLS, MAP_ROWS, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    //Spawns and instantiates camera and player information variables.
    commands.spawn(CameraSpeed {speed: CAM_SPEED_MIN});
    commands.spawn(PlayerStats {gold:0,knowledge:0,loonkas:vec![]});
    commands.spawn(Camera2dBundle::default()).insert(PanCam::default());

    //Generate the map
    generate_new_map(commands, texture, texture_atlas_layout,None);
}

///Generates the map tiles, data, and resources.
fn generate_new_map(
    mut commands: Commands, 
    texture: Handle<Image>, 
    texture_atlas_layout: Handle<TextureAtlasLayout>, 
    tiles_q: Option<Query<Entity, With<TileBaseType>>>)
{
    //despawn all tiles if they exist
    if let Some(query) = tiles_q {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }

    
    //Generates set of perlin noises for the map
    let mut perlin_rand = rand::thread_rng();
    let base_perlin = Perlin::new(perlin_rand.gen());
    let forest_perlin = Perlin::new(perlin_rand.gen());
    let dark_forest_perlin = Perlin::new(perlin_rand.gen());
    let jungle_perlin = Perlin::new(perlin_rand.gen());
    let dirt_perlin = Perlin::new(perlin_rand.gen());
    let mud_perlin = Perlin::new(perlin_rand.gen());
    let pond_perlin = Perlin::new(perlin_rand.gen());
    let islands_perlin = Perlin::new(perlin_rand.gen());


    //Create base tiles and use perlin noise to fill in the tilebasetype.
    let mut tiles = HashSet::new();
    for x in 0..GRID_COLS {
        for y in 0..GRID_ROWS {
            let mut value = base_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let tile_base_type = if value <= 0.10 {
                let normalized_rand = Normal::new(0.0,0.02).unwrap();
                let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
                value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &0.14);

                    match value {
                        v if v < 0.01 => TileBaseType::DeepWater,
                        _ => TileBaseType::Water,
                    }
            } else {
                let normalized_rand = Normal::new(0.0,0.05).unwrap();
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
            let normalized_rand = Normal::new(0.0,0.05).unwrap();
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && tile_base_type == TileBaseType::Plains) => TileBaseType::Forest,
                _ => {tile_base_type}
            };
            
            //Generate dark forests
            let mut value = dark_forest_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap();
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && tile_base_type == TileBaseType::Plains) => TileBaseType::DarkForest,
                _ => {tile_base_type}
            };
            
            //Generate jungles
            let mut value = jungle_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap();
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && (tile_base_type == TileBaseType::Plains || tile_base_type == TileBaseType::DarkMountain)) => TileBaseType::Jungle,
                _ => {tile_base_type}
            };
            
            //Generate dirt
            let mut value = dirt_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap();
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && tile_base_type == TileBaseType::Plains) => TileBaseType::DarkDirt,
                v if (v > 0.8 && tile_base_type == TileBaseType::Plains) => TileBaseType::Dirt,
                _ => {tile_base_type}
            };
            
            //Generate mud
            let mut value = mud_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap();
            let normal_rng_val = normalized_rand.sample(&mut rand::thread_rng());
            value = *clamp(&(value + (normal_rng_val) as f64), &0.00, &1.00);
            let tile_base_type = match value {
                v if (v > 0.9 && tile_base_type == TileBaseType::Sand) => TileBaseType::Mud,
                _ => {tile_base_type}
            };

            //Generate ponds
            let mut value = pond_perlin.get([(x as f64 /PERLIN_NOISE_SCALE),(y as f64 / PERLIN_NOISE_SCALE)]);
            let normalized_rand = Normal::new(0.0,0.05).unwrap();
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
                resource_data.0 = OutpostTile{ outpost_type: OutpostType::get_rand(tile_base_type), sales_mod: sales_mod_rand };
            } else if  resource_type == "enemy" {
                let amount_mod_rand = rand::thread_rng().gen_range(0..=100);
                let health_mod_rand = rand::thread_rng().gen_range(0..=100);
                let damage_mod_rand = rand::thread_rng().gen_range(0..=100);
                resource_data.2 = EnemyTile { enemy_type: EnemyTileType::get_rand(tile_base_type), yields: ([(ResourceItemType::get_rand(TileBaseType::All)),ResourceItemType::get_rand(TileBaseType::All),ResourceItemType::get_rand(TileBaseType::All),ResourceItemType::get_rand(TileBaseType::All),ResourceItemType::get_rand(TileBaseType::All)]), amount_mod: amount_mod_rand, health: health_mod_rand, damage: damage_mod_rand }
                
            } else if resource_type == "harvest" {
                let rand_item: ResourceItemType = ResourceItemType::get_rand(tile_base_type);
                let rand_item_type = match rand_item {
                    ResourceItemType::Nut(_) => ResourceItemType::Nut(NutType::get_rand(tile_base_type)),
                    ResourceItemType::Fruit(_) => ResourceItemType::Fruit(FruitType::get_rand(tile_base_type)),
                    ResourceItemType::Bush(_) => ResourceItemType::Bush(BushType::get_rand(tile_base_type)),
                    ResourceItemType::Spice(_) => ResourceItemType::Spice(SpiceType::get_rand(tile_base_type)),
                    _ => ResourceItemType::Mine(MineType::get_rand(tile_base_type)),
                };
                let rand_amount = rand::random();
                resource_data.1 = HarvestableTile {yields: rand_item_type, amount_mod: rand_amount};
                
            };
            // Insert into the tiles collection as a point and a tile type enum
            tiles.insert(((x, y), tile_base_type, resource_data.0, resource_data.1, resource_data.2));
        }
    }

    //Go through each tile, scale the grid locations to the world location using tile width and scale factor, then get the sprite based on the tilebasetype enum and draw it.
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
            _ => {210}, //shouldn't happen ever since all is only used internally
        };

        //Gets the random tile type or color.
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
                ResourceItemType::Spice(SpiceType::Seamint) => 149, //seamint has a unique icon
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
                texture: texture.clone(),
                ..default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: texture_index,
            },
            Tile {
                location :(x,y), 
                base_type: *tile_base_type, 
                outpost: *outpost_data,
                harvest: *harvest_data,
                enemy: *enemy_data,
                loonkas: vec![],
            },
        ));

        if resource_index < 191 { //if it has a valid resource

            commands.spawn(( //Spawns the resource texture with the given resource index calculated previously
                SpriteBundle {
                    transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR as f32)).with_translation(vec3(x as f32, y as f32, 0.0)),
                    texture: texture.clone(),
                    ..default()
                },
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: resource_index,
                },
            ));
        }
    }
}

///Creates a new loonka for the player.
fn create_loonka(
    mut player_stat_q: Query<&mut PlayerStats>,
) {
    //get single player stat query
    let Ok(mut player_stats) = player_stat_q.get_single_mut() else {return;};

    let loonka_names = vec!["Toby".to_string(),"Richard".to_string(),"Raffielli".to_string(),"Mark".to_string(),"Reese".to_string(),"Alice".to_string(),
    "Lisa".to_string(),"Tara".to_string(),"Primrose".to_string(),"Xantu".to_string(), "Patrick".to_string(), "Cody".to_string(), "Dani".to_string(), "Quinn".to_string(),
    "Lainey".to_string(), "Chloe".to_string()];
    
    let currentid = player_stats.loonkas.len();
    let name = loonka_names.choose(&mut rand::thread_rng()).unwrap().clone();
    player_stats.loonkas.push(Loonka{
        name: name,
        id: currentid+1,
        current_job: LoonkaJob::None,
        vigor: rand::thread_rng().gen_range(25..=100),
        speed: rand::thread_rng().gen_range(25..=100),
        dexterity: rand::thread_rng().gen_range(25..=100),
        strength: rand::thread_rng().gen_range(25..=100),
        charisma: rand::thread_rng().gen_range(25..=100),
        intellect: rand::thread_rng().gen_range(25..=100),
        icon_num: rand::thread_rng().gen_range(0..=11),
    });
    // println!("loonkas: {:?}",player_stats.loonkas);
}

///Place a loonka (WIP)
fn place_loonka(
    mut commands: Commands, 
    texture: Handle<Image>, 
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    tile_q: Query<&Tile>, 
    mut player_stat_q: Query<&mut PlayerStats>, 
    mut clicked_tile: Tile) {

    // let Ok(mut player_stats) = player_stats.get_single_mut() else {return;};



    // //get texture index
    
    // commands.spawn(( //Spawns the texture with the given texture index calculated previously
    //     SpriteBundle {
    //         transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR as f32)).with_translation(vec3(x as f32, y as f32, 0.0)),
    //         texture: texture.clone(), //cloning is performance hit -- necessary?
    //         ..default()
    //     },
    //     TextureAtlas {
    //         layout: texture_atlas_layout.clone(), //cloning is performance hit -- necessary?
    //         index: texture_index,
    //     },
    //     Tile {
    //         location :(x,y), 
    //         base_type: *tile_base_type, 
    //         outpost: *outpost_data,
    //         harvest: *harvest_data,
    //         enemy: *enemy_data ,
    //     },
    //     // animation_indices,
    //     // AnimationTimer(Timer::from_seconds(0.4, TimerMode::Repeating)),
    // ));

}

///Handles keyboard input
fn keyboard_handler(
    commands:Commands,
    keys: Res<ButtonInput<KeyCode>>, 
    player_stat_q: Query<&mut PlayerStats>,
    asset_server: Res<AssetServer>,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    tilebasetype_q: Query<Entity, With<TileBaseType>>,
    uinode_q: Query<&mut UINode>,
    scrolls_q: Query<&mut ScrollableContent>,
    style_q: Query<&mut Style>,
    contexts: EguiContexts,
) {
    if keys.just_pressed(KeyCode::KeyB) {
        create_loonka(player_stat_q);
    } else if keys.just_pressed(KeyCode::KeyR) {
        reload_on_r(commands, asset_server, texture_atlas_layouts, tilebasetype_q)
    } else if keys.just_pressed(KeyCode::KeyF) {
        new_message(scrolls_q, commands, asset_server, uinode_q)
    } else if keys.just_pressed(KeyCode::Tab) {
        load_inventory(commands,asset_server,uinode_q,player_stat_q,style_q,None)
    }
}

//Sends player a new message (WIP)
fn new_message(
    mut scrolls_q: Query<&mut ScrollableContent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    uinode_q: Query<&mut UINode>,
) {
    let font = asset_server.load(TEXT_FONT);
    let font_size = TEXT_SIZE_STANDARD;
    let font_color = TEXT_COLOR_STANDARD;
    
        let new_msg = commands.spawn((TextBundle::from_section(
            "New message!",
            TextStyle {
                font: font.clone(),
                font_size: font_size,
                color: font_color,
                ..default()
            },
        ),
        )).id();
        let msg_gap = commands.spawn(NodeBundle { //DIY line-gap
                style: Style {
                    height: Val::Px(4.),
                    ..default()
                },
                ..default()
            }).id();

    let text_area = {
        let mut res = None;
        for ui_node in uinode_q.iter() {
             if ui_node.name == "text_area".to_string() {res = Some(ui_node.id)};
        }
        res
    };
    let text_area = text_area.unwrap();
        commands
            .entity(text_area)
            .push_children(&[new_msg])
            .push_children(&[msg_gap]);


    for mut scroll in scrolls_q.iter_mut() { //scroll down in textbox
            scroll.pos_y -= 20.;
            //println!("scroll pos: {}",scroll.pos_y);
    }
    
}

///Loads the inventory, or reloads if reload is set to true.
fn load_inventory(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    uinode_q: Query<&mut UINode>, 
    player_stat_q: Query<&mut PlayerStats>,
    mut style_q: Query<&mut Style>,
    reload: Option<bool>,
) {
    let reload_bool = match reload {Some(val) => {val},None => {false}};

    let inv_list_container = {
        let mut res = None;
        for ui_node in uinode_q.iter() {
            if ui_node.name == "inv_list_container".to_string() {res = Some(ui_node.id)};
        }
        res
    };
    let inv_list_container = inv_list_container;
        match inv_list_container {
            Some(x) => {
                if let Ok(style) = style_q.get_mut(inv_list_container.unwrap()) {
                    if reload_bool {
                        if style.display == Display::Block {
                            commands.entity(x).insert(Style {
                                display: Display::None,
                                ..Default::default()
                            });
                            refresh_inventory(commands,asset_server,uinode_q,player_stat_q);
                        }} else {
                            if style.display == Display::Block {
                                commands.entity(x).insert(Style {
                                    display: Display::None,
                                    ..Default::default()
                                });
                            } else {
                                refresh_inventory(commands,asset_server,uinode_q,player_stat_q);
                            }

                        }
                } else {
                    refresh_inventory(commands,asset_server,uinode_q,player_stat_q);
                }
            },
            None => {
                refresh_inventory(commands,asset_server,uinode_q,player_stat_q);
            }
        }
    
}

///Handles GUI button interactions.
fn gui_buttons(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut interaction_q: Query<
            (&Interaction,
            &mut UiImage, &mut Loonka),
        (Changed<Interaction>, With<Button>),
    >,
    uinode_q: Query<&mut UINode>,
    mut player_stats_q: Query<&mut PlayerStats>,
    style_q: Query<&mut Style>,
) {    
    let Ok(mut player_stats) = player_stats_q.get_single_mut() else {return;};

    let mut old_job = LoonkaJob::None; 
    let mut new_job_ex = LoonkaJob::None;

    for (interaction, mut img, mut loonka) in &mut interaction_q {
        match *interaction {
            Interaction::Pressed => {
                let mut idx = loonka.current_job as usize + 1; 
                if idx > 7 {idx = 0}; //loop around if it's out of range.
                if let Some(new_job) = LoonkaJob::from_index(idx) {
                    *img = UiImage::new(asset_server.load(format!("ui/loonkas/{:?}.png", new_job)));
                    old_job = loonka.current_job;
                    loonka.current_job = new_job;
                    new_job_ex = new_job;
                    for playerloonka in player_stats.loonkas.iter_mut() {
                        if playerloonka.id == loonka.id {
                            playerloonka.current_job = new_job;
                        }
                    }
                } else {
                    println!("Invalid index for LoonkaJob: {}", idx);
                }
            }
            Interaction::Hovered => {
                    *img = UiImage::new(asset_server.load("ui/loonkas/Hover.png"));
            }
            Interaction::None => {
                *img = UiImage::new(asset_server.load(format!("ui/loonkas/{:?}.png",loonka.current_job)));
            }
        }
    }
    if old_job != new_job_ex {
        load_inventory(commands,asset_server,uinode_q,player_stats_q,style_q, Some(true));
    }
}

///Refreshes the inventory UI.
fn refresh_inventory(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    uinode_q: Query<&mut UINode>,
    mut player_stat_q: Query<&mut PlayerStats>,
    ) {
    
    let font = asset_server.load(TEXT_FONT);
    let font_size = TEXT_SIZE_STANDARD;
    let font_color = TEXT_COLOR_STANDARD;
    let highlight_font_size = TEXT_SIZE_HIGHLIGHT;
    let highlight_font_color = TEXT_COLOR_HIGHLIGHT;

    let Ok(player_stats) = player_stat_q.get_single_mut() else {return;};


    let root_uinode = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .id();

    let mut inv_base = root_uinode; //set up textarea here to keep it in scope. Assign rootnode to be overwritten later.
    let inv_list_container = commands.spawn((NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(80.),
            display: Display::Block,
            height: Val::Percent(60.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            margin: UiRect::axes(Val::Auto, Val::Px(15.)),
            padding: UiRect::axes(Val::Px(5.), Val::Px(5.)),
            ..default()
        },
        border_radius: BorderRadius { top_left: (Val::Px(15.)), top_right: (Val::Px(15.)), bottom_left: (Val::Px(15.)), bottom_right: (Val::Px(15.))},
        background_color: Color::srgba(0.02, 0.06, 0.23, 0.8).into(),
        ..default()
    },
    ScrollView::default(),
    )).insert(Name("UIBox".to_string())).with_children(|p| {
    inv_base = p.spawn((
        NodeBundle {
            style: Style {
                flex_direction: bevy::ui::FlexDirection::Column,
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(5.), Val::Px(5.)),
                ..default()
            },
            transform: Transform::from_translation(vec3(0.0 as f32, 0.0 as f32, 0.0)),
            ..default()
        },
        ScrollableContent::default(),
    ))
    .with_children(|scroll_area| {



        for loonka in player_stats.loonkas.clone() {
            
            let mut _horizontal_row = scroll_area.spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: bevy::ui::FlexDirection::Row,
                        width: Val::Percent(100.0),
                        height: Val::Px(75.),
                        align_items:AlignItems::Center,
                        padding: UiRect::axes(Val::Px(5.), Val::Px(5.)),
                        ..default()
                    },
                    transform: Transform::from_translation(vec3(0.0 as f32, 0.0 as f32, 0.0)),
                    ..default()
                },
                ScrollableContent::default(),
            )).with_children(|x| {

                let mut _vertical_row = x.spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: bevy::ui::FlexDirection::Column,
                        width: Val::Percent(100.0),
                        height: Val::Px(75.),
                        padding: UiRect::axes(Val::Px(5.), Val::Px(5.)),
                        ..default()
                    },
                    transform: Transform::from_translation(vec3(0.0 as f32, 0.0 as f32, 0.0)),
                    ..default()
                },
                ScrollableContent::default(),
                )).with_children(|x| {
                    x.spawn((TextBundle::from_section(
                        format!("{}: {:?}",loonka.name, loonka.current_job),
                        TextStyle {
                            font: font.clone(),
                            font_size: highlight_font_size,
                            color: highlight_font_color,
                            ..default()
                        },
                    ),
                ));


                x.spawn(NodeBundle { //DIY line-gap
                    style: Style {
                        height: Val::Px(2.),
                        margin: UiRect::axes(Val::Px(0.), Val::Px(4.)),
                        ..default()
                    },
                    ..default()
                });

                    x.spawn((TextBundle::from_section(
                        format!("Speed: {}, Charisma: {}, Strength: {}, Dexterity: {}, Intellect: {}, Vigor: {}",loonka.speed,loonka.charisma,
                        loonka.strength,loonka.dexterity,loonka.intellect,loonka.vigor),
                        TextStyle {
                            font: font.clone(),
                            font_size: font_size,
                            color: font_color,
                            ..default()
                        },
                    ),
                ));  
                }

                );
                       

            x.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(75.0),
                        height: Val::Px(75.0),
                        margin: UiRect::axes(Val::Px(15.), Val::Px(2.)),
                        ..default()
                    },
                    image: UiImage::new(asset_server.load(format!("ui/loonkas/{:?}.png", loonka.current_job))),
                    ..default()
                },
                loonka,
            ));

            });
            
            scroll_area.spawn(NodeBundle { //DIY line-gap
                style: Style {
                    height: Val::Px(2.),
                    margin: UiRect::axes(Val::Px(0.), Val::Px(4.)),
                    ..default()
                },
                background_color: Color::srgba(0.02, 0.06, 0.23, 0.8).into(),
                ..default()
            });
           

        }
    }).id();
    }).id();

    commands.spawn(UINode {name: "inv_list_container".to_string(), id:inv_list_container}); //instantiate the root UInode
    commands.spawn(UINode {name: "inv_base".to_string(), id:inv_base}); //instantiate the root UInode

    let root_uinode = {
        let mut res = None;
        for ui_node in uinode_q.iter() {
            if ui_node.name == "root_uinode".to_string() {res = Some(ui_node.id)};
        }
        res
    };
    let root_uinode = root_uinode.unwrap();
    commands
        .entity(root_uinode)
        .push_children(&[inv_list_container]);
}

///Initiates the UI for the player message system.
fn init_msg_ui(
    mut commands: Commands, 
    asset_server: Res<AssetServer>
) {
    let font = asset_server.load(TEXT_FONT);
    let font_size = TEXT_SIZE_STANDARD;
    let font_color = TEXT_COLOR_STANDARD;

    let root_uinode = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            ..default()
        })
        .id();

    let mut text_area = root_uinode; //set up textarea here to keep it in scope. Assign rootnode to be overwritten later.
    let text_box = commands.spawn((NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(40.),
            height: Val::Percent(15.),
            align_items: AlignItems::Start,
            margin: UiRect::axes(Val::Px(15.), Val::Px(15.)),
            padding: UiRect::axes(Val::Px(5.), Val::Px(5.)),
            ..default()
        },
        border_radius: BorderRadius { top_left: (Val::Px(15.)), top_right: (Val::Px(15.)), bottom_left: (Val::Px(15.)), bottom_right: (Val::Px(15.))},
        background_color: Color::srgba(0.02, 0.06, 0.23, 0.6).into(),
        ..default()
    },
    ScrollView::default(),
)).insert(Name("UIBox".to_string())).with_children(|p| {
    text_area = p.spawn((
        NodeBundle {
            style: Style {
                flex_direction: bevy::ui::FlexDirection::Column,
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(5.), Val::Px(5.)),
                ..default()
            },
            ..default()
        },
        ScrollableContent::default(),
    ))
    .with_children(|scroll_area| {
        scroll_area.spawn((TextBundle::from_section(
                "You awaken on a strange island...",
                TextStyle {
                    font: font.clone(),
                    font_size: font_size,
                    color: font_color,
                    ..default()
                },
            ),
        ));
        scroll_area.spawn(NodeBundle { //DIY line-gap
            style: Style {
                height: Val::Px(4.),
                ..default()
            },
            ..default()
        });
        scroll_area.spawn((TextBundle::from_section(
            "You don't know where you are.",
                TextStyle {
                    font: font.clone(),
                    font_size: font_size,
                    color: font_color,
                    ..default()
                },
            ),
        ));
        scroll_area.spawn(NodeBundle { //DIY line-gap
            style: Style {
                height: Val::Px(4.),
                ..default()
            },
            ..default()
        });
        scroll_area.spawn((TextBundle::from_section(
            "You don't know where you came from.",
                TextStyle {
                    font: font.clone(),
                    font_size: font_size,
                    color: font_color,
                    ..default()
                },
            ),
        ));
        scroll_area.spawn(NodeBundle { //DIY line-gap
            style: Style {
                height: Val::Px(4.),
                ..default()
            },
            ..default()
        });
        scroll_area.spawn((TextBundle::from_section(
            "You know just one thing. You are thirsty!",
                TextStyle {
                    font: font.clone(),
                    font_size: font_size,
                    color: font_color,
                    ..default()
                },
            ),
        ));
        scroll_area.spawn(NodeBundle { //DIY line-gap
            style: Style {
                height: Val::Px(4.),
                ..default()
            },
            ..default()
        });
        scroll_area.spawn((TextBundle::from_section(
            "Try clicking some water to harvest it!",
                TextStyle {
                    font: font.clone(),
                    font_size: font_size,
                    color: font_color,
                    ..default()
                },
            ),
        ));
        scroll_area.spawn(NodeBundle { //DIY line-gap
            style: Style {
                height: Val::Px(4.),
                ..default()
            },
            ..default()
        });
    }).id();
}).id();
commands.spawn(UINode {name: "root_uinode".to_string(), id:root_uinode}); //instantiate the root UInode
commands.spawn(UINode {name: "text_box".to_string(), id:text_box}); //instantiate the root UInode
commands.spawn(UINode {name: "text_area".to_string(), id:text_area}); //instantiate the root UInode

    commands
        .entity(root_uinode)
        .push_children(&[text_box]);
}

///Handles mouse input.
fn mouse_input_handler(
    mut cursor_coords: ResMut<CursorWorldCoords>,
    tile_q: Query<&Tile>, 
    buttons: Res<ButtonInput<MouseButton>>,
    mut evr_scroll: EventReader<MouseWheel>,
    windows_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut uinode_q: Query<(Entity, &Style), With<Name>>, //align_items
    mut pancam_q: Query<&mut PanCam>,
) {

    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = camera_q.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = windows_q.single();

    for _evt in evr_scroll.read() { //Check for scroll events
        
        if let Some(cursor_position) = window.cursor_position() {
            for (_entity, style) in uinode_q.iter_mut() {
                let ui_node_position = Vec2::new(
                    match style.left {
                        Val::Px(px) => px,
                        Val::Percent(percent) => percent / 100.0 * window.size().x,
                        _ => 0.0,
                    },
                    85. / 100. * window.size().y //Text box is 15% height, so it is 85% down the window.
                );
    
                let ui_node_size = Vec2::new(
                    match style.width {
                        Val::Px(px) => px,
                        Val::Percent(percent) => percent / 100.0 * window.size().x,
                        _ => 0.0,
                    },
                    match style.height {
                        Val::Px(px) => px,
                        Val::Percent(percent) => percent / 100.0 * window.size().y,
                        _ => 0.0,
                    },
                );
    
                // Check if cursor is within UI node bounds
                if cursor_position.x >= ui_node_position.x
                    && cursor_position.x <= ui_node_position.x + ui_node_size.x
                    && cursor_position.y >= ui_node_position.y
                    && cursor_position.y <= ui_node_position.y + ui_node_size.y
                {
                    // If cursor overlaps with UI node, disable PanCam
                    for mut cam in pancam_q.iter_mut() {
                        cam.enabled = false;
                        // println!("disabled: {},{},{},{}",ui_node_position.x,ui_node_position.x+ui_node_size.x,ui_node_position.y,ui_node_position.y+ui_node_size.y);
                        // println!("cs_pos: {},{}",cursor_position.x,cursor_position.y);
                    }
                } else {
                    // Re-enable PanCam if cursor is not over UI
                    for mut cam in pancam_q.iter_mut() {
                        cam.enabled = true;
                        // println!("enabled: {},{},{},{}",ui_node_position.x,ui_node_position.x+ui_node_size.x,ui_node_position.y,ui_node_position.y+ui_node_size.y);
                        // println!("cs_pos: {},{}",cursor_position.x,cursor_position.y);
                    }
                }
            }
        }
    }
    if buttons.just_pressed(MouseButton::Left) { //Same as above but for drag-scrolling
        if let Some(cursor_position) = window.cursor_position() {

            for (_entity, style) in uinode_q.iter_mut() {

                let ui_node_position = Vec2::new(
                    match style.left {
                        Val::Px(px) => px,
                        Val::Percent(percent) => percent / 100.0 * window.size().x,
                        _ => 0.0,
                    },
                    85. / 100. * window.size().y //textbos is 15% of window size, so it's 85% down the screen since it's at the bottom.
                );
    
                let ui_node_size = Vec2::new(
                    match style.width {
                        Val::Px(px) => px,
                        Val::Percent(percent) => percent / 100.0 * window.size().x,
                        _ => 0.0,
                    },
                    match style.height {
                        Val::Px(px) => px,
                        Val::Percent(percent) => percent / 100.0 * window.size().y,
                        _ => 0.0,
                    },
                );
    
                // Check if cursor is within UI node bounds
                if cursor_position.x >= ui_node_position.x
                    && cursor_position.x <= ui_node_position.x + ui_node_size.x
                    && cursor_position.y >= ui_node_position.y
                    && cursor_position.y <= ui_node_position.y + ui_node_size.y
                {
                    // If cursor overlaps with UI node, disable PanCam
                    for mut cam in pancam_q.iter_mut() {
                        cam.enabled = false;
                        // println!("disabled: {},{},{},{}",ui_node_position.x,ui_node_position.x+ui_node_size.x,ui_node_position.y,ui_node_position.y+ui_node_size.y);
                        // println!("cs_pos: {},{}",cursor_position.x,cursor_position.y);
                    }
                } else {
                    // Re-enable PanCam if cursor is not over UI
                    for mut cam in pancam_q.iter_mut() {
                        cam.enabled = true;
                        // println!("enabled: {},{},{},{}",ui_node_position.x,ui_node_position.x+ui_node_size.x,ui_node_position.y,ui_node_position.y+ui_node_size.y);
                        // println!("cs_pos: {},{}",cursor_position.x,cursor_position.y);
                    }
                }
            }
        }
    }


    if buttons.just_pressed(MouseButton::Left) {

        

        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
        {
            cursor_coords.0 = world_position;
            for tile in tile_q.iter() {
                    //Shift value x by -24, -24 and value 2 (outer value) by +24, +24
                if world_position.x > tile.location.0 - 24. && world_position.x < (tile.location.0 + TILE_WIDTH as f32 * 3.) && 
                    world_position.y > tile.location.1 - 24. && world_position.y < (tile.location.1 + TILE_HEIGHT as f32 * 3.) {
                    
                    //println!("\n->cursor coords: {}/{}", world_position.x, world_position.y);
                    //println!("Tile data: {:#?}", tile); //pretty print
                    if tile.harvest.yields != ResourceItemType::None {
                        println!("Harvestable!");

                    }
                }
            }
        };
    }
}

fn reload_on_r( //Reload map textures on 'r' press
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    tilebasetype_q: Query<Entity, With<TileBaseType>>,
) {
    for entities in &tilebasetype_q {
        commands.entity(entities).despawn(); //despawn all entities
    };
    let texture: Handle<Image> = asset_server.load(SPRITE_SHEET_PATH);

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_WIDTH as u32, TILE_HEIGHT as u32), MAP_COLS, MAP_ROWS, None, None);

    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    generate_new_map(commands, texture, texture_atlas_layout, Some(tilebasetype_q));
    
}

fn update_camera( //Allows for movement-key control for the camera
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
    mut cam_speed_q: Query<&mut CameraSpeed>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut camera) = camera_q.get_single_mut() else { //Make sure there's just 1 camera
        return;
    };
    let Ok(mut cam_speed) = cam_speed_q.get_single_mut() else {return;}; //And camera speed

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