#[cfg(test)]
mod test {

    use lock_ordering::{
        lock::MutexLockLevel, relation::LockAfter, LockLevel, LockedAt, MutualExclusion, Unlocked,
    };

    struct FirstLock;
    struct SecondLock;

    // Either lock can be acquired without acquiring the other, but if the second
    // lock *is* held, the first lock can't be acquired.
    impl LockAfter<Unlocked> for FirstLock {}
    impl LockAfter<Unlocked> for SecondLock {}
    impl LockAfter<FirstLock> for SecondLock {}

    impl LockLevel for FirstLock {
        type Method = MutualExclusion;
    }
    impl MutexLockLevel for FirstLock {
        type Mutex = std::sync::Mutex<usize>;
    }

    impl LockLevel for SecondLock {
        type Method = MutualExclusion;
    }
    impl MutexLockLevel for SecondLock {
        type Mutex = std::sync::Mutex<char>;
    }

    #[test]
    fn main_fail() {
        let first = std::sync::Mutex::new(1234);
        let second = std::sync::Mutex::new('b');

        let mut locked = LockedAt::new();

        let (mut locked, mut first_guard) = locked.with_lock::<FirstLock>(&first).unwrap();
        *first_guard = 666;

        // This is fine: the second lock can be acquired without holding the first.
        let (mut locked, mut second_guard) = locked.with_lock::<SecondLock>(&second).unwrap();
        *second_guard = 'c';

        // This is problematic: the first lock can't be acquired while the second is
        // held.
        // let (mut locked, mut first_guard) = locked.with_lock::<FirstLock>(&first).unwrap();
    }
}
