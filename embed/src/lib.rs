use std::thread;

#[no_mangle]
pub extern "C" fn process() -> u64 {
    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let mut x = 0;
                for _ in 0..5_000_000 {
                    x += 1
                }
                x
            })
        })
        .collect();

    let mut ret_val = 0;

    for h in handles {
        let jr = h.join();
        if jr.is_ok() {
            ret_val += jr.unwrap();
        }
    }

    return ret_val;
}

#[cfg(test)]
mod test {
    use crate::process;

    #[test]
    fn it_works() {
        assert_eq!(10 * 5000000, process());
    }
}
