use std::collections::HashMap;

// 仮想マシンの構造体を定義
struct Vm {
  stack: Vec<Value>, // スタックを保持するベクタ
  vars: HashMap<String, Value>, // 変数を保持するハッシュマップ
}

impl Vm {
  fn new() -> Self {
    Self {
      stack: vec![],
      vars: HashMap::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
  Num(i32),
  Op(String),
  Block(Vec<Value>),
  Sym(String),
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
    let mut vm = Vm::new();
    for line in std::io::stdin().lines().flatten() {
        parse(&line, &mut vm);
    }
}

fn parse(line: &str, vm: &mut Vm) -> Vec<Value> {
    let input: Vec<_> = line.split(" ").collect();
    let mut words = &input[..]; // 全範囲を意味する

    // 配列が空でない場合、最初の要素（&T）と残りの要素（&[T]）を含むタプルを返す
    while let Some((&word, mut rest)) = words.split_first() {
       if word.is_empty() {
           break;
       }
       if word == "{" {
            // 今見ている要素が { だった場合、parse_block を呼び出して、それ以降の部分をパースする
            let value;
            (value, rest) = parse_block(rest);
            vm.stack.push(value);
            words = rest;
            continue;
       }
       // 値の種類によって、Value のインスタンスを生成しcodeに保持する
       let code = if let Ok(parsed) = word.parse::<i32>() {
           // 数字の場合は、Num としてスタックに積む
           Value::Num(parsed)
       } else if word.starts_with("/") {
           Value::Sym(word[1..].to_string()) // /から始まる文字列を変数名とするため、/を取り除いた文字列を保持する
       } else {
           // 数字、{} 以外の場合、演算子として処理する
           Value::Op(word.to_string())
       };
       eval(code, vm);
       words = rest;
       println!("stack: {:?}", vm.stack);
    }

    println!("stack: {:?}", vm.stack);

    vm.stack.clone()
}

fn eval(code: Value, vm: &mut Vm) {
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
          let val = vm.vars.get(op).expect(&format!(
            "{op:?} is not a defined operation"
          ));
          vm.stack.push(val.clone());
        }
      },
      _ => vm.stack.push(code.clone()),
    }
}

fn parse_block<'a>(input: &'a [&'a str]) -> (Value, &'a [&'a str]) {
    let mut tokens = vec![];
    let mut words = input;

    while let Some((&word, mut rest)) = words.split_first() {
        if word.is_empty() {
            break;
        }
        // {}の中身をtokensに保持して、}が来たら値(数値と演算子を含むオブジェクト)を返却する
        if word == "{" {
            let value;
            (value, rest) = parse_block(rest);
            tokens.push(value);
        } else if word == "}" {
            // {}の中身はBlockとして保持する
            return (Value::Block(tokens), rest);
        } else if let Ok(value) = word.parse::<i32>() {
            tokens.push(Value::Num(value)); // 数字は数字で保持する
        } else {
            tokens.push(Value::Op(word.to_string())); // それ以外は演算子として保持する
        }
        words = rest;
    }
    
    // 最後にtokensを返却する
    (Value::Block(tokens), words)
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
      vec![Num(3), Block(vec![Num(3), Num(4) , Op("*".to_string())])]
    );
  }

  #[test]
  fn test_if_false() {
    let mut vm = Vm::new();
    assert_eq!(
      parse("{ 0 } { 1 } { -1 } if", &mut vm),
      vec![Num(-1)]
    );
  }

  #[test]
  fn test_if_true() {
    let mut vm = Vm::new();
    assert_eq!(
      parse("{ 1 } { 1 } { -1 } if", &mut vm),
      vec![Num(1)]
    );
  }

  #[test]
  fn test_multiline() {
    use std::io::Cursor;
    use std::io::BufRead;
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
