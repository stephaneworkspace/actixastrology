extern crate base64;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
use actix_web::{middleware, get, web, App, HttpResponse, HttpServer, Responder, Result}; 
use astrology::svg_draw::{ DataChartNatalC, DataObjectSvg, DataObjectType};
use base64::encode;
use serde::{Deserialize, Serialize}; 
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

struct AppState {
    _foo: String,
}
/*
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn index2() -> impl Responder {
    HttpResponse::Ok().body("Hello world again!")
}
*/

#[get("/api/hello")]
async fn index3() -> impl Responder {
    const PATH: &str = "data.json";
    let mut s = String::new();
    let mut file_path = PathBuf::new();
    file_path.push(env::current_dir().unwrap().as_path());
    file_path.push(PATH);
    File::open(file_path.as_path())
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    let data: astrology::Data = serde_json::from_str(&s).unwrap();
    let path_str: String = format!("{}/swisseph-for-astrology-crate/", env::current_dir().unwrap().as_path().display().to_string());
    println!("{}", path_str);
    let d = DataChartNatalC {
        year: data.year,
        month: data.month,
        day: data.day,
        hourf32: data.hourf32,
        hour: data.hour,
        min: data.min,
        sec: data.sec,
        lat: data.lat,
        lng: data.lng,
    };
    let res: Vec<DataObjectSvg> = astrology::svg_draw::chart(1000.0, d, path_str.as_str());
    let mut svg_res: String = "".to_string();
    for r in res.clone() {
        if r.object_type == DataObjectType::Chart {
            svg_res = r.svg;
        }
    }
    if svg_res != "" {
        svg_res = svg_res.replace("</svg>", "");
        for r in res {
            if r.object_type != DataObjectType::Chart {
                // to do better inside after for real use
                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()));
            }
        }
    } else {
        svg_res = "<svg>".to_string();
    }
    svg_res = format!("{}</svg>", svg_res);
 
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(svg_res)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(app_config)
            //.service(index3)
            //.route("/", web::get().to(index))
            //.route("/again", web::get().to(index2))
    })
    .bind("0.0.0.0:8088")?
    .run()
    .await
}

fn app_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("")
            .data(AppState {
                _foo: "bar".to_string(),
            })
            .service(web::resource("/").route(web::get().to(index)))
            .service(index3)
            .service(web::resource("/natal_chart").route(web::post().to(handle_post_natal_chart))),
        );
}

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/form.html")))
}

async fn handle_post_natal_chart(params: web::Form<MyParams>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("Your name is {}", params.name)))
}

#[derive(Serialize, Deserialize)]
pub struct MyParams {
    name: String,
}
