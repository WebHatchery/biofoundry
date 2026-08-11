use super::*;

#[test]
fn stock_accounting() {
    let mut b = Building::new("kiln", TilePos::new(1, 1));
    assert_eq!(b.stock(Good::Wood), 0.0);

    b.add_stock(Good::Wood, 3.0);
    assert_eq!(b.take_stock(Good::Wood, 1.5), 1.5);
    assert_eq!(b.take_stock(Good::Wood, 5.0), 1.5);
    assert_eq!(b.take_stock(Good::Wood, 1.0), 0.0);
}
