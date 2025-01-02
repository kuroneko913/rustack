use std::{
    collections::HashMap, 
    io::{BufRead, BufReader}, 
};

#[derive(Clone)]
struct NativeOp(fn(&mut Vm));

// Eq, PartialEq, Debug トレイトを実装する
impl Eq for NativeOp {}
impl PartialEq for NativeOp {
    fn eq(&self, _other: &NativeOp) -> bool {
        self.0 as *const fn() == _other.0 as *const fn()
    }
}
impl std::fmt::Debug for NativeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<NativeOp>")
    }
}

macro_rules! impl_op {
    {$name:ident, $op:tt} => {
        fn $name(vm: &mut Vm) {
            let rhs = vm.stack.pop().unwrap().as_num();
            let lhs = vm.stack.pop().unwrap().as_num();
            vm.stack.push(Value::Num((lhs $op rhs) as i32));
        }
    }
}

impl_op!(add, +);
impl_op!(sub, -);
impl_op!(mul, *);
impl_op!(div, /);
impl_op!(lt, <);

// 仮想マシンの構造体を定義
#[derive(Debug, Clone)]
struct Vm {
    stack: Vec<Value>,            // スタックを保持するベクタ
    vars: HashMap<String, Value>, // 変数を保持するハッシュマップ
    blocks: Vec<Vec<Value>>,      // ブロックを保持するベクタ
}

