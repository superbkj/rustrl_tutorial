use specs::prelude::*;
use crate::WantsToMelee;

use super::{Viewshed, Monster, Name, Map, Position, RunState};
use rltk::{Point, console};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    WriteExpect<'a, Map>,
    ReadExpect<'a, Point>,
    ReadExpect<'a, Entity>,
    ReadExpect<'a, RunState>,
    Entities<'a>,
    WriteStorage<'a, Viewshed>,
    ReadStorage<'a, Monster>,
    // ReadStorage<'a, Name>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, WantsToMelee>
  );

  fn run(&mut self, data : Self::SystemData) {
    let (mut map, player_pos, player_entity,  runstate, entities, mut viewshed, monster, mut position, mut wants_to_melee) = data;

    if *runstate != RunState::MonsterTurn {return;}
    
    for (entity, mut viewshed, _monster, mut pos) in (&entities, &mut viewshed, &monster, &mut position).join() {
      if viewshed.visible_tiles.contains(&*player_pos) {
        let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
        // プレイヤーに隣接したら攻撃。Returnでこれ以上近づかない
        if distance < 1.5 {
          wants_to_melee.insert(entity, WantsToMelee { target: *player_entity }).expect("Unable to insert attack");
        }
        // 隣接してないなら移動
        else if viewshed.visible_tiles.contains(&*player_pos) {
          // A star search: 経路探索アルゴリズム。最短経路探索に優れる
          let path = rltk::a_star_search(
            map.xy_idx(pos.x, pos.y) as i32,
            map.xy_idx(player_pos.x, player_pos.y) as i32,
            &mut *map
          );

          if path.success && path.steps.len() > 1 {
            let mut idx = map.xy_idx(pos.x, pos.y);
            map.blocked[idx] = false;

            // steps[0]は今いるところ
            // Playerに向かって1歩進む
            pos.x = path.steps[1] as i32 % map.width;
            pos.y = path.steps[1] as i32 / map.width;
            idx = map.xy_idx(pos.x, pos.y);

            map.blocked[idx] = true;
            viewshed.dirty = true;
          }
        }
      }
    }
  }
}