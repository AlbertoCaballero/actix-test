use actix_web::{App, HttpResponse, HttpServer, Responder, get, guard, http::KeepAlive, post, web};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::{sync::Mutex, time::Duration};
use tokio;

struct AppState {
    app_name: String,
    app_dev: String,
}

struct AppStateCounter {
    counter: Mutex<i32>, //Mutex necessary to mutate safely across threads
}

#[get("/")]
async fn index(data: web::Data<AppStateCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // get counter MutexGuard
    *counter += 1; // access counter inside MutexGuard
    return format!("Request #{counter}");
}

#[get("/app-info")]
async fn app_info(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name;
    let app_dev = &data.app_dev;
    return format!("Welcome to {app_name}! By {app_dev}");
}

#[get("/guarded")]
async fn guarded() -> impl Responder {
    HttpResponse::Ok().body("On guard!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Manual hello!")
}

// any long, non-cpu-bound operation should be expressed as futures or asynchronous functions.
async fn async_function() -> impl Responder {
    tokio::time::sleep(Duration::from_secs(5)).await; // Worker thread will handle other request here
    "response"
}

// functions should be in different module
// Each ServiceConfig can have its own data, routes, and services.
fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/test")
            .route(web::get().to(|| async { HttpResponse::Ok().body("Scoped Configured") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app")
            .route(web::get().to(|| async { HttpResponse::Ok().body("Configured") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(AppStateCounter {
        counter: Mutex::new(0),
    });

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();
    // GENERATE CERT
    // $ openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem \
    // -days 365 -sha256 -subj "/C=CN/ST=Fujian/L=Xiamen/O=TVlinux/OU=Org/CN=muro.lxd"

    // let scope = web::scope("/guarded").service(guarded);

    return HttpServer::new(move || {
        App::new()
            .configure(config) // /test
            .service(web::scope("/api").configure(scoped_config)) // /api/test
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Test"),
                app_dev: String::from("AlbertoCaballero"),
            }))
            .app_data(counter.clone())
            .service(index)
            .service(app_info)
            .service(echo)
            // .service(scope)
            // .service(
            //     web::scope("/guarded")
            //         .guard(guard::Host("www.rust-lang.org"))
            //         .route("/", web::to(|| async { HttpResponse::Ok().body("www") })),
            // )
            // .route("/", web::to(HttpResponse::Ok))
            .route("/hey", web::get().to(manual_hello))
    })
    .workers(4) // Multi-Threading, by default number of CPUs in device
    .bind(("127.0.0.1", 5050))?
    // .bind_openssl("127.0.0.1:5050", builder)?
    // .keep_alive(Duration::from_secs(60)) // 60 seconds of keep alive conections
    // .keep_alive(None) // Don't keep alive conections
    .keep_alive(KeepAlive::Os) // Use OS configuration
    .run()
    .await;
}
