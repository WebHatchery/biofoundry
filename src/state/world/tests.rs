use super::*;

#[test]
fn generation_is_deterministic() {
    let mut a = SeededRng::new(99);
    let mut b = SeededRng::new(99);
    let map_a = WorldMap::generate(48, 32, &mut a);
    let map_b = WorldMap::generate(48, 32, &mut b);

    assert_eq!(map_a.spawn, map_b.spawn);
    for (pos, tile) in map_a.tiles.iter_with_pos() {
        assert_eq!(map_b.tiles.get(pos), Some(tile));
    }
}

#[test]
fn map_has_all_starting_resources() {
    let mut rng = SeededRng::new(20260710);
    let map = WorldMap::generate(48, 32, &mut rng);

    let mut floors = 0;
    let mut water = 0;
    let mut mushrooms = 0;
    let mut ore = 0;
    let mut sporewood = 0;
    for (_, tile) in map.tiles.iter_with_pos() {
        match tile {
            Tile::Floor => floors += 1,
            Tile::Water => water += 1,
            Tile::MushroomPatch => mushrooms += 1,
            Tile::OreVein => ore += 1,
            Tile::Sporewood => sporewood += 1,
            Tile::Rock => {}
        }
    }

    assert!(
        floors > 100,
        "expected a carved warren, got {floors} floors"
    );
    assert!(water > 0, "expected at least one water pool");
    assert!(mushrooms > 0, "expected mushroom patches");
    assert!(ore > 0, "expected reachable ore veins");
    assert!(sporewood > 0, "expected sporewood groves");
}

#[test]
fn spawn_is_walkable_and_edges_are_sealed() {
    let mut rng = SeededRng::new(7);
    let map = WorldMap::generate(48, 32, &mut rng);

    assert!(map.tiles.get(map.spawn).unwrap().walkable());
    for x in 0..48 {
        assert_eq!(map.tiles.get(TilePos::new(x, 0)), Some(&Tile::Rock));
        assert_eq!(map.tiles.get(TilePos::new(x, 31)), Some(&Tile::Rock));
    }
    for y in 0..32 {
        assert_eq!(map.tiles.get(TilePos::new(0, y)), Some(&Tile::Rock));
        assert_eq!(map.tiles.get(TilePos::new(47, y)), Some(&Tile::Rock));
    }
}
