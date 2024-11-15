use std::marker::PhantomData;
use std::sync::Mutex;
use std::sync::MutexGuard;

struct TakenLockPriority<'a, T: ?Sized, const PRIORITY: usize> {
    phantom: PhantomData<&'a mut T>,
}

struct PriorityMutex<'a, T: ?Sized, U, const PRIORITY: usize> {
    previous: PhantomData<&'a mut U>,
    mutex: Mutex<T>,
}

impl<'a, T: ?Sized, U, const PRIORITY: usize> PriorityMutex<'a, T, U, PRIORITY> {
    fn lock<'b, 'c, V, const PREVIOUS_PRIORITY: usize>(
        &self,
        _previous_priority: PhantomData<&'c mut TakenLockPriority<'b, V, PREVIOUS_PRIORITY>>,
    ) -> (TakenLockPriority<'c, Self, PRIORITY>, MutexGuard<'_, T>) {
        const {
            if PREVIOUS_PRIORITY >= PRIORITY {
                panic!("Improper use of lock is detetected")
            }
        }
        (
            TakenLockPriority::<'_, _, PRIORITY> {
                phantom: PhantomData::<&mut _>,
            },
            self.mutex.lock().unwrap(),
        )
    }
}

fn use_priority<'a, 'b, V, const PREVIOUS_PRIORITY: usize>(
    _priority: &'a mut TakenLockPriority<'b, V, PREVIOUS_PRIORITY>,
) -> PhantomData<&'a mut TakenLockPriority<'b, V, PREVIOUS_PRIORITY>> {
    PhantomData
}

pub fn main() {
    let mut root = TakenLockPriority::<'static, (), 0> {
        phantom: PhantomData::<&mut _>,
    };
    let m1 = PriorityMutex::<'_, (), (), 1> {
        previous: PhantomData::<&mut _>,
        mutex: Mutex::new(()),
    };
    let m2 = PriorityMutex::<'_, (), (), 2> {
        previous: PhantomData::<&mut _>,
        mutex: Mutex::new(()),
    };
    {
        let (mut _protector2, _guard2) = m2.lock(use_priority(&mut root));
        let (_protector1, _guard1) = m1.lock(use_priority(&mut root));
    }
    let (mut protector1, _guard1) = m1.lock(use_priority(&mut root));
    let (_protector2, _guard2) = m2.lock(use_priority(&mut protector1));
}
