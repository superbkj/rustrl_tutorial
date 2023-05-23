// rltkという名前空間から使う
// Rltk, GameStateという型
use rltk::{Rltk, GameState, RGB, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};
use specs_derive::Component;

// 構造体をつくる
// データなりメソッドなりを持たせることができるが、ここではからっぽにして、
// コードをアタッチするための場所として使う
// World: Specsが提供するECS
struct State {
    ecs: World
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

        // 登録したすべてのシステムが走る
        self.run_systems();

        player_input(self, ctx);
        
        // 各コンポーネントの保存場所への読み取りアクセス
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        // join: PositionとRenderable両方のコンポーネントを持つエンティティ (だけ) をすべて返す
        // The join method is passing us both, guaranteed to belong to the same enitity
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

// PositionがコンポーネントであるとSpecsに伝える
// #[derive(X)]: Xに必要なお決まりのコードを代わりに書いてくれる
// 場所
#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

// 見た目
#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component)]
struct LeftMover {}

#[derive(Component)]
struct Player {}

struct LeftWalker {}
// 'a: lifetime specifier. とりあえずあんまり気にしない
impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    // Systemトレイトのrun関数の実装
    fn run(&mut self, (lefty, mut pos) : Self::SystemData) {
        // LeftMoverとPositionの両方を持つエンティティだけを取得
        // _left: アンダースコアはその変数を使用しないことを示す。今回はそれを持ってさえいればいいので
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 { pos.x = 79 }
        }
    }
}

// Stateに機能を実装
// LeftWalkerシステムを使えるようにするため、Stateに追加
impl State {
    // self: Stateのインスタンス
    fn run_systems(&mut self) {
        // LeftWalkerシステムのインスタンス作成
        let mut lw = LeftWalker{};
        // LeftWalkerシステムを走らせる
        lw.run_now(&self.ecs);
        // システムによってなにか変更がなされたら、その変更はすぐ？Worldに適用してください
        self.ecs.maintain();
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {} // 何も起こらない
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {} // anything else
        },
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
        ecs: World::new()
    };

    // コンポーネントの登録。WorldというECSに登録
    // これで、各々のコンポーネントの保存システムを内部に作ってくれる
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();

    // 空っぽのエンティティつくって、コンポーネントをくっつける
    gs.ecs
        .create_entity()
        .with(Position {x: 40, y: 25})
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .build();

    for i in 0..10 {
        gs.ecs
        .create_entity()
        .with(Position {x: i * 7, y: 20})
        .with(Renderable {
            glyph: rltk::to_cp437('☺'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(LeftMover{})
        .build();
    }

    // メインループ: UIの表示やゲームを走らせ続けるなどの複雑なところを受け持つ
    // こいつがtick関数を毎度呼ぶことになる
    rltk::main_loop(context, gs)
}
