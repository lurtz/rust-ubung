use std::marker::PhantomPinned;
use std::pin::Pin;

#[derive(Default)]
pub struct AddrTracker {
    prev_addr: Option<usize>,
    // remove auto-implemented `Unpin` bound to mark this type as having some
    // address-sensitive state. This is essential for our expected pinning
    // guarantees to work, and is discussed more below.
    _pin: PhantomPinned,
}

impl AddrTracker {
    pub fn check_for_move(self: Pin<&mut Self>) {
        let current_addr = &*self as *const Self as usize;
        match self.prev_addr {
            None => {
                // SAFETY: we do not move out of self
                let self_data_mut = unsafe { self.get_unchecked_mut() };
                self_data_mut.prev_addr = Some(current_addr);
            }
            Some(prev_addr) => assert_eq!(prev_addr, current_addr),
        }
    }

    pub fn print_addr(&self) {
        let current_addr = self as *const Self as usize;
        println!(
            "addr == {:?}, current addr == {}",
            self.prev_addr, current_addr
        )
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Borrow;
    use std::pin::{Pin, pin};

    use crate::AddrTracker;

    #[test]
    fn main() {
        // 1. Create the value, not yet in an address-sensitive state
        let tracker = AddrTracker::default();

        {
            // 2. Pin the value by putting it behind a pinning pointer, thus putting
            // it into an address-sensitive state
            let mut ptr_to_pinned_tracker: Pin<&mut AddrTracker> = pin!(tracker);
            ptr_to_pinned_tracker.as_mut().check_for_move();

            // Trying to access `tracker` or pass `ptr_to_pinned_tracker` to anything that
            // requires mutable access to a non-pinned version of it will no longer compile

            // 3. We can now assume that the tracker value will never be moved, thus
            // this will never panic!
            ptr_to_pinned_tracker.as_mut().check_for_move();

            // tracker = ptr_to_pinned_tracker.u;
            ptr_to_pinned_tracker.borrow();
            // ptr_to_pinned_tracker.check_for_move();
            ptr_to_pinned_tracker.print_addr();

            // Assignment is not available because AddrTracker is !Unpin
            // ptr_to_pinned_tracker.prev_addr = None;

            // get_mut() is not available because AddrTracker is !Unpin
            // let ptr2 = ptr_to_pinned_tracker.get_mut();
            // ptr2.print_addr();
        }

        // tracker.print_addr();
    }
}
