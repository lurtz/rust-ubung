#![allow(dead_code)]

mod avahi {
    use std;
    use avahi_sys;
    use libc::{c_void, c_int, c_char};
    use std::ffi;

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

    pub type ClientState = avahi_sys::AvahiClientState;

    macro_rules! callback_types {
        ($name:ident, [$($module:path),*], $function:ty, $ccallback:ty, $callback_fn:path) => {pub mod $name {
            $(use $module;)*
            use std;
            use libc::c_void;

            pub type CallbackFn = $function;
            pub type CallbackBoxed = Box<CallbackFn>;
            pub type CallbackBoxed2 = Box<CallbackBoxed>;
            pub type Callback = Option<CallbackBoxed2>;
            pub type CCallback = $ccallback;

            pub fn get_callback_with_data(user_callback: &Callback) -> (Option<CCallback>, *mut c_void) {
                let callback : Option<CCallback>;
                let userdata : *mut c_void;
                if let Some(ref cb_box) = *user_callback {
                    callback = Some($callback_fn);
                    userdata = &(*(*cb_box)) as * const CallbackBoxed as * mut CallbackBoxed as * mut c_void;
                } else {
                    callback = None;
                    userdata = std::ptr::null_mut();
                }

                return (callback, userdata);
            }
        }}
    }

    callback_types![client_callback, [libc, avahi_sys, avahi2::avahi], Fn(&avahi::ClientTrait, avahi::ClientState), unsafe extern "C" fn (_client: *mut avahi_sys::AvahiClient, _state: avahi_sys::AvahiClientState, _userdata: *mut libc::c_void), avahi::callback_fn];

    pub trait ClientTrait {
    }

    struct WrappedClient {
        client : * mut avahi_sys::AvahiClient,
    }

    impl ClientTrait for WrappedClient {}

    impl WrappedClient {
        fn new(client : * mut avahi_sys::AvahiClient) -> WrappedClient {
            WrappedClient{client}
        }
    }

    pub struct Client {
        poller : Poller,
        client : * mut avahi_sys::AvahiClient,
        callback : client_callback::Callback,
        service_browser: Option<ServiceBrowser>,
    }

//    impl Client_trait for Client {}

    unsafe extern "C" fn callback_fn(_client: *mut avahi_sys::AvahiClient,
                                  _state: avahi_sys::AvahiClientState,
                                  _userdata: *mut c_void) {
        let functor : &client_callback::CallbackBoxed = std::mem::transmute(_userdata);
        let client = WrappedClient::new(_client);
        functor(&client, _state);
    }

    impl Client {
        pub fn new(user_callback: client_callback::Callback) -> Option<Client> {
            unsafe {
                let (callback, userdata) = client_callback::get_callback_with_data(&user_callback);

                if let Some(mut poller) = Poller::new() {
                    let mut err: c_int = 0;
                    let client = avahi_sys::avahi_client_new(
                                          poller.get(),
                                          avahi_sys::AvahiClientFlags(0),
                                          callback,
                                          userdata,
                                          &mut err);
                    if 0 == err {
                        return Some(Client{poller: poller, client: client, callback: user_callback, service_browser: None});
                    }
                }
                return None;
            }
        }

        fn get(&self) -> * mut avahi_sys::AvahiClient {
            return self.client;
        }

