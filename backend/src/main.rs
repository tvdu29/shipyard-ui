use std::{
    env, io,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use actix_web::{client::ClientBuilder, get, web, App, HttpResponse, HttpServer};
use itertools::Itertools;
use redis::{Client, Commands};
use shipyard::{Repos, Tags};

async fn req_list_images(
    page_size: usize,
    redis: MutexGuard<'_, Client>,
) -> Result<usize, anyhow::Error> {
    println!("req_list_images");
    let mut res_repos = Repos::default();
    let url = format!(
        "{}/_catalog",
        env::var("SHIPYARD_REGISTRY_URL").unwrap_or("https://docker.adotmob.com/v2".to_string())
    );
    let mut last = "";
    loop {
        let url_req = match (page_size == 0, last.is_empty()) {
            (true, false) => format!("{}?last={}", url, last),
            (true, true) => url.to_string(),
            (false, false) => format!("{}?n={}&last={}", url, page_size, last),
            (false, true) => format!("{}?n={}", url, page_size),
        };
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
                    if repos.repositories.len() < page_size {
                        let mut con = redis.get_connection().expect("Failed to get connection");
                        let _: () = con.del("catalog").expect("Failed to clear catalog");
                        for i in res_repos.repositories.iter() {
                            let _: () = con.sadd("catalog", i).expect("Failed to set catalog");
                        }
                        return Ok(res_repos.repositories.len());
                    }
                    res_repos.repositories.append(repos.repositories.as_mut());
                    last = match res_repos.repositories.last() {
                        None => return Ok(0),
                        Some(i) => i,
                    };
                }
            },
        }
    }
}

#[get("/refresh_catalog")]
async fn refresh_catalog(client: web::Data<Arc<Mutex<Client>>>) -> HttpResponse {
    match client.lock() {
        Ok(client) => match req_list_images(300, client).await {
            Ok(s) => HttpResponse::Ok().body(format!("refreshed catalog\nnb images: {}", s)),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/catalog/{page}")]
async fn list_images_page(
    web::Path(page): web::Path<usize>,
    client: web::Data<Arc<Mutex<Client>>>,
) -> HttpResponse {
    let page_size = {
        match env::var("SHIPYARD_REPO_PAGE_SIZE") {
            Ok(ps) => match ps.parse::<usize>() {
                Ok(ps) => ps,
                Err(_) => {
                    eprintln!("Error parsing page size\ndefaulting to 20...");
                    20
                }
            },
            Err(_) => {
                eprintln!("Error parsing page size\ndefaulting to 20...");
                20
            }
        }
    };
    match client.lock() {
        Ok(client) => match client.get_connection() {
            Ok(mut con) => match con.scard("catalog") {
                Ok(card) => {
                    let card: usize = card;
                    if page_size < 1 || page < 1 || card / page_size < page {
                        return HttpResponse::NotFound().body(format!(
                            "invalid page or page size\nmax page: {}\npage: {}",
                            card / page_size,
                            page
                        ));
                    };
                    match redis::cmd("SORT")
                        .arg("catalog")
                        .arg("alpha")
                        .arg("limit")
                        .arg(((card / page_size * (page - 1)) as usize).to_string())
                        .arg(page_size.to_string())
                        .query(&mut con)
                    {
                        Ok(vec) => {
                            let vec: Vec<String> = vec;
                            HttpResponse::Ok().body(vec.join(","))
                        }
                        Err(e) => HttpResponse::InternalServerError()
                            .body(format!("Failed to request page: {}", e)),
                    }
                }
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Failed to request catalog size: {}", e)),
            },
            Err(e) => HttpResponse::InternalServerError()
                .body(format!("Failed to get redis connection: {}", e)),
        },
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Failed to get redis client lock: {}", e)),
    }
}

#[get("/tags/{image}")]
async fn list_tags(web::Path(image): web::Path<String>) -> HttpResponse {
    let url = format!(
        "{}/{}/tags/list",
        env::var("SHIPYARD_REGISTRY_URL").unwrap_or("https://docker.adotmob.com/v2".to_string()),
        image
    );
    match ClientBuilder::new()
        .timeout(Duration::from_secs(60))
        .finish()
        .get(url)
        .send()
        .await
    {
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to request tags: {}", e))
        }
        Ok(mut tags) => match tags.json::<Tags>().await {
            Ok(tags) => HttpResponse::Ok().body(tags.tags.join(",")),
            Err(e) => {
                HttpResponse::InternalServerError().body(format!("Failed to parse tags: {}", e))
            }
        },
    }
}

#[get("/manifest/{image}")]
async fn get_manifest(web::Path(image): web::Path<String>) -> HttpResponse {
    let (image, tag) = match image.splitn(2, ":").collect_tuple() {
        None => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to parse image and/or tag"))
        }
        Some((img, tg)) => (img, tg),
    };
    let url = format!(
        "{}/{}/manifests/{}",
        env::var("SHIPYARD_REGISTRY_URL").unwrap_or("https://docker.adotmob.com/v2".to_string()),
        image,
        tag
    );
    match ClientBuilder::new()
        .timeout(Duration::from_secs(60))
        .finish()
        .get(url)
        .send()
        .await
    {
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to request manifest: {}", e))
        }
        Ok(mut manifest) => match manifest.body().await {
            Ok(tags) => HttpResponse::Ok().body(format!("{:#?}", std::str::from_utf8(&tags).expect("Failed to parse manifest").to_string())),
            Err(e) => {
                HttpResponse::InternalServerError().body(format!("Failed to parse tags: {}", e))
            }
        },
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let client = Arc::new(Mutex::new(
        redis::Client::open(
            env::var("SHIPYARD_REDIS_URL").unwrap_or("redis://127.0.0.1:6379/".to_string()),
        )
        .expect("Failed connecting redis"),
    ));
    println!("init catalog...");
    let lock = client
        .lock()
        .expect("Failed to lock redis client at startup");
    req_list_images(300, lock)
        .await
        .expect("Failed to fetch images at startup");
    println!("start api...");
    HttpServer::new(move || {
        App::new().app_data(web::Data::new(client.clone())).service(
            web::scope("/v2")
                .service(list_images_page)
                .service(refresh_catalog)
                .service(list_tags)
                .service(get_manifest),
        )
    })
    .bind(format!(
        "{}:{}",
        env::var("SHIPYARD_URL").unwrap_or("127.0.0.1".to_string()),
        env::var("SHIPYARD_PORT").unwrap_or("8080".to_string())
    ))?
    .run()
    .await
}
