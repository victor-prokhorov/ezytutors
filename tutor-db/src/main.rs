// aside for errors handling
use actix_web::{error::Error, web, App, HttpResponse, HttpServer};
use std::fs::File;

async fn hello() -> Result<HttpResponse, Error> {
    println!("hello");
    let _ = File::open("fictionalfile.txt")?;
    Ok(HttpResponse::Ok().body("file read successfully"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/hello", web::get().to(hello)))
        .bind("127.0.0.1:3000")?
        .run()
        .await
}