        pub fn create_service_browser(&mut self, service_type: &str, callback: service_browser_callback::CallbackBoxed2) -> Result<(), ()> {
            self.service_browser = ServiceBrowser::new(self, service_type, callback);
            if self.service_browser.is_some() {
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn simple_poll_loop(&mut self) -> bool {
            unsafe {
                avahi_sys::avahi_simple_poll_loop(self.poller.get_raw());
            }
            true
        }
    }

    impl Drop for Client {
        fn drop(&mut self) {
            unsafe {
                avahi_sys::avahi_client_free(self.client);
            }
        }
    }

// typedef void(* AvahiServiceBrowserCallback) (AvahiServiceBrowser *b, AvahiIfIndex interface, AvahiProtocol protocol, AvahiBrowserEvent event, const char *name, const char *type, const char *domain, AvahiLookupResultFlags flags, void *userdata)

    pub type BrowserEvent = avahi_sys::AvahiBrowserEvent;
    pub type LookupResultFlags = avahi_sys::AvahiLookupResultFlags;

    callback_types![service_browser_callback, [libc, avahi_sys, avahi2::avahi], Fn(&avahi::WrappedServiceBrowser, avahi::BrowserEvent, &str, &str, &str, avahi::LookupResultFlags), unsafe extern "C" fn (_service_browser: *mut avahi_sys::AvahiServiceBrowser, _ifindex: avahi_sys::AvahiIfIndex, _protocol: avahi_sys::AvahiProtocol, _event: avahi_sys::AvahiBrowserEvent, *const libc::c_char, *const libc::c_char, *const libc::c_char, _flags: avahi_sys::AvahiLookupResultFlags, _userdata: *mut libc::c_void), avahi::service_browser_callback_fn];

    pub trait ServiceBrowserTrait {}

    pub struct WrappedServiceBrowser {
        service_browser: * mut avahi_sys::AvahiServiceBrowser,
    }

    impl WrappedServiceBrowser {
        fn new(service_browser: * mut avahi_sys::AvahiServiceBrowser) -> WrappedServiceBrowser {
            WrappedServiceBrowser{service_browser}
        }
    }

    unsafe extern "C" fn service_browser_callback_fn(
        _service_browser: *mut avahi_sys::AvahiServiceBrowser, _ifindex: avahi_sys::AvahiIfIndex, _protocol: avahi_sys::AvahiProtocol, _event: avahi_sys::AvahiBrowserEvent, _name: *const c_char, _type: *const c_char, _domain: *const c_char, _flags: avahi_sys::AvahiLookupResultFlags, _userdata: *mut c_void) {
        let functor : &service_browser_callback::CallbackBoxed = std::mem::transmute(_userdata);
        let sb = WrappedServiceBrowser::new(_service_browser);

        let name_string;
        if std::ptr::null() != _name {
            name_string = ffi::CStr::from_ptr(_name).to_string_lossy().into_owned()
        } else {
            name_string = String::from("");
        }

        let type_string;
        if std::ptr::null() != _type {
            type_string = ffi::CStr::from_ptr(_type).to_string_lossy().into_owned();
        } else {
            type_string = String::from("");
        }

        let domain_string;
        if std::ptr::null() != _domain {
            domain_string = ffi::CStr::from_ptr(_domain).to_string_lossy().into_owned();
        } else {
            domain_string = String::from("");
        }

        functor(&sb, _event, &name_string, &type_string, &domain_string, _flags);
    }

    pub struct ServiceBrowser {
        service_browser: * mut avahi_sys::AvahiServiceBrowser,
        callback : service_browser_callback::CallbackBoxed2,
    }

    impl ServiceBrowser {
        fn new(client: &Client, service_type: &str, user_callback: service_browser_callback::CallbackBoxed2) -> Option<ServiceBrowser> {
            let cb_option: service_browser_callback::Callback = Some(user_callback);

            unsafe {
                let flag: avahi_sys::AvahiLookupFlags = std::mem::transmute(0);

                let ctype = ffi::CString::new(service_type).unwrap();
                let (callback, userdata) = service_browser_callback::get_callback_with_data(&cb_option);
                let sb = avahi_sys::avahi_service_browser_new(client.get(), -1, -1, ctype.as_ptr(), std::ptr::null(), flag, callback, userdata);
                if std::ptr::null() != sb {
                    Some(ServiceBrowser{service_browser: sb, callback: cb_option.unwrap()})
                } else {
                    println!("error while creating service browser: {}", avahi_sys::avahi_client_errno(client.get()));
                    None
                }
            }
        }

        fn get(&self) -> * mut avahi_sys::AvahiServiceBrowser {
            return self.service_browser;
        }
    }

    impl Drop for ServiceBrowser {
        fn drop(&mut self) {
            unsafe {
                avahi_sys::avahi_service_browser_free(self.service_browser);
            }
        }
    }

    #[cfg(test)]
    mod test {
        use avahi2::avahi::client_callback::get_callback_with_data;
        use avahi2::avahi::client_callback::CallbackBoxed;
        use avahi2::avahi::client_callback::CallbackBoxed2;
        use avahi2::avahi::client_callback::CCallback;
        use avahi2::avahi::callback_fn;
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
            assert!(0x0 != expected_userdata_mv as usize);
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
    }
}

#[cfg(test)]
mod test {
    use avahi2::avahi::client_callback::CallbackBoxed2;
    use avahi2::avahi::service_browser_callback;
    use avahi2::avahi::Client;

    use avahi_sys::{AvahiClient, AvahiClientFlags, AvahiClientState};
    use avahi_sys::{avahi_client_new, avahi_client_free};
    use avahi_sys::{avahi_simple_poll_new, avahi_simple_poll_get, avahi_simple_poll_free};

    use std::ptr;
    use libc::{c_void, c_int};

    #[test]
    fn example_code() {
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

    /*
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
        println!("pointer of callback on heap casted without intermediaries {:?}",  &*cb as &BoxedClosure as * const BoxedClosure as * const c_void);
    }
    */

    #[test]
    fn constructor_without_callback_works() {
        let _ = Client::new(None);
    }

    #[test]
    fn constructor_with_callback_works() {
        let cb: CallbackBoxed2 = Box::new(Box::new(|_, state| {println!("received state: {:?}", state);}));
        let _ = Client::new(Some(cb));
    }

    #[test]
    fn create_service_browser_with_callback() {
        use std::thread;
        use std::time::Duration;

        let cb: CallbackBoxed2 = Box::new(Box::new(|_, state| {println!("received state: {:?}", state);}));
        let mut client = Client::new(Some(cb)).unwrap();
        //let sbcb: service_browser_callback::CallbackBoxed2 = Box::new(Box::new(|_,_,_,_,_,_| {println!("received service")}));
        let sbcb: service_browser_callback::CallbackBoxed2 = Box::new(Box::new(|_, _event, name_string, type_string, domain_string, _flags| {println!("received service: name {}, type {}, domain {}", name_string, type_string, domain_string)}));

        let sb = client.create_service_browser("_presence._tcp", sbcb);
        assert!(sb.is_ok());

        client.simple_poll_loop();

        thread::sleep(Duration::from_millis(50000));

        assert!(false);
    }
}

