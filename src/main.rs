use actix_web::{web, App, HttpServer, HttpResponse, Responder, HttpRequest};
use askama::Template;
use dotenv::dotenv;
use services::aggregator::fetch_combined_news;
use serde::{Deserialize, Serialize};

mod api;
mod models;
mod services;

use models::news::NewsItem;

use actix_session::{SessionMiddleware, storage::CookieSessionStore, Session};
use actix_web::cookie::Key;
use reqwest::Client;

#[derive(Template)]
#[template(path = "index.html")]
struct NewsTemplate {
    symbol: String,
    news: Vec<NewsItem>,
    error: String,
    username: Option<String>,
}


#[derive(Deserialize, Serialize)]
struct AuthForm {
    username: String,
    password: String,
}

#[derive(Deserialize, Serialize)]
struct ApiKeyForm {
    api_key: String,
}

async fn show_login() -> impl Responder {
    HttpResponse::Ok().body(r#"
        <h1>Login</h1>
        <form method="POST" action="/login">
            Username: <input name="username"><br>
            Password: <input name="password" type="password"><br>
            <button type="submit">Login</button>
        </form>
    "#)
}

async fn show_register() -> impl Responder {
    HttpResponse::Ok().body(r#"
        <h1>Register</h1>
        <form method="POST" action="/register">
            Username: <input name="username"><br>
            Password: <input name="password" type="password"><br>
            <button type="submit">Register</button>
        </form>
    "#)
}

async fn do_login(form: web::Form<AuthForm>, session: Session) -> impl Responder {
    let client = Client::new();
    let backend_url = "http://localhost:5000/api/login";

    let res = client.post(backend_url)
        .json(&*form)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let json: serde_json::Value = response.json().await.unwrap();
                let token = json["token"].as_str().unwrap().to_string();

                session.insert("token", token).unwrap();
                HttpResponse::Found().append_header(("Location", "/account")).finish()
            } else {
                HttpResponse::Ok().body("\u{274c} Invalid login")
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn do_register(form: web::Form<AuthForm>) -> impl Responder {
    let client = Client::new();
    let backend_url = "http://localhost:5000/api/register";

    let res = client.post(backend_url)
        .json(&*form)
        .send()
        .await;

    if let Ok(response) = res {
        if response.status().is_success() {
            HttpResponse::Found().append_header(("Location", "/login")).finish()
        } else {
            HttpResponse::Ok().body("\u{274c} Registration failed")
        }
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Found().append_header(("Location", "/")).finish()
}

async fn show_account(session: Session) -> impl Responder {
    if let Ok(Some(token)) = session.get::<String>("token") {
        let client = Client::new();
        let res = client.get("http://localhost:5000/api/me")
            .bearer_auth(&token)
            .send()
            .await;

        if let Ok(response) = res {
            if response.status().is_success() {
                let json: serde_json::Value = response.json().await.unwrap();
                let username = json["username"].as_str().unwrap_or("").to_string();
                let api_key = json["apiKey"].as_str().unwrap_or("").to_string();

                return HttpResponse::Ok().body(format!(r#"
                    <h1>Account</h1>
                    <p>Logged in as: <strong>{}</strong></p>
                    <form method="POST" action="/account">
                        API Key: <input name="api_key" value="{}"><br>
                        <button type="submit">Update API Key</button>
                    </form>
                    <p><a href="/logout">Logout</a></p>
                "#, username, api_key));
            }
        }
    }

    HttpResponse::Found().append_header(("Location", "/login")).finish()
}

async fn update_api_key(form: web::Form<ApiKeyForm>, session: Session) -> impl Responder {
    println!("🔑 update_api_key called!");

    match session.get::<String>("token") {
        Ok(Some(token)) => {
            println!("✅ Session token: {}", token);

            let client = Client::new();
            let res = client
                .put("http://localhost:5000/api/api-key")
                .bearer_auth(&token)
                .json(&*form)
                .send()
                .await;

            if res.is_ok() {
                return HttpResponse::Found().append_header(("Location", "/account")).finish();
            } else {
                println!("❌ Failed to update API key");
            }
        }
        _ => println!("🚫 No session token!"),
    }

    HttpResponse::Found().append_header(("Location", "/login")).finish()
}


async fn search(
    session: Session,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    if session.get::<String>("token").unwrap_or(None).is_none() {
        return HttpResponse::Found().append_header(("Location", "/login")).finish();
    }

    let symbol = query.get("symbol").cloned().unwrap_or_default();

    if symbol.is_empty() {
        return HttpResponse::Ok().body("Please provide a symbol, e.g., /search?symbol=BTC");
    }

    let username = get_username(&session).await; // <- this is Option<String>

    match fetch_combined_news(&symbol).await {
        Ok(news) => {
            let tmpl = NewsTemplate {
                symbol,
                news,
                error: "".to_string(),
                username,
            };
            HttpResponse::Ok().content_type("text/html").body(tmpl.render().unwrap())
        }
        Err(err) => {
            let tmpl = NewsTemplate {
                symbol,
                news: vec![],
                error: err,
                username,
            };
            HttpResponse::Ok().content_type("text/html").body(tmpl.render().unwrap())
        }
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("Server running at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
        .wrap(SessionMiddleware::new(
            CookieSessionStore::default(),
            Key::from("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".as_bytes()),
        ))
        
            .route("/", web::get().to(|| async {
                HttpResponse::Ok().body("Welcome! Use /search?symbol=BTC")
            }))
            .route("/search", web::get().to(search))
            .route("/login", web::get().to(show_login))
            .route("/login", web::post().to(do_login))
            .route("/register", web::get().to(show_register))
            .route("/register", web::post().to(do_register))
            .route("/logout", web::get().to(logout))
            .route("/account", web::get().to(show_account))
            .route("/account", web::post().to(update_api_key))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn get_username(session: &Session) -> Option<String> {
    if let Ok(Some(token)) = session.get::<String>("token") {
        let client = Client::new();
        if let Ok(res) = client.get("http://localhost:5000/api/me")
            .bearer_auth(token)
            .send()
            .await
        {
            if let Ok(json) = res.json::<serde_json::Value>().await {
                return json["username"].as_str().map(|s| s.to_string());
            }
        }
    }
    None
}
