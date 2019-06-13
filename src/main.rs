use actix::prelude::*;
use actix_rt::spawn;
use actix::dev::{MessageResponse, ResponseChannel};


#[derive(Debug)]
struct Ping(i32);
#[derive(Debug)]
struct Pong(i32);


impl Message for Ping {
    type Result = Result<i32, String>;
}

impl Message for Pong {
    type Result = Result<i32, String>;
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


impl<T> Handler<T> for Proxy 
where
    T: Message + Send + 'static,
    <T as Message>::Result: MessageResponse<Upstream, T> + Send,
    <T as Message>::Result: MessageResponse<Proxy, T>,
    Upstream: Handler<T>
{
    type Result = <T as Message>::Result;

    fn handle(&mut self, msg: T, _: &mut Context<Self>) -> Self::Result {
        self.addr.send(msg)
//        Err("Error".to_string())
    }
}


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
