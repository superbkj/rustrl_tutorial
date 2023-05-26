use rltk::{console};
use specs::prelude::*;
use super::{CombatStats, WantsToMelee, Name, SufferDamage};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
  type SystemData = (
    Entities<'a>,
    WriteStorage<'a, WantsToMelee>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, CombatStats>,
    WriteStorage<'a, SufferDamage>,
  );

  fn run(&mut self, data : Self::SystemData) {
    // inflict: 与える、課す
    let (entities, mut wants_to_melee, names, combat_stats, mut inflict_damage) = data;

    for (_entity, wants_to_melee, name, stats) in (&entities, &wants_to_melee, &names, &combat_stats).join() {
      if stats.hp > 0 {
        let target_stats = combat_stats.get(wants_to_melee.target).unwrap();
        if target_stats.hp > 0 {
          let target_name = names.get(wants_to_melee.target).unwrap();

          let damage = i32::max(0, stats.power - target_stats.defense);

          if damage == 0 {
            console::log(&format!("{} is unable to hurt {}", &name.name, &target_name.name));
          } else {
            console::log(&format!("{} hits {}, for {} hp", &name.name, &target_name.name, damage));
            
            // storage, victim, amount
            SufferDamage::new_damage(&mut inflict_damage, wants_to_melee.target, damage);
          }
        }
      }
    }

    wants_to_melee.clear();
  }
}