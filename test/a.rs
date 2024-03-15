// fn processNumber(number: i32) {
//     if number > 0 {
//         println!("Number is positive.");
//     } else if number < 0 {
//         println!("Number is negative.");
//     } else {
//         println!("Number is zero.");
//     }
// }

fn main() {
    let num = 10;
    // processNumber(num);
    let mut a;
    if num > 0 {
        a = 1;
    } else if num < 0 {
        a = 2;
    } else {
        a = 3;
    }
    a += 1;
}
