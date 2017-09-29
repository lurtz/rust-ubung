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
            use std::rc::Rc;
            use libc::c_void;

            pub type CallbackFn = $function;
            pub type CallbackBoxed = Box<CallbackFn>;
            pub type CallbackBoxed2 = Rc<CallbackBoxed>;
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

    callback_types![client_callback, [libc, avahi_sys, avahi2::avahi], Fn(avahi::ClientState), unsafe extern "C" fn (_client: *mut avahi_sys::AvahiClient, _state: avahi_sys::AvahiClientState, _userdata: *mut libc::c_void), avahi::callback_fn];

    pub type ServiceResolver = avahi_sys::AvahiServiceResolver;
    pub type IfIndex = avahi_sys::AvahiIfIndex;
    pub type Protocol = avahi_sys::AvahiProtocol;
    pub type LookupFlags = avahi_sys::AvahiLookupFlags;
    pub type ResolverEvent = avahi_sys::AvahiResolverEvent;
    pub type Address = avahi_sys::AvahiAddress;
    pub type StringList = avahi_sys::AvahiStringList;

    unsafe extern "C" fn callback_fn_resolver(
        r: *mut ServiceResolver,
           _interface: IfIndex,
           _protocol: Protocol,
           _event: ResolverEvent,
           _name: *const ::libc::c_char,
           _type_: *const ::libc::c_char,
           _domain: *const ::libc::c_char,
           host_name: *const ::libc::c_char,
           _a: *const Address,
           _port: u16,
           _txt: *mut StringList,
           _flags: LookupResultFlags,
           userdata: *mut ::libc::c_void) {
        if avahi_sys::AvahiResolverEvent::AVAHI_RESOLVER_FOUND == _event {
            let functor : &resolver_callback::CallbackBoxed = std::mem::transmute(userdata);

            let host_name_string;
            if std::ptr::null() != host_name {
                host_name_string = ffi::CStr::from_ptr(host_name).to_string_lossy().into_owned()
            } else {
                host_name_string = String::from("");
            }

            functor(&host_name_string);
        }
        avahi_sys::avahi_service_resolver_free(r);
    }


    callback_types![resolver_callback, [avahi2::avahi], Fn(&str), unsafe extern "C" fn(
         *mut avahi::ServiceResolver,
                           avahi::IfIndex,
                           avahi::Protocol,
                           avahi::ResolverEvent,
                           *const ::libc::c_char,
                           *const ::libc::c_char,
                           *const ::libc::c_char,
                           *const ::libc::c_char,
                           *const avahi::Address,
                           u16,
                           *mut avahi::StringList,
                           avahi::LookupResultFlags,
                           *mut ::libc::c_void
        ), avahi::callback_fn_resolver];

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
        functor(_state);
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

        pub fn simple_poll_iterate(&mut self, mut sleep_time: i32) {
            unsafe {
                use std::thread;
                use std::time::Duration;

                while sleep_time > 0 && 0 == avahi_sys::avahi_simple_poll_iterate(self.poller.get_raw(), 0) {
                    thread::sleep(Duration::from_millis(100));
                    sleep_time -= 100;
                }
            }
        }

        pub fn create_service_resolver(&self, ifindex: IfIndex, prot: Protocol, name: &str, type_: &str, domain: &str, cb: resolver_callback::Callback) {
            unsafe {
                let (callback, userdata) = resolver_callback::get_callback_with_data(&cb);

                let name_string = ffi::CString::new(name).unwrap();
                let type_string = ffi::CString::new(type_).unwrap();
                let domain_string = ffi::CString::new(domain).unwrap();

                avahi_sys::avahi_service_resolver_new(self.client, ifindex, prot, name_string.as_ptr(), type_string.as_ptr(), domain_string.as_ptr(), -1, std::mem::transmute(0), callback, userdata);
            }
        }
    }

    impl Drop for Client {
        fn drop(&mut self) {
            self.service_browser = None;
            unsafe {
                avahi_sys::avahi_client_free(self.client);
            }
        }
    }

    pub type BrowserEvent = avahi_sys::AvahiBrowserEvent;
    pub type LookupResultFlags = avahi_sys::AvahiLookupResultFlags;

    callback_types![service_browser_callback, [libc, avahi_sys, avahi2::avahi], Fn(avahi::IfIndex, avahi::Protocol, avahi::BrowserEvent, &str, &str, &str, avahi::LookupResultFlags), unsafe extern "C" fn (_service_browser: *mut avahi_sys::AvahiServiceBrowser, _ifindex: avahi_sys::AvahiIfIndex, _protocol: avahi_sys::AvahiProtocol, _event: avahi_sys::AvahiBrowserEvent, *const libc::c_char, *const libc::c_char, *const libc::c_char, _flags: avahi_sys::AvahiLookupResultFlags, _userdata: *mut libc::c_void), avahi::service_browser_callback_fn];

    unsafe extern "C" fn service_browser_callback_fn(
        _service_browser: *mut avahi_sys::AvahiServiceBrowser, _ifindex: avahi_sys::AvahiIfIndex, _protocol: avahi_sys::AvahiProtocol, _event: avahi_sys::AvahiBrowserEvent, _name: *const c_char, _type: *const c_char, _domain: *const c_char, _flags: avahi_sys::AvahiLookupResultFlags, _userdata: *mut c_void) {
        let functor : &service_browser_callback::CallbackBoxed = std::mem::transmute(_userdata);

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

        functor(_ifindex, _protocol, _event, &name_string, &type_string, &domain_string, _flags);
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
        use std::rc::Rc;

        #[test]
        fn get_callback_with_data_without_callback_works() {
            let (c_callback, data) = get_callback_with_data(&None);
            assert_eq!(None, c_callback);
            assert_eq!(None, c_callback);
            assert_eq!(std::ptr::null_mut(), data);
        }

        #[test]
        fn get_callback_with_data_with_callback_works() {
            let cb: CallbackBoxed2 = Rc::new(Box::new(|_| {}));
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
    use avahi2::avahi::resolver_callback;
    use avahi2::avahi::Client;

    use avahi_sys::{AvahiClient, AvahiClientFlags, AvahiClientState};
    use avahi_sys::{avahi_client_new, avahi_client_free};
    use avahi_sys::{avahi_simple_poll_new, avahi_simple_poll_get, avahi_simple_poll_free};

    use std::ptr;
    use std::rc::Rc;
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

    #[test]
    fn address_of_closures() {
        use std::rc::Rc;
        type ClosureFn = Fn(u32);
        type BoxedClosure = Box<ClosureFn>;

        let bla: Box<u32> = Box::new(42);
        println!("ptr of int on heap {:p}", bla);
        let cb: Rc<BoxedClosure> = Rc::new(Box::new(|n| { println!("blub {}", n); }));

        println!("address of static function {:?}", address_of_closures as * const fn());
        println!("stack address of cb {:?}", &cb as * const Rc<BoxedClosure>);
        println!("pointer of callback on heap via :p {:p}", cb);

        let cb_ref : &BoxedClosure = &*cb;
        let cb_ptr = cb_ref as * const BoxedClosure;
        println!("pointer of callback on heap via cast {:?}", cb_ptr);
        println!("pointer of callback on heap casted without intermediaries {:?}",  &*cb as &BoxedClosure as * const BoxedClosure);
        println!("pointer of callback on heap casted without intermediaries {:?}",  &*cb as &BoxedClosure as * const BoxedClosure as * const c_void);

        unsafe {
            (*cb_ptr)(3);
        }
    }

    #[test]
    fn constructor_without_callback_works() {
        let _ = Client::new(None);
    }

    #[test]
    fn constructor_with_callback_works() {
        let cb: CallbackBoxed2 = Rc::new(Box::new(|state| {println!("received state: {:?}", state);}));
        let _ = Client::new(Some(cb));
    }

    #[test]
    fn create_service_browser_with_callback() {
        use avahi_sys;
        use avahi2::avahi;
        use std::sync::mpsc::channel;

        let cb: CallbackBoxed2 = Rc::new(Box::new(|state| {println!("received state: {:?}", state);}));
        let mut client = Client::new(Some(cb)).unwrap();

        let (tx, rx) = channel::<(avahi::IfIndex, avahi::Protocol, String, String, String)>();
        let sbcb: service_browser_callback::CallbackBoxed2 = Rc::new(Box::new(
                move |_ifindex, _protocol, _event, name_string, type_string, domain_string, _flags| {
                    println!("received service: name {}, type {}, domain {}", name_string, type_string, domain_string);
                    if avahi_sys::AvahiBrowserEvent::AVAHI_BROWSER_NEW == _event {
                        tx.send((_ifindex, _protocol, name_string.to_owned(), type_string.to_owned(), domain_string.to_owned())).unwrap();
                    }
                }));

        let sb1 = client.create_service_browser("_raop._tcp", sbcb);
        assert!(sb1.is_ok());

        client.simple_poll_iterate(1000);

        let (tx_host, rx_host) = channel::<String>();
        let scrcb: resolver_callback::CallbackBoxed2 = Rc::new(Box::new(move |host_name| {
            println!("hostname: {}" , host_name);
            tx_host.send(host_name.to_owned()).unwrap();
        }));

        while let Ok(response) = rx.try_recv() {
            println!("callback sent: {:?}", response);
            client.create_service_resolver(response.0, response.1, &response.2, &response.3, &response.4, Some(scrcb.clone()));
        }
        client.simple_poll_iterate(1000);

        while let Ok(hostname) = rx_host.try_recv() {
            println!("hostname received: {:?}", hostname);
        }
    }
}

