pub type Rank = i32;
pub type Deck = [u32; 10];
pub type HandResult = f32;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Action {
    Stand,
    Hit,
    Double,
    Split,
}
