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

    // from https://cs.opensource.google/fuchsia/fuchsia/+/main:src/connectivity/network/netstack3/core/lock-order/src/relation.rs;l=107;drc=e2e00bc897e7362f33b25a7d98d9d7ba5ff07f69
    // dead code warnings are suppressed, because these types are only used
    // by the borrow checker and will never land in a binary. Maybe the warning
    // also would go away if the full test of fuchsia would be copied
    // forward in graph
    #[allow(dead_code)]
    pub trait LockAfterF<A> {}

    // backward in graph
    #[allow(dead_code)]
    pub trait LockBefore<X> {}

    // automatic backward implementation
    impl<B: LockAfterF<A>, A> LockBefore<B> for A {}

    // creates a graph with edges from A to B and all predecessors of A to B
    #[macro_export]
    macro_rules! impl_lock_after {
        ($A:ty => $B:ty) => {
            impl LockAfterF<$A> for $B {}
            // in case of circular dependency this will create a second trait LockAfterF<$A> implementation and cause compile error
            impl<X: LockBefore<$A>> LockAfterF<X> for $B {}
        };
    }

    #[allow(dead_code)]
    enum A {}
    #[allow(dead_code)]
    enum B {}
    #[allow(dead_code)]
    enum C {}
    #[allow(dead_code)]
    enum D {}

    impl_lock_after!(A => B);
    // impl_lock_after!(B => A);
    impl_lock_after!(B => C);
    impl_lock_after!(C => D);
    // impl_lock_after!(D => A); // this will create a compile error
}
