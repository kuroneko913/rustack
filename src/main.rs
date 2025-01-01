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

fn main() {
    for line in std::io::stdin().lines().flatten() {
        parse(&line);
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
            match word {
                "+" => add(&mut stack),
                "-" => sub(&mut stack),
                "*" => mul(&mut stack),
                "/" => div(&mut stack),
                _ => panic!("{word:?} could not be parsed"),
            }
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
}