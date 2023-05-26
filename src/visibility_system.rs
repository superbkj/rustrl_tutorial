use rltk::{field_of_view, Point};
use specs::prelude::*;
use super::{Viewshed, Position, Map, Player};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
  type SystemData = ( 
    WriteExpect<'a, Map>, // Mapリソース取得、無ければパニック
    Entities<'a>,
    WriteStorage<'a, Viewshed>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, Player>,
  );

  fn run(&mut self, data : Self::SystemData) {
    let (mut map, entities, mut viewshed, pos, player) = data;

    for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
      if viewshed.dirty {
        viewshed.dirty = false;
        viewshed.visible_tiles.clear();
        // &*map: "dereference (*), then get a reference (&)" to unwrap Map from the ECS
        viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
        // retain: ラムダ式の条件を満たす要素だけを保持する。
        // |p|: 引数。visible_tilesの要素のPointが順番に入るっぽい
        // 視野内のPointがマップ境界を出ないようにしてる
        viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

        // If this is the player, reveal what they can see
        // playerストレージにそのエンティティが存在するか？
        let _p : Option<&Player> = player.get(ent);
        // if there is a Player component
        if let Some(_p) = _p {
          // いったんvisible falseにクリア
          for t in map.visible_tiles.iter_mut() {*t = false};

          for vis in viewshed.visible_tiles.iter() {
            let idx = map.xy_idx(vis.x, vis.y);
            map.revealed_tiles[idx] = true;
            map.visible_tiles[idx] = true;
          }
        }
      }
    }
  }
}