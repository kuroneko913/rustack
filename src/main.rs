fn main() {
    for line in std::io::stdin().lines() {
        let mut stack = vec![]; // mutableなスタックを用意

        if let Ok(line) = line {
            // let words: Vec<_> = line.split(" ").collect(); // splitはイテレータを返すのでcollectでVecに変換

            // パースした文字列が数値ならスタックに積む
            for word in line.split(" ") {
                if let Ok(parsed) = word.parse::<i32>() {
                    stack.push(parsed);
                } else {
                    match word {
                        // 演算子が来たらスタックから取り出して計算
                        "+" => add(&mut stack),
                        "-" => sub(&mut stack),
                        "*" => mul(&mut stack),
                        "/" => div(&mut stack),
                        _ => panic!("unknown operator: {word:?}"),
                    }
                }
            }   
        }
        println!("stack: {stack:?}");
    }
}

fn add(stack: &mut Vec<i32>) {
    let a = stack.pop().unwrap();
    let b = stack.pop().unwrap();
    stack.push(a + b);
}
fn sub(stack: &mut Vec<i32>) {
    let a = stack.pop().unwrap();
    let b = stack.pop().unwrap();
    stack.push(a - b);
}
fn mul(stack: &mut Vec<i32>) {
    let a = stack.pop().unwrap();
    let b = stack.pop().unwrap();
    stack.push(a * b);
}
fn div(stack: &mut Vec<i32>) {
    let a = stack.pop().unwrap();
    let b = stack.pop().unwrap();
    stack.push(a / b);
}
