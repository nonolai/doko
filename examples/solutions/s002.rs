pub fn solve() -> String {
    let mut prev = 1;
    let mut cur = 1;
    let mut total = 0;
    while cur < 4_000_000 {
        if cur % 2 == 0 {
            total = total + cur;
        }
        let tmp = cur;
        cur = cur + prev;
        prev = tmp;
    }
    total.to_string()
}
