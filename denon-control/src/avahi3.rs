use crate::avahi_error::Error;
use std::any::Any;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};
use zeroconf::prelude::{TEventLoop, TMdnsBrowser};
use zeroconf::{MdnsBrowser, ServiceDiscovery, ServiceType};

#[derive(Default, Debug)]
pub struct Context {
    hostname: Option<String>,
}

fn get_hostname(service_type: ServiceType) -> Result<String, Error> {
    let context: Arc<Mutex<Context>> = Arc::default();
    let mut browser = MdnsBrowser::new(service_type);
    browser.set_service_discovered_callback(Box::new(on_service_discovered));
    browser.set_context(Box::new(context.clone()));
    let event_loop = browser.browse_services()?;

    let timeout = Duration::from_secs(2);
    let start = std::time::Instant::now();

    while context
        .lock()
        .and_then(|res| match res.hostname {
            Some(_) => Ok(()),
            _ => Err(PoisonError::new(res)),
        })
        .is_err()
        && (Instant::now() - start) <= timeout
    {
        match event_loop.poll(Duration::from_secs(0)) {
            Ok(_) => {}
            Err(x) => println!("{}", x),
        }
    }

    let result = match &context.lock().unwrap().hostname {
        Some(x) => Ok(x.clone()),
        None => Err(Error::NoHostsFound),
    };
    result
}

fn on_service_discovered(
    result: zeroconf::Result<ServiceDiscovery>,
    context: Option<Arc<dyn Any>>,
) {
    if let Ok(sd) = result {
        if let Some(ctx) = context {
            if let Some(m) = ctx.downcast_ref::<Arc<Mutex<Context>>>() {
                if let Ok(mut ctx) = m.lock() {
                    ctx.hostname = Some(sd.host_name().clone());
                }
            }
        }
    }
}

pub fn get_receiver() -> Result<String, Error> {
    get_hostname(ServiceType::new("roap", "tcp").unwrap())
}

#[cfg(test)]
mod test {
    use super::get_receiver;
    use crate::{avahi3::get_hostname, avahi_error::Error};
    use std::net::TcpStream;
    use zeroconf::ServiceType;

    #[test]
    fn get_receiver_may_return() {
        match get_receiver() {
            Ok(address) => assert!(TcpStream::connect((address, 23)).is_ok()),
            Err(e) => assert!(matches!(e, Error::NoHostsFound)),
        }
    }

    #[test]
    fn timeout() {
        println!("test start");
        // ServiceType::new("http", "tcp").unwrap();
        // ServiceType::new("roap", "tcp").unwrap();
        let sn = ServiceType::new("does_not_exit", "tcp").unwrap();
        match get_hostname(sn) {
            Err(Error::NoHostsFound) => {}
            _ => assert!(false),
        }
    }

    // #[test]
    // fn found_service() {
    //     println!("test start");
    //     // ServiceType::new("http", "tcp").unwrap();
    //     // ServiceType::new("roap", "tcp").unwrap();
    //     println!(
    //         "{:?}",
    //         get_hostname(ServiceType::new("googlecast", "tcp").unwrap())
    //     );
    // }
}
