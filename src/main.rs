use std::{collections::HashMap, io::BufRead, vec};

// 仮想マシンの構造体を定義
struct Vm {
    stack: Vec<Value>,            // スタックを保持するベクタ
    vars: HashMap<String, Value>, // 変数を保持するハッシュマップ
    blocks: Vec<Vec<Value>>,      // ブロックを保持するベクタ
}

impl Vm {
    fn new() -> Self {
        Self {
            stack: vec![],
            vars: HashMap::new(),
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
        }
    }
}

fn main() {
    parse_interactive();
}

fn parse_batch(source: impl BufRead) -> Vec<Value> {
    let mut vm = Vm::new();
    for line in source.lines().flatten() {
        for word in line.split(" ") {
            parse_word(word, &mut vm);
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
    // ブロック構造の中にある場合、評価せずにブロックにコードを追加する
    if let Some(top_block) = vm.blocks.last_mut() {
        top_block.push(code);
        return;
    }
    match code {
        Value::Op(ref op) => match op.as_str() {
            "+" => add(&mut vm.stack),
            "-" => sub(&mut vm.stack),
            "*" => mul(&mut vm.stack),
            "/" => div(&mut vm.stack),
            "<" => lt(&mut vm.stack),
            "if" => op_if(vm),
            "def" => op_def(vm),
            "puts" => puts(vm),
            _ => {
                let val = vm
                    .vars
                    .get(op)
                    .expect(&format!("{op:?} is not a defined operation"));
                vm.stack.push(val.clone());
            }
        },
        _ => vm.stack.push(code.clone()),
    }
}

// 関数を作るマクロ addなどの関数を作成する
macro_rules! impl_op {
    { $name:ident, $op:tt } => {
        fn $name(stack: &mut Vec<Value>) {
            let rhs = stack.pop().unwrap().as_num();
            let lhs = stack.pop().unwrap().as_num();
            stack.push(Value::Num((lhs $op rhs) as i32));
        }
    }
}

// 二項演算子の関数を作成する
impl_op!(add, +);
impl_op!(sub, -);
impl_op!(mul, *);
impl_op!(div, /);
impl_op!(lt, <);

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

// 値を標準出力に出力する関数
fn puts(vm: &mut Vm) {
    let value = vm.stack.pop().unwrap();
    println!("{}", value.to_string());
}

#[cfg(test)]
mod test {
    use super::{parse, Value::*, Vm};

    #[test]
    fn test_group() {
        let mut vm = Vm::new();
        assert_eq!(
            parse("1 2 + { 3 4 * }", &mut vm),
            vec![Num(3), Block(vec![Num(3), Num(4), Op("*".to_string())])]
        );
    }

    #[test]
    fn test_if_false() {
        let mut vm = Vm::new();
        assert_eq!(parse("{ 0 } { 1 } { -1 } if", &mut vm), vec![Num(-1)]);
    }

    #[test]
    fn test_if_true() {
        let mut vm = Vm::new();
        assert_eq!(parse("{ 1 } { 1 } { -1 } if", &mut vm), vec![Num(1)]);
    }

    #[test]
    fn test_multiline() {
        use std::io::BufRead;
        use std::io::Cursor;
        // 複数行の標準入力をシミュレートする
        let input = r#"
/x 10 def
/y 20 def

{ x y < }
{ x }
{ y }
if
"#;
        let mut vm = Vm::new();

        // Cursorを用いて標準入力をシミュレート
        let cursor = Cursor::new(input.as_bytes());
        for line in cursor.lines().flatten() {
            parse(&line, &mut vm);
        }

        // 結果を確認
        assert_eq!(vm.stack, vec![Num(10)]);
    }
}
