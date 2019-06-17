#[macro_use]
extern crate actix_derive;
use actix::prelude::*;
use actix_rt::spawn;
use actix::dev::MessageResponse;


#[derive(Debug)]
struct Ping(i32);
#[derive(Debug)]
struct Pong(i32);


impl Message for Ping {
    type Result = Result<i32, ServiceError>;
}

impl Message for Pong {
    type Result = Result<i32, ServiceError>;
}

struct Upstream;

impl Actor for Upstream {
    type Context = Context<Self>;
}

impl Handler<Ping> for Upstream {
    type Result = <Ping as Message>::Result;

    fn handle(&mut self, msg: Ping, _: &mut Context<Self>) -> Self::Result {
        // dbg!(msg.0);
        Ok(42)
    }
}

impl Handler<Pong> for Upstream {
    type Result = <Pong as Message>::Result;

    fn handle(&mut self, msg: Pong, _: &mut Context<Self>) -> Self::Result {
        // dbg!(msg.0);
        Ok(32)
    }
}

struct Proxy {
    addr: Addr<Upstream>,
}

impl Actor for Proxy {
    type Context = Context<Self>;
}

trait Cacheable {
}

impl Cacheable for Ping {

}

#[derive(Debug, Message)]
enum CachedMessages {
    #[derive(Message)]
    Ping(i32),
    #[derive(Message)]
    Pong(i32),
}

struct CacheMessage(pub CachedMessages);

impl Message for CacheMessage {
    type Result = Result<i32, ServiceError>;
}


// impl Handler<CacheMessage> for Proxy

#[derive(Debug)]
enum ServiceError {
    Common
}

impl From<MailboxError> for ServiceError {
    fn from(error: MailboxError) -> Self {
        ServiceError::Common
    }
}

impl From<String> for ServiceError {
    fn from(error: MailboxError) -> Self {
        ServiceError::Common
    }
}


use futures::future::IntoFuture;
impl<T> Handler<T> for Proxy 
where
    T: Message + Send + 'static,
    <T as Message>::Result: IntoFuture + Send,
    Upstream: Handler<T>,
    <<T as Message>::Result as IntoFuture>::Error: From<MailboxError>,
{
    type Result = ResponseFuture<<<<T as Message>::Result as IntoFuture>::Future as Future>::Item, <<<T as Message>::Result as IntoFuture>::Future as Future>::Error>;

    fn handle(&mut self, msg: T, _: &mut Context<Self>) -> Self::Result {
        let res = self
            .addr
            .send(msg)
            .map_err(|e| e.into())
            .and_then(|res| res);
        Box::new(res)
    }
}


// impl<T> Handler<T> for Proxy 
// where
    // T: Message + Send + 'static,
    // // <T as Message>::Result: MessageResponse<Upstream, T> + Send,
    // <T as Message>::Result: MessageResponse<Proxy, T>,
    // Upstream: Handler<T>,
    // // <T as Message>::Result: std::convert::From<Request<Upstream, T>>,
// {
    // type Result = <T as Message>::Result;

    // fn handle(&mut self, msg: T, _: &mut Context<Self>) -> Self::Result {
        // self.addr.send(msg).unwrap()
// //        Err("Error".to_string())
    // }
// }


fn main() -> std::io::Result<()> {
    System::run(|| {
        // start new actor
        let addr = Upstream {}.start();
        let proxy = Proxy { addr: addr.clone() }.start();

        // send message and get future for result
        let res = proxy.send(Ping(10));

        // handle() returns tokio handle
        spawn(
            res.map(|res| {
                println!("RESULT: {:?}", res);

                // stop system and exit
                System::current().stop();
            })
            .map_err(|_| ()),
        );
    })
}
