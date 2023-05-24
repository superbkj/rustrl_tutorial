// rltkという名前空間から使う
// Rltk, GameStateという型
use rltk::{Rltk, GameState, RGB, VirtualKeyCode, Tile};
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

        // fetchでリソースを取得
        // fetchで返されるのはReferenceでなくshred型らしい
        // Referenceとして振る舞うものだがちょっと違うらしい
        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);
        
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
struct Player {}

// Stateに機能を実装
// LeftWalkerシステムを使えるようにするため、Stateに追加
impl State {
    // self: Stateのインスタンス
    fn run_systems(&mut self) {
        // システムによってなにか変更がなされたら、その変更はすぐ？Worldに適用してください
        self.ecs.maintain();
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
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

// PartialEq: == によって型がマッチしているか調べられるようになる。tile_type == TileType::Wallのような感じで
// Copy: tile1 = tile2 としたときに、同じものを参照するのではなく？コピーが作られる動きになる。
// Clone: .clone()メソッドが追加される。
#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall, Floor
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    // セミコロンなしの行はReturn式とみなされる
    // usizeはプラットフォーム依存の整数型であり、実行環境のポインタサイズと同じビット幅を持ちます。つまり、32ビットの環境では32ビットの符号なし整数型であり、64ビットの環境では64ビットの符号なし整数型となります。
    (y as usize * 80) + x as usize
}

fn new_map() -> Vec<TileType> {
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

// 上のnew_mapの返り値の型である &Vec<TileType> ではなく &[TileType] を渡している
// こうすると、mapのスライスを渡せるようになる (ここではまだそうしてはいないが、後で便利になる)
fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), rltk::to_cp437('.'));
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'));
            }
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
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
    // gs.ecs.register::<LeftMover>();
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

    // マップを作り、「リソース」にする
    // つまりECS全体の共有データにする
    // ecs.get, ecs.fetch, get_mut などでアクセスできる
    gs.ecs.insert(new_map());

    // メインループ: UIの表示やゲームを走らせ続けるなどの複雑なところを受け持つ
    // こいつがtick関数を毎度呼ぶことになる
    rltk::main_loop(context, gs)
}
