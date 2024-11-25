#[cfg(test)]
mod test {

    use lock_ordering::{
        lock::MutexLockLevel, relation::LockAfter, LockLevel, LockedAt, MutualExclusion, Unlocked,
    };

    struct FirstLock;
    struct SecondLock;
    struct ThirdLock;

    // Either lock can be acquired without acquiring the other, but if the second
    // lock *is* held, the first lock can't be acquired.
    impl LockAfter<Unlocked> for FirstLock {}
    impl LockAfter<Unlocked> for SecondLock {}
    impl LockAfter<Unlocked> for ThirdLock {}
    impl LockAfter<FirstLock> for SecondLock {}
    impl LockAfter<SecondLock> for ThirdLock {}
    // this should not be possible: is a cycle, on fuchsia should should not compile
    impl LockAfter<ThirdLock> for SecondLock {}

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

    impl LockLevel for ThirdLock {
        type Method = MutualExclusion;
    }
    impl MutexLockLevel for ThirdLock {
        type Mutex = std::sync::Mutex<u32>;
    }

    #[test]
    fn main_fail() {
        let first = std::sync::Mutex::new(1234);
        let second = std::sync::Mutex::new('b');
        let third = std::sync::Mutex::new(3u32);

        let mut _locked = LockedAt::new();
        let mut locked_third = LockedAt::new();

        // This is fine: the second lock can be acquired without holding the first.
        let (mut _locked, mut second_guard) = _locked.with_lock::<SecondLock>(&second).unwrap();
        *second_guard = 'c';
        // let (mut locked_third_from_second, mut third_guard) =
        //     locked.with_lock::<ThirdLock>(&third).unwrap();

        let (mut _locked, mut third_guard) = locked_third.with_lock::<ThirdLock>(&third).unwrap();
        *third_guard = 4u32;
        // let (mut locked, mut third_guard) = locked.with_lock::<SecondLock>(&second).unwrap();

        // This is problematic: the first lock can't be acquired while the second is
        // held.
        // let (mut locked, mut first_guard) = locked.with_lock::<FirstLock>(&first).unwrap();
        let mut locked = LockedAt::new();
        let (mut _locked, mut first_guard) = locked.with_lock::<FirstLock>(&first).unwrap();
        *first_guard = 666;
    }
}
