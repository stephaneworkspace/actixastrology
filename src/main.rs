extern crate htmlescape;
extern crate base64;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate filter_city;
extern crate city_time_zone_sqlite;
use actix_cors::Cors;
use actix_web::{middleware, get, web, App, HttpResponse, HttpServer, Responder, Result}; 
use astrology::svg_draw::{DataChartNatal, DataObjectAspectSvg};
use libswe_sys::sweconst::{Language, AspectsFilter};
use serde::{Deserialize, Serialize}; 
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;
use city_time_zone_sqlite::{Repo, TraitRepoUtils, TraitRepoD01, TraitRepoD03, AppError};
use num_traits::FromPrimitive;

pub struct AppState {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    time_zone: f32,
}

/// Generate natal svg
#[get("/api/svg/natal.svg")]
async fn index3(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let data = data.lock().unwrap();
    const PATH: &str = "data.json";
    let mut s = String::new();
    let mut file_path = PathBuf::new();
    file_path.push(env::current_dir().unwrap().as_path());
    file_path.push(PATH);
    File::open(file_path.as_path())
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    let a_data: DataChartNatal= serde_json::from_str(&s).unwrap();
    let path_str: String = format!("{}/swisseph-for-astrology-crate/", env::current_dir().unwrap().as_path().display().to_string());
    println!("{}", path_str);
    let d = DataChartNatal {
        year: data.year,
        month: data.month,
        day: data.day,
        hour: data.hour,
        min: data.min,
        sec: a_data.sec,
        time_zone: data.time_zone,
        lat: a_data.lat,
        lng: a_data.lng,
    };
    let svg_res = astrology::svg_draw::chart_svg(1000.0 as f32, d, path_str.as_str(), Language::French, AspectsFilter::AllAspects);
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(svg_res)
}

/// Main
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Log
    std::env::set_var("RUST_LOG", "actix_web=info");
    // env_logger::init(); Not compile, but interessant skills

    // Server
    HttpServer::new(|| {
        App::new()
            .wrap(
                // Cors::new().support_credentials().finish()) Interessant
                Cors::new().finish())
            .wrap(middleware::Logger::default())
            .configure(app_config)
            //.service(index3)
            //.route("/", web::get().to(index))
            //.route("/again", web::get().to(index2))
    })
    .bind("92.222.64.94:8088")?
    //.bind("0.0.0.0:8088")?
    .run()
    .await
}

/// Configuration
fn app_config(config: &mut web::ServiceConfig) {
    let data = web::Data::new(Mutex::new(AppState {
                year: 2000,
                month: 1,
                day: 1,
                hour: 0,
                min: 0,
                time_zone: 2.0,
            }));
    config.service(
        web::scope("")
            .app_data(data.clone())
            .service(web::resource("/api/").route(web::get().to(index)))
            .service(index3)
            .service(web::resource("/api/natal_chart").route(web::post().to(handle_post_natal_chart)))
            .service(web::resource("/api/svg_chart").route(web::post().to(handle_post_natal_chart_svg)))
            .service(web::resource("/api/svg_chart_transit").route(web::post().to(handle_post_natal_chart_svg_transit)))
            .service(all_aspects)
            .service(web::resource("/api/filter-city/{name}").route(web::get().to(handle_post_filter_city)))
            .service(web::resource("/api/filter-city-2/{name}").route(web::get().to(handle_post_filter_city_2)))
            .service(web::resource("/api/filter-city-time-zone").route(web::get().to(handle_post_filter_city_time_zone))),
        );
}

/// Form
async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/form.html")))
}

/// Handle filter-city
async fn handle_post_filter_city(obj: web::Path<MyParamsFilterCity>) -> Result<HttpResponse> {
    let search : Vec<filter_city::City> = filter_city::filter_city(obj.name.as_str());
    let data = serde_json::to_string(&search).unwrap();
 
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(data))
}

/// Handle filter-city with sqlite for query a city
async fn handle_post_filter_city_2(obj: web::Path<MyParamsFilterCity>) -> Result<HttpResponse> {
    let status = Repo::connect();
    let repo = match status {
        Ok(res) => res,
        Err(AppError { err_type, message }) => {
            panic!("{:?} {}", err_type, message)
        }
    };
    println!("{}",&obj.name.as_str()); 
    let status = repo.d01_search_compact(&obj.name.as_str());
    let recs = match status {
        Ok(res) => res,
        Err(AppError { err_type, message }) => {
            panic!("{:?} {}", err_type, message)
        }
    };
   
    let data = serde_json::to_string(&recs).unwrap();
 
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(data))
}

