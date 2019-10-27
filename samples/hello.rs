// Our main function
fn main() {
    let x = 1;
    let y = 0;

    loop {
        y += x;
        x += 1;
        if x > 10 {
            break;
        }
    }

    print(x);
}