impl Vm {
    fn new() -> Self {
        let functions: [(&str, fn(&mut Vm)); 10] = [
            ("+", add),
            ("-", sub),
            ("*", mul),
            ("/", div),
            ("<", lt),
            ("if", op_if),
            ("def", op_def),
            ("puts", puts),
            ("dup", dup),
            ("exch", exch),
        ];
        Self {
            stack: vec![],
            vars: functions
                .into_iter().map(|(name, fun)| {
                    (name.to_owned(), Value::Native(NativeOp(fun)))
                }).collect(),
            blocks: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Num(i32),
    Op(String),
    Sym(String),
    Block(Vec<Value>),
    Native(NativeOp),
}

impl Value {
    fn as_num(&self) -> i32 {
        match self {
            Self::Num(val) => *val,
            _ => panic!("Value is not a number"),
        }
    }
    fn to_block(self) -> Vec<Value> {
        match self {
            Self::Block(val) => val,
            _ => panic!("Value is not a block"),
        }
    }
    fn as_sym(&self) -> &str {
        if let Self::Sym(sym) = self {
            sym
        } else {
            panic!("Value is not a symbol");
        }
    }
    fn to_string(&self) -> String {
        match self {
            Self::Num(i) => i.to_string(),
            Self::Op(ref s) | Self::Sym(ref s) => s.clone(),
            Self::Block(_) => "<Block>".to_string(),
            Self::Native(_) => "<Native>".to_string(),
        }
    }
}

fn main() {
    if let Some(f) = std::env::args().nth(1).and_then(|f| std::fs::File::open(f).ok()) {
        parse_batch(BufReader::new(f));
    } else {
        parse_interactive();
    }
}

fn parse_batch(source: impl BufRead) -> Vec<Value> {
    let mut vm = Vm::new();
    for line in source.lines().flatten() {
        for word in line.split(" ") {
            let vm_before = vm.clone();
            parse_word(word, &mut vm);
            debug_vm_diff(word, &vm_before, &vm);
        }
    }
    vm.stack
}

fn parse_interactive() {
    let mut vm = Vm::new();
    for line in std::io::stdin().lines().flatten() {
        for word in line.split(" ") {
            parse_word(word, &mut vm);
        }
        println!("stack: {:?}", vm.stack);
    }
}

fn parse_word(word: &str, vm: &mut Vm) {
    if word.is_empty() {
        return;
    }
    if word == "{" {
        // ブロックを保持できるように、blocksに空のベクタを追加する
        vm.blocks.push(vec![]);
        return;
    }
    if word == "}" {
        // ブロックを保持するベクタを取り出し、Blockとしてスタックに積む
        let block = vm.blocks.pop().expect("block stack is empty");
        eval(Value::Block(block), vm);
        return;
    }
    if word == "\u{3000}" {
        return;
    }
    // 値の種類によって、Value のインスタンスを生成しcodeに保持する
    let code = if let Ok(num) = word.parse::<i32>() {
        // 数字の場合は、Num としてスタックに積む
        Value::Num(num)
    } else if word.starts_with("/") {
        Value::Sym(word[1..].to_string()) // /から始まる文字列を変数名とするため、/を取り除いた文字列を保持する
    } else {
        // 数字、{} 以外の場合、演算子として処理する
        Value::Op(word.to_string())
    };
    eval(code, vm);
}

fn eval(code: Value, vm: &mut Vm) {
    println!("--------------------------------");
    println!("eval: {:?}\nStack: {:?} \n", code, vm.stack);
    for (key,value) in vm.vars.iter() {
        if matches!(value, Value::Native(_)) {
            continue;
        }
        println!("{}: {:?}", key, value);
    }
    // ブロック構造の中にある場合、評価せずにブロックにコードを追加する
    if let Some(top_block) = vm.blocks.last_mut() {
        top_block.push(code);
        return;
    }
    // 演算子でない場合はスタックに積む
    if !matches!(code, Value::Op(_)) {
        vm.stack.push(code);
        return;
    }

    // 演算子の場合
    let Value::Op(op) = code else {
        panic!("Expected operator, found {:?}", code);
    };

    // op_defで定義された変数がある場合は、その値を取得する
    if let Some(val) = vm.vars.get(&op).cloned() {
        match val {
            Value::Block(block) => {
                // ブロックの中身を評価
                for code in block {
                    eval(code, vm);
                }
            },
            Value::Native(op) => op.0(vm), // ネイティブ関数の場合は実行
            _ => {
                vm.stack.push(val); // 他の値はスタックに積む
            }
        }
        return;
    }
}

// if演算子を定義する関数
fn op_if(vm: &mut Vm) {
    let false_branch = vm.stack.pop().unwrap().to_block();
    let true_branch = vm.stack.pop().unwrap().to_block();
    let cond = vm.stack.pop().unwrap().to_block();

    // 条件式の評価を行う
    for code in cond {
        eval(code, vm);
    }

    // 条件式の評価結果を取得する
    let cond_result = vm.stack.pop().unwrap().as_num();

    // 条件式の結果によって、true_branch か false_branch を評価する
    if cond_result != 0 {
        for code in true_branch {
            eval(code, vm);
        }
    } else {
        for code in false_branch {
            eval(code, vm);
        }
    }
}

// 変数定義を行う演算子を定義する関数
fn op_def(vm: &mut Vm) {
    let value = vm.stack.pop().unwrap();
    eval(value, vm);
    let value = vm.stack.pop().unwrap();
    let sym = vm.stack.pop().unwrap().as_sym().to_string();

    vm.vars.insert(sym, value);
}

fn dup(vm: &mut Vm) {
    let value = vm.stack.last().unwrap();
    vm.stack.push(value.clone());
}

fn exch(vm: &mut Vm) {
    // [second, last] -> [last, second]
    let last = vm.stack.pop().unwrap();
    let second = vm.stack.pop().unwrap();
    vm.stack.push(last); 
    vm.stack.push(second);
}

// 値を標準出力に出力する関数
fn puts(vm: &mut Vm) {
    let value = vm.stack.pop().unwrap();
    println!("{}", value.to_string());
}

// Vmの状態の差分を表示する関数
fn debug_vm_diff(code: &str, before: &Vm, after: &Vm) {
    println!("--------------------------------");
    println!("Code: {}\n", code);

    // スタックの差分を表示
    if before.stack != after.stack {
        println!("Stack: {:?}\n -> {:?}\n", before.stack, after.stack);
    }

    // 変数の差分を表示
    if before.vars != after.vars {
        let added: HashMap<_, _> = after
            .vars
            .iter()
            .filter(|(key, _)| !before.vars.contains_key(*key))
            .collect();
        println!("Vars Added: {:?}\n", added);
    }

    // ブロックの差分を表示
    if before.blocks != after.blocks {
        println!("Blocks: {:?} \n-> {:?}\n", before.blocks, after.blocks);
    }
}

#[cfg(test)]
mod test {
    use super::{parse_batch, Value::*};
    use std::io::Cursor;

    #[test]
    fn test_group() {
        assert_eq!(
            parse_batch(Cursor::new("1 2 + { 3 4 * }")),
            vec![Num(3), Block(vec![Num(3), Num(4), Op("*".to_string())])]
        );
    }

    #[test]
    fn test_if_false() {
        assert_eq!(
            parse_batch(Cursor::new("{ 0 } { 1 } { -1 } if")),
             vec![Num(-1)]
        );
    }

    #[test]
    fn test_if_true() {
        assert_eq!(
            parse_batch(Cursor::new("{ 1 } { 1 } { -1 } if")),
             vec![Num(1)]
        );
    }

    #[test]
    fn test_multiline() {
        // 複数行の標準入力をシミュレートする
        let input = r#"
/x 10 def
/y 20 def

{ x y < }
{ x }
{ y }
if
"#;
        // Cursorを用いて標準入力をシミュレート
        let cursor = Cursor::new(input.as_bytes());

        // 結果を確認
        assert_eq!(parse_batch(cursor), vec![Num(10)]);
    }

    #[test]
    fn test_function_definition() {
        // 複数行の標準入力をシミュレートする
        let input = r#"
/double { 2 * } def
10 double
"#;
        // Cursorを用いて標準入力をシミュレート
        let cursor = Cursor::new(input.as_bytes());

        // 結果を確認
        assert_eq!(parse_batch(cursor), vec![Num(20)]);
    }
}
