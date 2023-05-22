// rltkという名前空間からRltk, GameStateという型を使う
use rltk::{Rltk, GameState};

// 構造体をつくる
// データなりメソッドなりを持たせることができるが、ここではからっぽにして、
// コードをアタッチするための場所として使う
struct State {}

// 上のStateがGameStateというトレイトを実装する
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
        // print the string at the position (x, y)
        ctx.print(1, 1, "Hello Rust World")
    }
}
fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    // simple80x50: 横80文字縦50文字のターミナルを作る
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    // gsという変数にState構造体のコピーをセットする。参照じゃなくて
    let gs = State{ };
    // メインループ: UIの表示やゲームを走らせ続けるなどの複雑なところを受け持つ
    // こいつがtick関数を毎度呼ぶことになる
    rltk::main_loop(context, gs)
}
