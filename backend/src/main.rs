use std::{sync::{Mutex, Arc}, time::Duration};

use actix_web::{
    client:: ClientBuilder,
    web::{self} , App, HttpResponse, HttpServer
};
use shipyard::Repos;

async fn req_list_images(
    page_size: usize,
) -> Result<Repos, anyhow::Error> {
    println!("req_list_images");
    let mut res_repos = Repos::default();
    let url = "https://docker.adotmob.com/v2/_catalog";
    let mut last = "";
    loop {
        let url_req = match (page_size == 0, last.is_empty()) {
            (true, false) => format!("{}?last={}", url, last),
            (true, true) => url.to_string(),
            (false, false) => format!("{}?n={}&last={}", url, page_size, last),
            (false, true) => format!("{}?n={}", url, page_size),
        };
        println!("DEBUG url {:#?}", url_req);
        match ClientBuilder::new()
            .timeout(Duration::from_secs(60))
            .finish()
            .get(url_req)
            .send()
            .await
        {
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "Failed to request docker directory: {}",
                    e
                )))
            }
            Ok(mut res) => match res.json::<Repos>().await {
                Err(e) => {
                    return Err(anyhow::Error::msg(format!(
                        "Failed to parse docker directory response: {}",
                        e
                    )))
                }
                Ok(mut repos) => {
                    if repos.repositories.is_empty() {
                        return Ok(res_repos);
                    }
                    res_repos.repositories.append(repos.repositories.as_mut());
                    last = match res_repos.repositories.last() {
                        None => return Ok(Repos::default()),
                        Some(i) => i,
                    };
                }
            },
        }
    }
}

async fn refresh_catalog(data: web::Data<Arc<Mutex<Repos>>>) -> HttpResponse {
    match (data.lock(), req_list_images(300).await) {
        (Ok(mut data), Ok(new_data)) => {data.repositories = new_data.repositories.to_vec(); HttpResponse::Ok().body(format!("catalog updated: {} repositories", data.repositories.len()))},
        (Err(e), _) => HttpResponse::InternalServerError().body(e.to_string()),
        (_, Err(e)) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn list_images(data: web::Data<Arc<Mutex<Repos>>>) -> HttpResponse {
    match data.lock() {
        Ok(response) => HttpResponse::Ok().body(response.repositories.join(",")),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = Arc::new(Mutex::new(req_list_images(300).await.expect("Failed requesting docker registry")));
    HttpServer::new(move || {
        App::new().data(web::Data::new(data.clone())).service(
            web::scope("/v2")
                .route("/catalog", web::get().to(list_images))
                .route("/refresh_catalog", web::get().to(refresh_catalog)),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
