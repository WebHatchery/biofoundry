use super::*;

#[test]
fn work_speed_follows_brownout_tiers() {
    let mut c = Creature::new(1, "goblin", Job::Miner, TilePos::new(0, 0));

    c.satiation = 1.0;
    assert_eq!(c.work_speed(), 1.0);
    c.satiation = 0.5;
    assert_eq!(c.work_speed(), 0.5);
    c.satiation = 0.1;
    assert_eq!(c.work_speed(), 0.25);
    c.satiation = 0.0;
    assert_eq!(c.work_speed(), 0.25);
}

#[test]
fn carried_goods_accounting() {
    let mut c = Creature::new(1, "goblin", Job::Carrier, TilePos::new(0, 0));

    c.add_carried(Good::Mushroom, 2);
    assert_eq!(c.carried(Good::Mushroom), 2);
    assert_eq!(c.carried(Good::Ore), 0);

    assert_eq!(c.take_carried(Good::Ore, 5), 0);
    assert_eq!(c.take_carried(Good::Mushroom, 1), 1);
    assert_eq!(c.take_carried(Good::Mushroom, 5), 1);
    assert!(c.carrying.is_none());
}
