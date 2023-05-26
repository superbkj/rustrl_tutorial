use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
  type SystemData = (
    WriteExpect<'a, Map>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, BlocksTile>,
    Entities<'a>
  );

  fn run(&mut self, data : Self::SystemData) {
    let (mut map, position, blockers, entities) = data;

    // 壁などのある所をブロックする
    map.populate_blocked();

    map.clear_content_index();

    // tile_contentにエンティティをセットする
    // そのタイルになにがあるか
    for (entity, position) in (&entities, &position).join() {
      let idx = map.xy_idx(position.x, position.y);
      
      // 敵のいるところなど、BlocksTileならその場所をブロックする
      let _p : Option<&BlocksTile> = blockers.get(entity);
      if let Some(_p) = _p {
        map.blocked[idx] = true;
      }

      // Push the entity to the appropriate index slot.
      // It's a copy type, so no need to clone it
      map.tile_content[idx].push(entity);
    }
  }
}