#![allow(dead_code)]

use std::ptr;

use avahi_sys::AvahiClient;
use avahi_sys::AvahiClientFlags;
use avahi_sys::AvahiClientState;
use avahi_sys::avahi_client_new;
use avahi_sys::avahi_client_free;
use avahi_sys::avahi_simple_poll_new;
use avahi_sys::avahi_simple_poll_get;
use avahi_sys::avahi_simple_poll_free;


mod avahi {
    use std;
    use avahi_sys;
    use libc;
    use libc::{c_void, c_int};

    struct Poller {
        poller : * mut avahi_sys::AvahiSimplePoll,
    }

    impl Poller {
        fn new() -> Option<Poller> {
            unsafe {
                let poller = avahi_sys::avahi_simple_poll_new();
                if std::ptr::null() != poller {
                    return Some(Poller{poller: poller});
                } else {
                    return None;
                }
            }
        }

        unsafe fn get(&mut self) -> * const avahi_sys::AvahiPoll {
            avahi_sys::avahi_simple_poll_get(self.poller)
        }

        unsafe fn get_raw(&mut self) -> * mut avahi_sys::AvahiSimplePoll {
            return self.poller;
        }
    }

    impl Drop for Poller {
        fn drop(&mut self) {
            unsafe {
                avahi_sys::avahi_simple_poll_free(self.poller);
            }
        }
    }

    pub type AvahiClientState = avahi_sys::AvahiClientState;
    pub type CallbackFn = Fn(&Client, AvahiClientState);
    pub type CallbackBoxed = Box<CallbackFn>;
    pub type CallbackBoxed2 = Box<CallbackBoxed>;
    pub type Callback = Option<CallbackBoxed2>;

    pub struct Client {
        poller : Poller,
        client : * mut avahi_sys::AvahiClient,
        callback : Callback,
        wrapped : bool,
    }

    type CCallback = unsafe extern "C" fn (_client: *mut avahi_sys::AvahiClient,
                                  _state: avahi_sys::AvahiClientState,
                                  _userdata: *mut c_void);

    unsafe extern "C" fn callback_fn(_client: *mut avahi_sys::AvahiClient,
                                  _state: avahi_sys::AvahiClientState,
                                  _userdata: *mut c_void) {
        println!("callback is at: {:?}", _userdata);
        let functor : &mut CallbackBoxed = std::mem::transmute(_userdata);
        if let Some(client) = Client::wrap(_client) {
            functor(&client, _state);
        }
    }

    fn get_callback_with_data(user_callback: &Callback) -> (Option<CCallback>, *mut c_void) {
         let callback : Option<unsafe extern "C" fn(*mut avahi_sys::AvahiClient, avahi_sys::AvahiClientState, *mut libc::c_void)>;
        let userdata : *mut c_void;
        if let Some(ref cb_box) = *user_callback {
            callback = Some(callback_fn);
            userdata = &(*(*cb_box)) as * const CallbackBoxed as * mut CallbackBoxed as * mut c_void;
        } else {
            callback = None;
            userdata = std::ptr::null_mut();
        }

        return (callback, userdata);
    }

    impl Client {
        pub fn new(user_callback: Callback) -> Option<Client> {
            unsafe {
                let (callback, userdata) = get_callback_with_data(&user_callback);

                println!("callback is at: {:?}", userdata);

                if let Some(mut poller) = Poller::new() {
                    let mut err: c_int = 0;
                    let client = avahi_sys::avahi_client_new(
                                          poller.get(),
                                          avahi_sys::AvahiClientFlags(0),
                                          callback,
                                          userdata,
                                          &mut err);
                    if 0 == err {
                        return Some(Client{poller: poller, client: client, callback: user_callback, wrapped: false});
                    }
                }
                return None;
            }
        }
        fn wrap(client: *mut avahi_sys::AvahiClient) -> Option<Client> {
            if let Some(poller) = Poller::new() {
                return Some(Client{poller: poller, client: client, callback: None, wrapped: true});
            }
            return None;
        }
    }

