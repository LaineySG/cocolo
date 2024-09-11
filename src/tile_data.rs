use bevy::prelude::Component;
use rand::Rng;
use rand::distributions::{Distribution, Standard};

#[derive(Component, Clone, Copy,)]
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
pub enum NutType {Walnut, Chestnut, Hazelnut, Macademia, Pistachio, Cocoa, Almond, Peanut}
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum BushType{Strawberry, Blueberry, Blackberry, Grapes, Raspberry, Coffee, Marshmallow, Rhubarb}
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum FruitType {Banana, Coconut, Cherry, Apple, Peach, Plum, Apricot, Pear, Elderberry, Goji, Lemon, Lime, MapleSyrup, Orange, Pineapple}
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum MineType {Sugar, JellyCrystals, Salt, Milkgem, Maltgem}
#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub enum SpiceType {Cinammon, Ginger, EssenceOfBear, EssenceOfWorm, Peppermint, Icemint, Seamint, Spearmint, GumgumBush, Rose, LicoriceRoot, Violet}

#[derive(Component, Hash, PartialEq, Clone, Copy, Debug)]
pub enum TileBaseType {
    Plains, Forest, DarkForest, Jungle, Mountain, DarkMountain, Sand, Dirt, DarkDirt, Mud, Water, DeepWater, River, Pond}
impl Eq for TileBaseType {
    //partialEq is implemented
}

impl Distribution<ResourceItemType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ResourceItemType {
        match rng.gen_range(0..=4) {
            0 => ResourceItemType::Nut(NutType::Almond),
            1 => ResourceItemType::Bush(BushType::Blackberry),
            2 => ResourceItemType::Fruit(FruitType::Apple),
            3 => ResourceItemType::Mine(MineType::JellyCrystals),
            _ => ResourceItemType::Spice(SpiceType::Cinammon),
        }
    }
}
impl Distribution<NutType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> NutType {
        match rng.gen_range(0..=7) {
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
impl Distribution<BushType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BushType {
        match rng.gen_range(0..=7) {
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
impl Distribution<FruitType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FruitType {
        match rng.gen_range(0..=15) {
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
impl Distribution<MineType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MineType {
        match rng.gen_range(0..=4) {
            0 => MineType::Sugar,
            1 => MineType::JellyCrystals,
            2 => MineType::Salt,
            3 => MineType::Milkgem,
            _ => MineType::Maltgem,
        }
    }
}
impl Distribution<SpiceType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SpiceType {
        match rng.gen_range(0..=11) {
            0  => SpiceType::Cinammon,
            1  => SpiceType::Ginger,
            2  => SpiceType::EssenceOfBear,
            3  => SpiceType::EssenceOfWorm,
            4  => SpiceType::Peppermint,
            5  => SpiceType::Icemint,
            6  => SpiceType::Seamint,
            7  => SpiceType::Spearmint,
            8  => SpiceType::GumgumBush,
            9  => SpiceType::Rose,
            10 => SpiceType::LicoriceRoot,
            _  => SpiceType::Violet,
        }
    }
}