/// Handle filter-city with sqlite for timezone find all
async fn handle_post_filter_city_time_zone() -> Result<HttpResponse> {
    let status = Repo::connect();
    let repo = match status {
        Ok(res) => res,
        Err(AppError { err_type, message }) => {
            panic!("{:?} {}", err_type, message)
        }
    };
    let status = repo.d03_find_all_compact();
    let recs = match status {
        Ok(res) => res,
        Err(AppError { err_type, message }) => {
            panic!("{:?} {}", err_type, message)
        }
    };
   
    let data = serde_json::to_string(&recs).unwrap();
 
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(data))
}

/// Handle form
async fn handle_post_natal_chart(params: web::Form<MyParams>, data: web::Data<Mutex<AppState>>) -> Result<HttpResponse> {
    let mut data = data.lock().unwrap();
    data.year = params.year;
    data.month = params.month;
    data.day = params.day;
    data.hour = params.hour;
    data.min = params.min;
    data.time_zone = params.time_zone;
    let svg = "<img src=\"svg/natal.svg\"/>";
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!("<html>Your year is {}{}<br />{}</html>", params.year, data.year, svg)))
}

/// Svg only
async fn handle_post_natal_chart_svg(params: web::Form<MyNatalParams>, _data: web::Data<Mutex<AppState>>) -> Result<HttpResponse> {
    let path_str: String = format!("{}/swisseph-for-astrology-crate/", env::current_dir().unwrap().as_path().display().to_string());
    println!("{}", path_str);
    let d = DataChartNatal {
        year: params.year,
        month: params.month,
        day: params.day,
        hour: params.hour,
        min: params.min,
        sec: 0.0,
        time_zone: params.time_zone,
        lat: params.lat,
        lng: params.lng,
    };
    let aspect: AspectsFilter = match FromPrimitive::from_i32(params.aspect) {
        Some(a) => a,
        None => AspectsFilter::NoAspects,
    };
    let svg_res = astrology::svg_draw::chart_svg(1000.0 as f32, d, path_str.as_str(), Language::French, aspect);
    Ok(HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(svg_res))
}
/// Svg only transit
async fn handle_post_natal_chart_svg_transit(params: web::Form<MyTransitParams>, _data: web::Data<Mutex<AppState>>) -> Result<HttpResponse> {
    let path_str: String = format!("{}/swisseph-for-astrology-crate/", env::current_dir().unwrap().as_path().display().to_string());
    println!("{}", path_str);
    let d = DataChartNatal {
        year: params.year,
        month: params.month,
        day: params.day,
        hour: params.hour,
        min: params.min,
        sec: 0.0,
        time_zone: params.time_zone,
        lat: params.lat,
        lng: params.lng,
    };
    let d_t = DataChartNatal {
        year: params.year_t,
        month: params.month_t,
        day: params.day_t,
        hour: params.hour_t,
        min: params.min_t,
        sec: 0.0,
        time_zone: params.time_zone_t,
        lat: params.lat_t,
        lng: params.lng_t,
    };
    let aspect: AspectsFilter = match FromPrimitive::from_i32(params.aspect) {
        Some(a) => a,
        None => AspectsFilter::NoAspects,
    };
    let svg_res = astrology::svg_draw::chart_svg_with_transit(1000.0, d, d_t, path_str.as_str(), Language::French, aspect);
    Ok(HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(svg_res))
}

/// Aspect svg
#[get("/api/aspects.json")]
async fn all_aspects(_data: web::Data<Mutex<AppState>>) -> impl Responder {
    let res: Vec<DataObjectAspectSvg> = astrology::svg_draw::all_aspects(Language::French);
    let data = serde_json::to_string(&res).unwrap();
 
    HttpResponse::Ok()
        .content_type("application/json")
        .body(data)
}

/// Form params
#[derive(Serialize, Deserialize)]
pub struct MyParams {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    time_zone: f32,
}


/// NewForm for js/ts front
#[derive(Serialize, Deserialize)]
pub struct MyNatalParams {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    time_zone: f32,
    lat: f32,
    lng: f32,
    aspect: i32,
}

/// NewForm for js/ts front
#[derive(Serialize, Deserialize)]
pub struct MyTransitParams {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    time_zone: f32,
    lat: f32,
    lng: f32,
    year_t: i32,
    month_t: u32,
    day_t: u32,
    hour_t: u32,
    min_t: u32,
    time_zone_t: f32,
    lat_t: f32,
    lng_t: f32,
    aspect: i32,
}

/// Filter city
#[derive(Serialize, Deserialize)]
pub struct MyParamsFilterCity {
    name: String
}
