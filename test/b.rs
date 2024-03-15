fn isEven(num: i32) -> bool {
    return num % 2 == 0;
}

fn isOdd(num: i32) -> bool {
    return num % 2 != 0;
}

fn ifStatementExample(x: i32) {
    if x > 0 && isEven(x) {
        println!("x is positive and even");
    } else if x > 0 && isOdd(x) {
        println!("x is positive and odd");
    } else if x < 0 {
        println!("x is negative");
    } else {
        println!("x is zero");
    }
}

fn switchStatementExample(grade: char) {
    match grade {
        'A' => {
            println!("Excellent!");
        }
        'B' => {
            println!("Good job!");
        }
        'C' => {
            println!("Passing grade");
        }
        _ => {
            println!("Invalid grade");
        }
    }
}

fn divide(dividend: f64, divisor: f64) -> f64 {
    if divisor == 0.0 {
        println!("Divisor cannot be zero");
    }
    return dividend / divisor;
}

fn loopStatementExample() {
    for i in 0..6 {
        println!("Iteration {}", i);
    }
    let mut j = 0;
    while j < 5 {
        println!("While loop iteration {}", j);
        j += 1;
    }
}

fn main() {
    let x = 10;
    ifStatementExample(x);

    let grade = 'B';
    switchStatementExample(grade);

    loopStatementExample();
}