    impl Drop for Client {
        fn drop(&mut self) {
            if !self.wrapped {
                unsafe {
                    avahi_sys::avahi_client_free(self.client);
                }
            }
        }
    }

    #[cfg(test)]
    mod test {
        use avahi2::avahi::get_callback_with_data;
        use avahi2::avahi::CallbackBoxed;
        use avahi2::avahi::CallbackBoxed2;
        use avahi2::avahi::callback_fn;
        use avahi2::avahi::CCallback;
        use libc;
        use std;

        #[test]
        fn get_callback_with_data_without_callback_works() {
            let (c_callback, data) = get_callback_with_data(&None);
            assert_eq!(None, c_callback);
            assert_eq!(None, c_callback);
            assert_eq!(std::ptr::null_mut(), data);
        }

        #[test]
        fn get_callback_with_data_with_callback_works() {
            let cb: CallbackBoxed2 = Box::new(Box::new(|_, _| {}));
            let expected_userdata_cfn = &*cb as * const CallbackBoxed;
            let expected_userdata_mfn = expected_userdata_cfn as * mut CallbackBoxed;
            let expected_userdata_mv = expected_userdata_mfn as * mut libc::c_void;
            assert!(0x1 != expected_userdata_mv as usize);
            let (c_callback, data) = get_callback_with_data(&Some(cb));
            assert!(c_callback.is_some());
            if let Some(callback) = c_callback {
                let expected_callback = callback_fn as * const CCallback;
                let actual_callback = callback as * const CCallback;
                assert_eq!(expected_callback, actual_callback);
            }

            assert_eq!(expected_userdata_mv, data);
        }

        #[test]
        fn address_of_closures() {
            type ClosureFn = Fn(u32);
            type BoxedClosure = Box<ClosureFn>;

            let bla: Box<u32> = Box::new(42);
            println!("ptr of int on heap {:p}", bla);
            let cb: Box<BoxedClosure> = Box::new(Box::new(|n| { println!("blub {}", n); }));

            println!("address of static function {:?}", address_of_closures as * const fn());
            println!("stack address of cb {:?}", &cb as * const Box<BoxedClosure>);
            println!("pointer of callback on heap via :p {:p}", cb);

            let cb_ref : &BoxedClosure = &*cb;
            println!("pointer of callback on heap via cast {:?}", cb_ref as * const BoxedClosure);
            println!("pointer of callback on heap casted without intermediaries {:?}",  &*cb as &BoxedClosure as * const BoxedClosure);
            println!("pointer of callback on heap casted without intermediaries {:?}",  &*cb as &BoxedClosure as * const BoxedClosure as * const libc::c_void);
        }
    }
}

fn bla() {
    use libc::{c_void, c_int};

    unsafe {
        let mut err: c_int = 0;
        unsafe extern "C" fn callback(_client: *mut AvahiClient,
                                      _state: AvahiClientState,
                                      _userdata: *mut c_void) {
        }

        let poller = avahi_simple_poll_new();
        let client = avahi_client_new(avahi_simple_poll_get(poller),
                                      AvahiClientFlags(0),
                                      Some(callback),
                                      ptr::null_mut(),
                                      &mut err);
        if err == 0
        // TODO AVAHI_OK, avahi_strerror..
        {
            avahi_client_free(client);
        }
        avahi_simple_poll_free(poller);
    }
}

#[cfg(test)]
mod test {
    use avahi2::avahi::CallbackBoxed2;
    use avahi2::avahi::Client;

    #[test]
    fn constructor_without_callback_works() {
        let _ = Client::new(None);
    }

    #[test]
    fn constructor_with_callback_works() {
        let cb: CallbackBoxed2 = Box::new(Box::new(|_, _| {}));
        let _ = Client::new(Some(cb));
    }
}

