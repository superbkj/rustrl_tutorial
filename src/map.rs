use rltk::{RGB, Rltk, RandomNumberGenerator, Algorithm2D, BaseMap, Point};
use crate::Viewshed;

use super::{Rect};
use std::cmp::{max, min};
use specs::prelude::*;

// PartialEq: == によって型がマッチしているか調べられるようになる。tile_type == TileType::Wallのような感じで
// Copy: tile1 = tile2 としたときに、同じものを参照するのではなく？コピーが作られる動きになる。
// Clone: .clone()メソッドが追加される。
#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub struct Map {
  pub tiles : Vec<TileType>,
  pub rooms : Vec<Rect>,
  pub width : i32,
  pub height : i32,
  pub revealed_tiles : Vec<bool>,
  pub visible_tiles : Vec<bool>,
  pub blocked: Vec<bool>,
  pub tile_content : Vec<Vec<Entity>>
}

impl Map {
  pub fn xy_idx(&self, x: i32, y: i32) -> usize {
    // セミコロンなしの行はReturn式とみなされる
    // usizeはプラットフォーム依存の整数型であり、実行環境のポインタサイズと同じビット幅を持ちます。つまり、32ビットの環境では32ビットの符号なし整数型であり、64ビットの環境では64ビットの符号なし整数型となります。
    (y as usize * self.width as usize) + x as usize
  }

  fn apply_room_to_map(&mut self, room: &Rect) {
    for y in room.y1 + 1 ..= room.y2 {
      for x in room.x1 + 1 ..= room.x2 {
        let idx = self.xy_idx(x, y);
        self.tiles[idx] = TileType::Floor;
      }
    }
  }
  
  fn apply_horizontal_tunnel(&mut self, x1:i32, x2:i32, y:i32) {
    for x in min(x1, x2) ..= max(x1, x2) {
      let idx = self.xy_idx(x, y);
      if idx > 0 && idx < self.width as usize * self.height as usize {
        self.tiles[idx as usize] = TileType::Floor;
      }
    }
  }
  
  fn apply_vertical_tunnel(&mut self, y1:i32, y2:i32, x:i32) {
    for y in min(y1, y2) ..= max(y1, y2) {
      let idx = self.xy_idx(x, y);
      if idx > 0 && idx < self.width as usize * self.height as usize {
        self.tiles[idx as usize] = TileType::Floor;
      }
    }
  }

  pub fn new_map_rooms_and_corridors() -> Map {
    let mut map = Map {
      tiles: vec![TileType::Wall; 80*50],
      rooms: Vec::new(),
      width: 80,
      height: 50,
      revealed_tiles: vec![false; 80*50],
      visible_tiles: vec![false; 80*50],
      blocked: vec![false; 80*50],
      tile_content: vec![Vec::new(); 80*50]
    };
    
    // const: Can never change
    const MAX_ROOMS : i32 = 30;
    const MIN_SIZE : i32 = 6;
    const MAX_SIZE : i32 = 10;
  
    let mut rng = RandomNumberGenerator::new();
    
    for _ in 0..MAX_ROOMS {
      let w = rng.range(MIN_SIZE, MAX_SIZE);
      let h = rng.range(MIN_SIZE, MAX_SIZE);
      let x = rng.roll_dice(1, 80 - w - 1) - 1;
      let y = rng.roll_dice(1, 50 - h - 1) - 1;
      let new_room = Rect::new(x, y, w, h);
      let mut ok = true;
      for other_room in map.rooms.iter() {
        if new_room.intersect(other_room) { ok = false }
      }
      if ok {
        map.apply_room_to_map(&new_room);
  
        if !map.rooms.is_empty() {
          let (new_x, new_y) = new_room.center();
          let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
          if rng.range(0, 2) == 1 {
            map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
            map.apply_vertical_tunnel(prev_y, new_y, new_x);
          } else {
            map.apply_vertical_tunnel(prev_y, new_y, prev_x);
            map.apply_horizontal_tunnel(prev_x, new_x, new_y);
          }
        }
  
        map.rooms.push(new_room);
      }
    }
  
    map
  }

  /// そのExitの座標が壁などブロックタイルでなければ、通れるものとしてTrue
  fn is_exit_valid(&self, x:i32, y:i32) -> bool {
    // 境界は常に壁だが、それでもこの行はやる意味ある。
    // 下のxy_idxでメモリ外参照をしないようにここでとどめる
    if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 {return false;}
    
    let idx = self.xy_idx(x, y);
    !self.blocked[idx]
  }

  /// マップの各タイルが壁などのブロックタイルか否かをセットする
  pub fn populate_blocked(&mut self) {
    for (i, tile) in self.tiles.iter_mut().enumerate() {
      self.blocked[i] = *tile == TileType::Wall;
    }
  }

  pub fn clear_content_index(&mut self) {
    for content in self.tile_content.iter_mut() {
      content.clear();
    }
  }
}


/*
/// Makes a map with solid boundaries and 400 randomly placed walls. No guarantees that it won't
/// look awful.
pub fn new_map_test() -> Vec<TileType> {
    // vec!マクロ: [型, 要素数]
    let mut map = vec![TileType::Floor; 80*50];

    // Make the boundaries walls
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }

    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    // ランダムに壁生成
    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..400 {
        // random number 1 ~ 79
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}
*/

// 上のnew_mapの返り値の型である &Vec<TileType> ではなく &[TileType] を渡している
// こうすると、mapのスライスを渡せるようになる (ここではまだそうしてはいないが、後で便利になる)
pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
  let map = ecs.fetch::<Map>();

  let mut y = 0;
  let mut x = 0;
  for (idx, tile) in map.tiles.iter().enumerate() {
    // Render a tile depending upon the tile type
    if map.revealed_tiles[idx] {
      let glyph;
      let mut fg;
      match tile {
          TileType::Floor => {
              glyph = rltk::to_cp437('.');
              fg = RGB::from_f32(0.0, 0.5, 0.5);
          }
          TileType::Wall => {
              glyph = rltk::to_cp437('#');
              fg = RGB::from_f32(0., 1.0, 0.);
          }
      }
      if !map.visible_tiles[idx] { fg = fg.to_greyscale() }
      ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
    }

    // Move the coordinates
    x += 1;
    if x > 79 {
        x = 0;
        y += 1;
      }
  }
}

impl Algorithm2D for Map {
  fn dimensions(&self) -> rltk::Point {
      Point::new(self.width, self.height)
  }
}

impl BaseMap for Map {
  // opaque: 不透明
  fn is_opaque(&self, idx:usize) -> bool {
    self.tiles[idx as usize] == TileType::Wall
  }

  /// 与えられた座標の上下左右斜め方向を見て、それぞれ通れるならExitとして追加
  fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
      let mut exits = rltk::SmallVec::new();
      let x = idx as i32 % self.width;
      let y = idx as i32 / self.width;
      let w = self.width as usize;

      // 上下左右
      if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
      if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
      if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
      if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };

      // ななめ
      if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45)) };
      if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, 1.45)) };
      if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)-1, 1.45)) };
      if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45)) };

      exits
  }

  // 2点間の距離
  fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
      let w = self.width as usize;
      let p1 = Point::new(idx1 % w, idx1 / w);
      let p2 = Point::new(idx2 % w, idx2 / w);
      rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
  }
}