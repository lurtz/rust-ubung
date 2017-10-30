#![allow(dead_code)]

mod avahi {
    use std;
    use avahi_sys;
    use libc::{c_void, c_int, c_char};
    use std::ffi;
    use std::sync::mpsc::Sender;
    use std::sync::{Arc, Mutex};
    use std::rc::Rc;
    use std::thread;
    use std::time::{Duration, Instant};

    type ClientState = avahi_sys::AvahiClientState;
    type ServiceResolver = avahi_sys::AvahiServiceResolver;
    type IfIndex = avahi_sys::AvahiIfIndex;
    type Protocol = avahi_sys::AvahiProtocol;
    type LookupFlags = avahi_sys::AvahiLookupFlags;
    type ResolverEvent = avahi_sys::AvahiResolverEvent;
    type Address = avahi_sys::AvahiAddress;
    type StringList = avahi_sys::AvahiStringList;
    type BrowserEvent = avahi_sys::AvahiBrowserEvent;
    type LookupResultFlags = avahi_sys::AvahiLookupResultFlags;
    type ServiceBrowserMessage = (IfIndex, Protocol, String, String, String);

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

    pub fn time_in_millis(timer: &std::time::Instant) -> u64 {
        let elapsed = timer.elapsed();
        let el_s = elapsed.as_secs() * 1000;
        let el_n = (elapsed.subsec_nanos() / 1_000_000) as u64;
        el_s + el_n
    }

    pub struct Poller {
        poller : * mut avahi_sys::AvahiSimplePoll,
    }

    impl Poller {
        pub fn new() -> Option<Poller> {
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

        fn simple_poll_iterate(&mut self, sleep_time: i32) -> i32 {
            unsafe {
                 avahi_sys::avahi_simple_poll_iterate(self.poller, sleep_time)
            }
        }

        pub fn iterate(&mut self, sleep_time: u64) {
            let start = Instant::now();
            let mut time_remaining = sleep_time;

            while time_remaining != 0 && 0 == self.simple_poll_iterate(time_remaining as i32) {
                let elapsed_time = time_in_millis(&start);
                if sleep_time > elapsed_time {
                    time_remaining = sleep_time - elapsed_time;
                    thread::sleep(Duration::from_millis(25));
                } else {
                    time_remaining = 0;
                }
            }
        }
    }

    impl Drop for Poller {
        fn drop(&mut self) {
            unsafe {
                avahi_sys::avahi_simple_poll_free(self.poller);
            }
        }
    }

    callback_types![
        client_callback,
        [libc, avahi_sys, avahi2::avahi],
        Fn(avahi::ClientState),
        unsafe extern "C" fn (
            _client: *mut avahi_sys::AvahiClient,
            _state: avahi_sys::AvahiClientState,
            _userdata: *mut libc::c_void),
        avahi::callback_fn];

    pub struct Client {
        poller: Arc<Mutex<Poller>>,
        client : * mut avahi_sys::AvahiClient,
        callback : client_callback::Callback,
        service_browser: Option<ServiceBrowser>,
    }

    unsafe extern "C" fn callback_fn(_client: *mut avahi_sys::AvahiClient,
                                  _state: avahi_sys::AvahiClientState,
                                  _userdata: *mut c_void) {
        let functor : &client_callback::CallbackBoxed = std::mem::transmute(_userdata);
        functor(_state);
    }

    impl Client {
        pub fn new(poller: Arc<Mutex<Poller>>, user_callback: client_callback::Callback) -> Option<Client> {
            unsafe {
                let (callback, userdata) = client_callback::get_callback_with_data(&user_callback);

                let native_poller;
                {
                    let poller_locked = poller.lock();
                    native_poller = poller_locked.unwrap().get();
                }

                let mut err: c_int = 0;
                let client = avahi_sys::avahi_client_new(
                                      native_poller,
                                      avahi_sys::AvahiClientFlags(0),
                                      callback,
                                      userdata,
                                      &mut err);
                if 0 == err {
                    return Some(Client{poller, client: client, callback: user_callback, service_browser: None});
                }
                return None;
            }
        }

        fn errno(&self) -> i32 {
            unsafe {
                avahi_sys::avahi_client_errno(self.client)
            }
        }

        pub fn create_service_browser(&mut self, service_type: &str, callback: service_browser_callback::CallbackBoxed2) -> Result<(), ()> {
            self.service_browser = self.create_service_browser2(service_type, callback);
            if self.service_browser.is_some() {
                Ok(())
            } else {
                Err(())
            }
        }

        fn create_service_browser2(&self, service_type: &str, user_callback: service_browser_callback::CallbackBoxed2) -> Option<ServiceBrowser> {
            let cb_option = Some(user_callback);

            unsafe {
                let flag = std::mem::transmute(0);

                let ctype = ffi::CString::new(service_type).unwrap();
                let (callback, userdata) = service_browser_callback::get_callback_with_data(&cb_option);
                let sb = avahi_sys::avahi_service_browser_new(self.client, -1, -1, ctype.as_ptr(), std::ptr::null(), flag, callback, userdata);
                if std::ptr::null() != sb {
                    Some(ServiceBrowser::new(sb, cb_option.unwrap()))
                } else {
                    println!("error while creating service browser: {}", self.errno());
                    None
                }
            }
        }

