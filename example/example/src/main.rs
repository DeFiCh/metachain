fn main() {
    let name = "hello";
    if name.chars().next() == Some('h') {
        println!("Hello, world!");
    };
}
#[allow(dead_code)]
fn foo(d: &str) -> &str {
    return d;
}

#[test]
fn foo_bar() {
    assert_eq!("bar", foo("bar"));
}
