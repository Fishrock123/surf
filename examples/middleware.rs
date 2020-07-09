use futures::future::BoxFuture;
use surf::middleware::{Middleware, Next};
use surf::{Client, Request, Response};

struct Printer;

impl Middleware for Printer {
    fn handle<'a>(
        &'a self,
        req: Request,
        client: Client,
        next: Next<'a>,
    ) -> BoxFuture<'a, Result<Response, http_types::Error>> {
        Box::pin(async move {
            println!("sending a request!");
            let res = next.run(req, client).await?;
            println!("request completed!");
            Ok(res)
        })
    }
}

#[async_std::main]
async fn main() -> Result<(), http_types::Error> {
    femme::start(log::LevelFilter::Info)?;

    let req = surf::get("https://httpbin.org/get");
    surf::client().middleware(Printer {}).send(req).await?;
    Ok(())
}
