use specs::prelude::*;
use specs_derive::*;
use rltk::{RGB};

// PositionがコンポーネントであるとSpecsに伝える
// #[derive(X)]: Xに必要なお決まりのコードを代わりに書いてくれる
// 場所
#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// 見た目
#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Viewshed {
  pub visible_tiles : Vec<rltk::Point>,
  pub range : i32,
  pub dirty : bool
}

#[derive(Component, Debug)]
pub struct Monster {}

#[derive(Component, Debug)]
pub struct Name {
  pub name: String
}