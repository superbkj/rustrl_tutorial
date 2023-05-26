// rltkという名前空間から使う
// Rltk, GameStateという型
use rltk::{Rltk, GameState, RGB, Point};
use specs::prelude::*;

mod components;
pub use components::*;
// mapモジュールを宣言？モジュールとして定義？
// モジュール (外部ファイル化したRustファイル) はルートファイル (ここではmain.rs) からmodキーワードで宣言が必要
// さもないとそのファイルは存在しないものとして扱われる
mod map;
// useキーワードでモジュールの内容を現在のスコープにインポート
// pub useを使うと、そのモジュールはmain.rs以外のモジュール内でも参照できるようになる
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;

// PartialEq allows you to compare the RunState with other RunState variables to determine if they are the same (or different)
#[derive(PartialEq, Copy, Clone)]
pub enum RunState { PreRun, AwaitingInput, PlayerTurn, MonsterTurn }

// 構造体をつくる
// データなりメソッドなりを持たせることができるが、ここではからっぽにして、
// コードをアタッチするための場所として使う
// World: Specsが提供するECS
pub struct State {
    pub ecs: World,
    // pub runstate: RunState,
}

// 上のStateでGameStateというトレイトを実装する
// トレイトはほかの言語で言うインターフェースとか基底クラスみたいなもの
// トレイトを提供するライブラリ (ここではRLTK) が
// 私自身のコードと (その実装がなんであるかを知らないまま) やりとりする、そのためのインターフェースみたいな
// 自身のコード　- トレイト - ライブラリ
impl GameState for State {
    // GameStateのtick関数を実装？なので引数のタイプは決まっている？
    // 返り値の指定がない: 他言語のvoidみたいなもの
    // &mut self: 親構造体にアクセス・変更できるようにする
    // ctx : &mut Rltk: 変数ctxを受け取る。型はRltk
    // &は参照を渡すことを表す。変数へのポインタ。この場合は変数がコピーされない。変更はオリジナルに対して行われる
    fn tick(&mut self, ctx : &mut Rltk) {
        // cls: clear the screen
        ctx.cls();

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }

        // newrunstateをリソースのRunStateに反映
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
        
        draw_map(&self.ecs, ctx);
        
        // 各コンポーネントの保存場所への読み取りアクセス
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        // join: PositionとRenderable両方のコンポーネントを持つエンティティ (だけ) をすべて返す
        // The join method is passing us both, guaranteed to belong to the same enitity
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

// Stateに機能を実装
// LeftWalkerシステムを使えるようにするため、Stateに追加
impl State {
    // self: Stateのインスタンス
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);

        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);

        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);

        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);

        // システムによってなにか変更がなされたら、その変更はすぐ？Worldに適用してください
        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    // simple80x50: 横80文字縦50文字のターミナルを作る
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    // gsという変数にState構造体のコピーをセットする。参照じゃなくて
    let mut gs = State{
        // World::new(): Worldのコンストラクタ。新しくWorldを作る
        ecs: World::new(),
        // runstate: RunState::Running
    };

    // コンポーネントの登録。WorldというECSに登録
    // これで、各々のコンポーネントの保存システムを内部に作ってくれる
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    let map: Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    // 空っぽのエンティティつくって、コンポーネントをくっつける
    // Player作成
    let player_entity = gs.ecs
        .create_entity()
        .with(Position {x: player_x, y: player_y})
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .with(Viewshed{ visible_tiles : Vec::new(), range : 8, dirty: true })
        .with(Name{ name: "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5})
        .build();

    // Monster作成
    let mut rng = rltk::RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph : rltk::FontCharType;
        let name : String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {glyph = rltk::to_cp437('g'); name = "Goblin".to_string();}
            _ => {glyph = rltk::to_cp437('o'); name = "Orc".to_string();}
        }

        gs.ecs.create_entity()
            .with(Position{x, y})
            .with(Renderable{
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK)
            })
            .with(Viewshed{
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true
            })
            .with(Monster{})
            .with(Name{ name: format!("{} #{}", &name, i)})
            .with(BlocksTile{})
            .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4})
            .build();
    }

    // マップを作り、「リソース」にする
    // つまりECS全体の共有データにする
    // ecs.get, ecs.fetch, get_mut などでアクセスできる
    gs.ecs.insert(map);
    
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::PreRun);

    // メインループ: UIの表示やゲームを走らせ続けるなどの複雑なところを受け持つ
    // こいつがtick関数を毎度呼ぶことになる
    rltk::main_loop(context, gs)
}
