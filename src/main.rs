#[derive(Debug, PartialEq, Eq)]
enum Value<'src> {
  Num(i32),
  Op(&'src str),
  Block(Vec<Value<'src>>),
}

impl<'src> Value<'src> {
    fn as_num(&self) -> i32 {
      match self {
        Self::Num(val) => *val,
        _ => panic!("Value is not a number"),
      }
    }
}

impl<'src> Value<'src> {
    fn to_block(self) -> Vec<Value<'src>> {
      match self {
        Self::Block(val) => val,
        _ => panic!("Value is not a block"),
      }
    }
}

fn main() {
    for line in std::io::stdin().lines().flatten() {
        parse(&line);
    }
}

fn eval<'src>(code: Value<'src>, stack: &mut Vec<Value<'src>>) {
    match code {
        Value::Op(op) => match op {
            "+" => add(stack),
            "-" => sub(stack),
            "*" => mul(stack),
            "/" => div(stack),
            "if" => op_if(stack),
            _ => panic!("Unknown operator: {op:?}"),
        },
        _ => stack.push(code),
    }
}

fn parse<'a>(line: &'a str)-> Vec<Value<'a>> {
    let mut stack = vec![];
    let input: Vec<_> = line.split(" ").collect();
    let mut words = &input[..]; // 全範囲を意味する

    // 配列が空でない場合、最初の要素（&T）と残りの要素（&[T]）を含むタプルを返す
    while let Some((&word, mut rest)) = words.split_first() {
       if word.is_empty() {
           break;
       }
       if word == "{" {
           let value;
           // 今見ている要素が { だった場合、parse_block を呼び出して、それ以降の部分をパースする
           (value, rest) = parse_block(rest);
           stack.push(value);
       } else if let Ok(parsed) = word.parse::<i32>() {
            // 数字の場合は、Num としてスタックに積む
           stack.push(Value::Num(parsed));
       } else {
            // 数字、{} 以外の場合、演算子として処理する
            eval(Value::Op(word), &mut stack);
       }
       words = rest;
    }

    println!("stack: {stack:?}");

    stack
}

fn parse_block<'src, 'a>(
    input: &'a [&'src str],
) -> (Value<'src>, &'a [&'src str]) {
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
            tokens.push(Value::Op(word)); // それ以外は演算子として保持する
        }
        words = rest;
    }
    
    // 最後にtokensを返却する
    (Value::Block(tokens), words)
}

fn add(stack: &mut Vec<Value>) {
    let rhs = stack.pop().unwrap().as_num();
    let lhs = stack.pop().unwrap().as_num();
    stack.push(Value::Num(lhs + rhs));
}
  
fn sub(stack: &mut Vec<Value>) {
    let rhs = stack.pop().unwrap().as_num();
    let lhs = stack.pop().unwrap().as_num();
    stack.push(Value::Num(lhs - rhs));
}
  
fn mul(stack: &mut Vec<Value>) {
    let rhs = stack.pop().unwrap().as_num();
    let lhs = stack.pop().unwrap().as_num();
    stack.push(Value::Num(lhs * rhs));
}
  
fn div(stack: &mut Vec<Value>) {
    let rhs = stack.pop().unwrap().as_num();
    let lhs = stack.pop().unwrap().as_num();
    stack.push(Value::Num(lhs / rhs));
}

fn op_if(stack: &mut Vec<Value>) {
    let false_branch = stack.pop().unwrap().to_block();
    let true_branch = stack.pop().unwrap().to_block();
    let cond = stack.pop().unwrap().to_block();

    // 条件式の評価を行う
    for code in cond {
        eval(code, stack);
    }

    // 条件式の評価結果を取得する
    let cond_result = stack.pop().unwrap().as_num();

    // 条件式の結果によって、true_branch か false_branch を評価する
    if cond_result != 0 {
        for code in true_branch {
            eval(code, stack);
        }
    } else {
        for code in false_branch {
            eval(code, stack);
        }
    }
} 

#[cfg(test)]
mod test {
  use super::{parse, Value::*};
  #[test]
  fn test_group() {
    assert_eq!(
      parse("1 2 + { 3 4 }"),
      vec![Num(3), Block(vec![Num(3), Num(4)])]
    );
  }

  #[test]
  fn test_if_false() {
    assert_eq!(
      parse("{ 0 } { 1 } { -1 } if"),
      vec![Num(-1)]
    );
  }

  #[test]
  fn test_if_true() {
    assert_eq!(
      parse("{ 1 } { 1 } { -1 } if"),
      vec![Num(1)]
    );
  }
}
