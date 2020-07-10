use futures::future::BoxFuture;
use futures::io::AsyncReadExt;
use surf::middleware::{Middleware, Next};
use surf::{Body, Client, Request, Response};

struct Doubler;

impl Middleware for Doubler {
    fn handle<'a>(
        &'a self,
        req: Request,
        client: Client,
        next: Next<'a>,
    ) -> BoxFuture<'a, Result<Response, http_types::Error>> {
        if req.method().is_safe() {
            Box::pin(async move {
                let mut new_req = http_types::Request::new(req.method(), req.url().clone());
                new_req.set_version(req.as_ref().version());
                let mut new_req: Request = new_req.into();

                for (name, value) in &req {
                    new_req.insert_header(name, value);
                }

                let mut buf = Vec::new();
                let (res1, res2) =
                    futures::future::join(next.run(req, client.clone()), next.run(new_req, client))
                        .await;

                let mut res = res1?;
                res.read_to_end(&mut buf).await?;

                let mut res = res2?;
                res.read_to_end(&mut buf).await?;
                res.set_body(Body::from(buf));
                Ok(res)
            })
        } else {
            next.run(req, client)
        }
    }
}

#[async_std::main]
async fn main() -> Result<(), http_types::Error> {
    femme::start(log::LevelFilter::Info)?;

    let mut res = surf::get("https://httpbin.org/get")
        .middleware(Doubler {})
        .await?;
    dbg!(&res);
    let body = res.body_bytes().await?;
    let body = String::from_utf8_lossy(&body);
    println!("{}", body);
    Ok(())
}
