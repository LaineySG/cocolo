use std::vec;

use bevy::prelude::Component;
use rand::Rng;
use rand::distributions::{Distribution, Standard};
use rand::seq::{IteratorRandom, SliceRandom};





///Stores tile data including location, resources, base type, and workers.
#[derive(Component, Clone, Debug)]
pub struct Tile {
    pub location :(f32,f32), 
    pub base_type: TileBaseType, 
    pub loonkas: Vec<Loonka>,
    pub outpost: OutpostTile,
    pub harvest: HarvestableTile,
    pub enemy: EnemyTile,
}

///Stores type of the outpost.
#[derive(Component, Hash, PartialEq, Clone, Copy, Debug)]
pub enum OutpostType { None, Outpost, ForestOutpost, MountainOutpost, ShipOutpost}
impl Eq for OutpostType {} //partialEq is implemented

impl OutpostType {
    pub fn get_rand(base: TileBaseType) -> OutpostType {
        let options: Vec<i32>;
        if base == TileBaseType::Mountain || base == TileBaseType::DarkMountain {options = vec![2]} 
        else if base == TileBaseType::Water || base == TileBaseType::DeepWater || base == TileBaseType::Pond || base == TileBaseType::River {options = vec![1]} //if mountain, only mines or spices
        else if base == TileBaseType::Forest || base == TileBaseType::DarkForest || base == TileBaseType::Jungle {options = vec![0]}
        else {options = vec![3]}
        let random = options.iter().choose(&mut rand::thread_rng()).unwrap();
        match random {
            0 => OutpostType::ForestOutpost,
            1 => OutpostType::ShipOutpost,
            2 => OutpostType::MountainOutpost,
            _ => OutpostType::Outpost,
        }
    }
}

///Stores data about the outpost tile including type and trading modifier.
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub struct OutpostTile {
    pub outpost_type: OutpostType,
    pub sales_mod: i32,
}
impl Eq for OutpostTile {} //partialEq is implemented

///Stores type of yield and amount of a harvestable tile.
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub struct HarvestableTile {
    pub yields: ResourceItemType,
    pub amount_mod: i32,
}

///Stores data about a loonka worker.
#[derive(PartialEq, Hash, Clone, Debug, Component)]
pub struct Loonka{
    pub name: String,
    pub id: usize,
    pub current_job: LoonkaJob,
    pub vigor: i32, //warrior, traveller, harvester, factory_worker
    pub speed: i32, //(traveller)
    pub dexterity: i32, //(harvester), (factory_worker), clown
    pub strength: i32, //(warrior)
    pub charisma: i32, //(trader), (clown)
    pub intellect: i32, //(researcher)
    pub icon_num: i32,
}

///Stores details of a Loonka's job
#[derive(PartialEq, Hash, Clone, Copy, Debug)]#[repr(usize)]
pub enum LoonkaJob {None, Harvester,Researcher,FactoryWorker,Traveller,Trader,Warrior,Clown}
impl LoonkaJob {
    pub fn from_index(index: usize) -> Option<LoonkaJob> {
        match index {
            0 => Some(LoonkaJob::None),
            1 => Some(LoonkaJob::Harvester),
            2 => Some(LoonkaJob::Researcher),
            3 => Some(LoonkaJob::FactoryWorker),
            4 => Some(LoonkaJob::Traveller),
            5 => Some(LoonkaJob::Trader),
            6 => Some(LoonkaJob::Warrior),
            7 => Some(LoonkaJob::Clown),
            _ => None,
        }
    }
}

///Stores details of the enemy on a tile such as enemy type, drops, health, and damage.
impl Eq for HarvestableTile {} //partialEq is implemented
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub struct EnemyTile {
    pub enemy_type: EnemyTileType,
    pub yields: [ResourceItemType;5], //Up to 5 drops
    pub amount_mod: i32,
    pub health: i32,
    pub damage: i32,
}
impl Eq for EnemyTile {} //partialEq is implemented

///Stores type of the enemy tile
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum EnemyTileType { None, PirateShip, BeepleBillage}

