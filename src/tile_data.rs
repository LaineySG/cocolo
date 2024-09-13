use std::vec;

use bevy::prelude::Component;
use rand::Rng;
use rand::distributions::{Distribution, Standard};
use rand::seq::{IteratorRandom, SliceRandom};

#[derive(Component, Clone, Copy, Debug)]
pub struct Tile {
    pub location :(f32,f32), 
    pub base_type: TileBaseType, 
    pub outpost: OutpostTile,
    pub harvest: HarvestableTile,
    pub enemy: EnemyTile,
}

#[derive(Component, Hash, PartialEq, Clone, Copy, Debug)]
pub enum OutpostType { None, Outpost, ForestOutpost, MountainOutpost, ShipOutpost}
impl Eq for OutpostType {} //partialEq is implemented
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub struct OutpostTile {
    pub outpost_type: OutpostType,
    pub sales_mod: i32,
}
impl Eq for OutpostTile {} //partialEq is implemented
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub struct HarvestableTile {
    pub yields: ResourceItemType,
    pub amount_mod: i32,
}
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
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum EnemyTileType { None, PirateShip, BeepleBillage}
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum ResourceItemType { None, Nut(NutType),Bush(BushType),Fruit(FruitType),Mine(MineType),Spice(SpiceType)}
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum NutType {None, Walnut, Chestnut, Hazelnut, Macademia, Pistachio, Cocoa, Almond, Peanut}
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum BushType{None, Strawberry, Blueberry, Blackberry, Grapes, Raspberry, Coffee, Marshmallow, Rhubarb}
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum FruitType {None, Banana, Coconut, Cherry, Apple, Peach, Plum, Apricot, Pear, Elderberry, Goji, Lemon, Lime, MapleSyrup, Orange, Pineapple}
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum MineType {None, Sugar, JellyCrystals, Salt, Milkgem, Maltgem}
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum SpiceType {None, Cinammon, Ginger, EssenceOfBear, EssenceOfWorm, Peppermint, Icemint, Seamint, Spearmint, GumgumBush, Rose, LicoriceRoot, Violet}

#[derive(Component, Hash, PartialEq, Clone, Copy, Debug)]
pub enum TileBaseType {
    Plains, Forest, DarkForest, Jungle, Mountain, DarkMountain, Sand, Dirt, DarkDirt, Mud, Water, DeepWater, River, Pond, All}
impl Eq for TileBaseType {
    //partialEq is implemented
}
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
impl SpiceType {
    pub fn get_rand(base: TileBaseType) -> SpiceType {
        let options: Vec<i32>;
        if base == TileBaseType::Water || base == TileBaseType::DeepWater || base == TileBaseType::Pond || base == TileBaseType::River {options = vec![6]} 
        else if base == TileBaseType::Jungle {options = vec![0,1,8]} 
        else if base == TileBaseType::DarkForest {options = vec![0,2,10]}
        else if base == TileBaseType::Forest {options = vec![0,1,4]}
        else if base == TileBaseType::Mountain || base == TileBaseType::DarkMountain {options = vec![2,7,9,11,5]} //bear, ice, spear, violet, rose
        else if base == TileBaseType::Dirt || base == TileBaseType::DarkDirt {options = vec![3]}
        else if base == TileBaseType::All {options = vec![0,1,2,3,4,5,6,7,8,9,10,11]}
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
            _  => SpiceType::Violet,
        }
    }
}