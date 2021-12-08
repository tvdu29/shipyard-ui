use actix_web::{
    client::Client,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use cached::proc_macro::cached;

#[cached]
async fn list_images() -> impl Responder {
    let mut client = Client::default();

    let response = client
        .get("https://docker.adotmob.com/v2/_catalog")
        .send()
        .await
        .expect("Failed to request catalog");
    HttpResponse::Ok().body(response)
}

async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new().service(hello).service(echo).service(
            web::scope("/api")
                .route("/files/", web::post().to(upload))
                .route("/files/{names}/", web::get().to(download)),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
