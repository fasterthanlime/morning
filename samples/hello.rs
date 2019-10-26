// Our main function
fn main() {
    let x = 1;
    let y = 0;
    loop {
        x = x + 1;
        y += x;
        if x > 10 {}
    }

    {
        x += y;
    }
}