        pub fn create_service_resolver(&self, ifindex: IfIndex, prot: Protocol, name: &str, type_: &str, domain: &str, cb: resolver_callback::CallbackBoxed2) {
            unsafe {
                let cb_option = Some(cb);
                let (callback, userdata) = resolver_callback::get_callback_with_data(&cb_option);

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

    callback_types![
        service_browser_callback,
        [libc, avahi_sys, avahi2::avahi],
        Fn(avahi::IfIndex, avahi::Protocol, avahi::BrowserEvent, &str, &str, &str, avahi::LookupResultFlags),
        unsafe extern "C" fn (
            _service_browser: *mut avahi_sys::AvahiServiceBrowser,
            _ifindex: avahi_sys::AvahiIfIndex,
            _protocol: avahi_sys::AvahiProtocol,
            _event: avahi_sys::AvahiBrowserEvent,
            *const libc::c_char,
            *const libc::c_char,
            *const libc::c_char,
            _flags: avahi_sys::AvahiLookupResultFlags,
            _userdata: *mut libc::c_void),
        avahi::service_browser_callback_fn];

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

    struct ServiceBrowser {
        service_browser: * mut avahi_sys::AvahiServiceBrowser,
        callback : service_browser_callback::CallbackBoxed2,
    }

    impl ServiceBrowser {
        fn new(service_browser: * mut avahi_sys::AvahiServiceBrowser, callback : service_browser_callback::CallbackBoxed2
            ) -> ServiceBrowser {
            ServiceBrowser{service_browser: service_browser, callback: callback}
        }

    }

    impl Drop for ServiceBrowser {
        fn drop(&mut self) {
            unsafe {
                avahi_sys::avahi_service_browser_free(self.service_browser);
            }
        }
    }

    pub fn create_service_browser_callback(client: Arc<Mutex<Client>>, tx: Sender<String>, name_to_filter: &str) -> service_browser_callback::CallbackBoxed2 {
        let filter_name = String::from(name_to_filter);

        let scrcb: resolver_callback::CallbackBoxed2 = Rc::new(Box::new(move |host_name| {
            tx.send(host_name.to_owned()).unwrap();
        }));

        let sbcb: service_browser_callback::CallbackBoxed2 = Rc::new(Box::new(
            move |_ifindex, _protocol, _event, name_string, type_string, domain_string, _flags| {
                if avahi_sys::AvahiBrowserEvent::AVAHI_BROWSER_NEW == _event {
                    if name_string.contains(&filter_name) {
                        let client_locked = client.lock().unwrap();
                        client_locked.create_service_resolver(_ifindex, _protocol, name_string, type_string, domain_string, scrcb.clone());
                    }
                }
            }));

        sbcb
    }

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

    callback_types![
        resolver_callback,
        [avahi2::avahi],
        Fn(&str),
        unsafe extern "C" fn(
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
            *mut ::libc::c_void),
        avahi::callback_fn_resolver];

    #[cfg(test)]
    mod test {
        use libc;
        use std;
        use std::rc::Rc;
        use avahi2::avahi::client_callback::get_callback_with_data;
        use avahi2::avahi::client_callback::CallbackBoxed;
        use avahi2::avahi::client_callback::CallbackBoxed2;
        use avahi2::avahi::client_callback::CCallback;
        use avahi2::avahi::callback_fn;

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

pub fn get_hostname(type_: &str, filter: &str) -> String {
    use avahi2::avahi;
    use std::sync::mpsc::channel;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    let poller = Arc::new(Mutex::new(avahi::Poller::new().unwrap()));
    let client = Arc::new(Mutex::new(avahi::Client::new(poller.clone(), None).unwrap()));

    let (tx_host, rx_host) = channel();

    let sbcb = avahi::create_service_browser_callback(client.clone(), tx_host, filter);

    let sb1 = client.lock().unwrap().create_service_browser(type_, sbcb);
    assert!(sb1.is_ok());

    let mut hostnames = Vec::new();
    let start = Instant::now();
    let wait_time = 2000;
    while avahi::time_in_millis(&start) < wait_time && hostnames.is_empty() {
        let message = rx_host.try_recv();
        match message {
            Ok(hostname) => hostnames.push(hostname),
            Err(_) => {
                let poller_locked = poller.lock();
                poller_locked.unwrap().iterate(100) },
        }
    }

    if hostnames.len() > 1 {
        println!("multiple hosts found: {:?}, taking: {}", hostnames, hostnames[0]);
    }

    if hostnames.is_empty() {
        println!("No host found!");
        return String::new();
    } else {
        return hostnames[0].clone();
    }
}

pub fn get_receiver() -> String {
    get_hostname("_raop._tcp", "DENON")
}

#[cfg(test)]
mod test {
    use avahi2::avahi::client_callback::CallbackBoxed2;
    use avahi2::avahi::Client;
    use avahi2::avahi::Poller;
    use avahi2;

    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::mpsc::channel;
    use libc::c_void;

    #[test]
    fn address_of_closures() {
        type ClosureFn = Fn(u32);
        type BoxedClosure = Box<ClosureFn>;

        let bla: Box<u32> = Box::new(42);
        println!("ptr of int on heap {:p}", bla);
        let (tx, rx) = channel();
        let cb: Rc<BoxedClosure> = Rc::new(Box::new(move |n| { tx.send(n).is_ok(); println!("blub {}", n); }));

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

        assert!(3 == rx.recv().unwrap());
    }

    #[test]
    fn constructor_without_callback_works() {
        let poller = Arc::new(Mutex::new(Poller::new().unwrap()));
        let _ = Client::new(poller, None);
    }

    #[test]
    fn constructor_with_callback_works() {
        let cb: CallbackBoxed2 = Rc::new(Box::new(|state| {println!("received state: {:?}", state);}));
        let poller = Arc::new(Mutex::new(Poller::new().unwrap()));
        let _ = Client::new(poller, Some(cb));
    }

    #[test]
    fn create_service_browser_with_callback() {
        let receiver = avahi2::get_receiver();
        assert!("DENON-AVR-1912.local" == receiver);
    }

    #[test]
    fn get_hostname() {
        let host = avahi2::get_hostname("_presence._tcp", "");
        assert!("barcas.local" == host);
    }
}

