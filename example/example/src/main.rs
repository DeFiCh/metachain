fn main() {
    let name = "hello";
    if name.chars().next() == Some('h') {
        println!("Hello, world!");
    };
}

fn foo(d: &str) -> &str {
    return d;
}

#[test]
fn foo_bar() {
    assert_eq!("bar", foo("bar"));
}
