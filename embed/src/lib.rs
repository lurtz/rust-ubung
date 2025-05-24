use std::thread;

#[unsafe(no_mangle)]
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
        if let Ok(val) = h.join() {
            ret_val += val;
        }
    }

    ret_val
}

#[cfg(test)]
mod test {
    use crate::process;

    #[test]
    fn it_works() {
        assert_eq!(10 * 5000000, process());
    }
}
