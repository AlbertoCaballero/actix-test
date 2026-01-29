use actix_web::{App, HttpResponse, HttpServer, Responder, get, guard, post, web};
use std::sync::Mutex;

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
    .bind(("127.0.0.1", 5050))?
    .run()
    .await;
}