impl EnemyTileType {
    pub fn get_rand(base: TileBaseType) -> EnemyTileType {
        let options: Vec<i32>;
        if base == TileBaseType::Water || base == TileBaseType::DeepWater || base == TileBaseType::Pond || base == TileBaseType::River {options = vec![2]} //if mountain, only mines or spices
        else if base == TileBaseType::Forest || base == TileBaseType::Plains {options = vec![0]}
        else {options = vec![1]}
        let random = options.iter().choose(&mut rand::thread_rng()).unwrap();
        match random {
            0 => EnemyTileType::BeepleBillage,
            1 => EnemyTileType::None,
            _ => EnemyTileType::PirateShip,
        }
    }
}

///Stores type of the resource tile, and yield of that tile.
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum ResourceItemType { None, Nut(NutType),Bush(BushType),Fruit(FruitType),Mine(MineType),Spice(SpiceType)}

impl ResourceItemType {
    pub fn get_rand(base: TileBaseType) -> ResourceItemType {
        let options: Vec<i32>;
        if base == TileBaseType::Mountain || base == TileBaseType::DarkMountain {options = vec![3,4]}
        else if base == TileBaseType::Water || base == TileBaseType::DeepWater || base == TileBaseType::Pond || base == TileBaseType::River {options = vec![4]} //only spice if water for seamint
        else {options = vec![0,1,2,4]}; //if mountain, only mines or spices
        let random = options.iter().choose(&mut rand::thread_rng()).unwrap();
        match random {
            0 => ResourceItemType::Nut(NutType::get_rand(base)),
            1 => ResourceItemType::Bush(BushType::get_rand(base)),
            2 => ResourceItemType::Fruit(FruitType::get_rand(base)),
            3 => ResourceItemType::Mine(MineType::get_rand(base)),
            _ => ResourceItemType::Spice(SpiceType::get_rand(base)),
        }
    }
}

///Stores type of nut.
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum NutType {None, Walnut, Chestnut, Hazelnut, Macademia, Pistachio, Cocoa, Almond, Peanut}

impl NutType {
    pub fn get_rand(base: TileBaseType) -> NutType {
        let options: Vec<i32>;
        if base == TileBaseType::DarkForest {options = vec![0,4]} 
        else if base == TileBaseType::Jungle {options = vec![3,5,7]} 
        else if base == TileBaseType::DarkForest {options = vec![0,1,2]}
        else if base == TileBaseType::Forest {options = vec![0,6]}
        else if base == TileBaseType::All {options = vec![0,1,2,3,4,5,6,7]}
        else {options = vec![6]}
        let random = options.iter().choose(&mut rand::thread_rng()).unwrap();
        match random {
            0 => NutType::Walnut,
            1 => NutType::Chestnut,
            2 => NutType::Hazelnut,
            3 => NutType::Macademia,
            4 => NutType::Pistachio,
            5 => NutType::Cocoa,
            6 => NutType::Almond,
            _ => NutType::Peanut,
        }
    }
}

///Stores type of bush.
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum BushType{None, Strawberry, Blueberry, Blackberry, Grapes, Raspberry, Coffee, Marshmallow, Rhubarb}

impl BushType {
    pub fn get_rand(base: TileBaseType) -> BushType {
        let options: Vec<i32>;
        if base == TileBaseType::DarkForest {options = vec![2,4]} 
        else if base == TileBaseType::Jungle {options = vec![5]} 
        else if base == TileBaseType::DarkForest {options = vec![5,7]}
        else if base == TileBaseType::Forest {options = vec![0,1,2,4]}
        else if base == TileBaseType::All {options = vec![0,1,2,3,4,5,6,7]}
        else {options = vec![3,6]}
        let random = options.iter().choose(&mut rand::thread_rng()).unwrap();
        match random {
            0 => BushType::Strawberry,
            1 => BushType::Blueberry,
            2 => BushType::Blackberry,
            3 => BushType::Grapes,
            4 => BushType::Raspberry,
            5 => BushType::Coffee,
            6 => BushType::Marshmallow,
            _ => BushType::Rhubarb,
        }
    }
}

///Stores type of fruit.
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum FruitType {None, Banana, Coconut, Cherry, Apple, Peach, Plum, Apricot, Pear, Elderberry, Goji, Lemon, Lime, MapleSyrup, Orange, Pineapple}

