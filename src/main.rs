// rltkという名前空間から使う
// Rltk, GameStateという型
use rltk::{Rltk, GameState, RGB};
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

// 構造体をつくる
// データなりメソッドなりを持たせることができるが、ここではからっぽにして、
// コードをアタッチするための場所として使う
// World: Specsが提供するECS
pub struct State {
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

// Stateに機能を実装
// LeftWalkerシステムを使えるようにするため、Stateに追加
impl State {
    // self: Stateのインスタンス
    fn run_systems(&mut self) {
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
        ecs: World::new()
    };

    // コンポーネントの登録。WorldというECSに登録
    // これで、各々のコンポーネントの保存システムを内部に作ってくれる
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    // gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();

    let (rooms, map) = new_map_rooms_and_corridors();
    
    // マップを作り、「リソース」にする
    // つまりECS全体の共有データにする
    // ecs.get, ecs.fetch, get_mut などでアクセスできる
    gs.ecs.insert(map);
    let (player_x, player_y) = rooms[0].center();

    // 空っぽのエンティティつくって、コンポーネントをくっつける
    gs.ecs
        .create_entity()
        .with(Position {x: player_x, y: player_y})
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .build();

    // メインループ: UIの表示やゲームを走らせ続けるなどの複雑なところを受け持つ
    // こいつがtick関数を毎度呼ぶことになる
    rltk::main_loop(context, gs)
}