impl FruitType {
    pub fn get_rand(base: TileBaseType) -> FruitType {
        let options: Vec<i32>;
        if base == TileBaseType::Jungle {options = vec![0,1,9,14]} 
        else if base == TileBaseType::DarkForest {options = vec![2,5,8,12]}
        else if base == TileBaseType::Forest {options = vec![3,4,6,7,10,11,13]}
        else if base == TileBaseType::Mud || base == TileBaseType::Dirt || base == TileBaseType::DarkDirt {options = vec![5,8]}
        else if base == TileBaseType::All {options = vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14]}
        else {options = vec![3,13]}
        let random = options.iter().choose(&mut rand::thread_rng()).unwrap();
        match random {
            0  => FruitType::Banana,
            1  => FruitType::Coconut,
            2  => FruitType::Cherry,
            3  => FruitType::Apple,
            4  => FruitType::Peach,
            5  => FruitType::Plum,
            6  => FruitType::Apricot,
            7  => FruitType::Pear,
            8  => FruitType::Elderberry,
            9  => FruitType::Goji,
            10 => FruitType::Lemon,
            11 => FruitType::Lime,
            12 => FruitType::MapleSyrup,
            13 => FruitType::Orange,
            _ => FruitType::Pineapple,
        }
    }
}

///Stores type of mine.
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum MineType {None, Sugar, JellyCrystals, Salt, Milkgem, Maltgem}

impl MineType {
    pub fn get_rand(base: TileBaseType) -> MineType {
        let options: Vec<i32>;
        if base == TileBaseType::Mountain {options = vec![0,2,3]} 
        else if base == TileBaseType::DarkMountain {options = vec![1,4]} 
        else if base == TileBaseType::All {options = vec![0,1,2,3,4]}
        else {options = vec![3,6]}
        let random = options.iter().choose(&mut rand::thread_rng()).unwrap();
        match random {
            0 => MineType::Sugar,
            1 => MineType::JellyCrystals,
            2 => MineType::Salt,
            3 => MineType::Milkgem,
            _ => MineType::Maltgem,
        }
    }
}

///Stores type of spice.
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum SpiceType {None, Cinammon, Ginger, EssenceOfBear, EssenceOfWorm, Peppermint, Icemint, Seamint, Spearmint, GumgumBush, Rose, LicoriceRoot, Violet, Vanilla}
impl SpiceType {
    pub fn get_rand(base: TileBaseType) -> SpiceType {
        let options: Vec<i32>;
        if base == TileBaseType::Water || base == TileBaseType::DeepWater || base == TileBaseType::Pond || base == TileBaseType::River {options = vec![6]} 
        else if base == TileBaseType::Jungle {options = vec![0,1,8,12]} 
        else if base == TileBaseType::DarkForest {options = vec![0,2,10,12]}
        else if base == TileBaseType::Forest {options = vec![0,1,4]}
        else if base == TileBaseType::Mountain || base == TileBaseType::DarkMountain {options = vec![2,7,9,11,5]} //bear, ice, spear, violet, rose
        else if base == TileBaseType::Dirt || base == TileBaseType::DarkDirt {options = vec![3]}
        else if base == TileBaseType::All {options = vec![0,1,2,3,4,5,6,7,8,9,10,11,12]}
        else {options = vec![4,5,9,11]}
        let random = options.iter().choose(&mut rand::thread_rng()).unwrap();
        match random {
            0  => SpiceType::Cinammon,
            1  => SpiceType::Ginger,
            2  => SpiceType::EssenceOfBear,
            3  => SpiceType::EssenceOfWorm,
            4  => SpiceType::Peppermint,
            5  => SpiceType::Icemint,
            6  => SpiceType::Seamint, //done
            7  => SpiceType::Spearmint,
            8  => SpiceType::GumgumBush,
            9  => SpiceType::Rose,
            10 => SpiceType::LicoriceRoot,
            11  => SpiceType::Violet,
            _  => SpiceType::Vanilla,
        }
    }
}

///Stores tile base type
#[derive(Component, Hash, PartialEq, Clone, Copy, Debug)]
pub enum TileBaseType {
    Plains, Forest, DarkForest, Jungle, Mountain, DarkMountain, Sand, Dirt, DarkDirt, Mud, Water, DeepWater, River, Pond, All}
impl Eq for TileBaseType {
    //partialEq is implemented
}


