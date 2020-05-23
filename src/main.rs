extern crate base64;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
use actix_cors::Cors;
use actix_web::{middleware, get, web, App, HttpResponse, HttpServer, Responder, Result}; 
use astrology::svg_draw::{DataChartNatal, DataObjectSvg, DataObjectType, DataObjectAspectSvg};
use libswe_sys::sweconst::Language;
use base64::encode;
use serde::{Deserialize, Serialize}; 
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;

struct AppState {
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    min: i32,
}

/// Natal svg
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
        lat: a_data.lat,
        lng: a_data.lng,
    };
    let res: Vec<DataObjectSvg> = astrology::svg_draw::chart(1000.0, d, path_str.as_str(), Language::French);
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
                min: 0
            }));
    config.service(
        web::scope("")
            .app_data(data.clone())
            .service(web::resource("/api/").route(web::get().to(index)))
            .service(index3)
            .service(web::resource("/api/natal_chart").route(web::post().to(handle_post_natal_chart)))
            .service(web::resource("/api/svg_chart").route(web::post().to(handle_post_natal_chart_svg)))
            .service(web::resource("/api/svg_chart_transit").route(web::post().to(handle_post_natal_chart_svg_transit)))
            .service(all_aspects),
        );
}

/// Form
async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/form.html")))
}

/// Handle form
async fn handle_post_natal_chart(params: web::Form<MyParams>, data: web::Data<Mutex<AppState>>) -> Result<HttpResponse> {
    let mut data = data.lock().unwrap();
    data.year = params.year;
    data.month = params.month;
    data.day = params.day;
    data.hour = params.hour;
    data.min = params.min;
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
        lat: params.lat,
        lng: params.lng,
    };
    let res: Vec<DataObjectSvg> = astrology::svg_draw::chart(600.0, d, path_str.as_str(), Language::French);
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
                if r.object_type == DataObjectType::Aspect {
                    match params.aspect {
                        0 => {
                        },
                        1 => {
                            let mut sw_res = false;
                            // Major
                            for a in r.aspects {
                                if a.to_string() == "Conjunction".to_string() || a.to_string() == "Opposition".to_string() || a.to_string() == "Trine".to_string() || a.to_string() == "Square".to_string() || a.to_string() == "Sextile".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        2 => {
                            let mut sw_res = false;
                            // Conjunction
                            for a in r.aspects {
                                if a.to_string() == "Conjunction".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        3 => {
                            let mut sw_res = false;
                            // Opposition
                            for a in r.aspects {
                                if a.to_string() == "Opposition".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        4 => {
                            let mut sw_res = false;
                            // Trigone
                            for a in r.aspects {
                                if a.to_string() == "Trine".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        5 => {
                            let mut sw_res = false;
                            // Square
                            for a in r.aspects {
                                if a.to_string() == "Square".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        6 => {
                            let mut sw_res = false;
                            // Sextile
                            for a in r.aspects {
                                if a.to_string() == "Sextile".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        7 => {
                            let mut sw_res = false;
                            // Minor
                            for a in r.aspects {
                                if a.to_string() == "Inconjunction".to_string() || a.to_string() == "Sesquisquare".to_string() || a.to_string() == "Semisquare".to_string() || a.to_string() == "Semisextile".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        8 => {
                            let mut sw_res = false;
                            // Inconjunction
                            for a in r.aspects {
                                if a.to_string() == "Inconjunction".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        9 => {
                            let mut sw_res = false;
                            // Sesquisquare
                            for a in r.aspects {
                                if a.to_string() == "Sesquisquare".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        10 => {
                            let mut sw_res = false;
                            // Semisquare
                            for a in r.aspects {
                                if a.to_string() == "Semisquare".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        11 => {
                            let mut sw_res = false;
                            // Semisextile
                            for a in r.aspects {
                                if a.to_string() == "Semisextile".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        12 => svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str())),
                        _ => svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str())),
                    }
                 } else {
                    // to do better inside after for real use
                    svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()));
                }
            } else {
                // to do better inside after for real use
                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()));
            }
        }
    } else {
        svg_res = "<svg>".to_string();
    }
    svg_res = format!("{}</svg>", svg_res);
 
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
        lat: params.lat_t,
        lng: params.lng_t,
    };
    let res: Vec<DataObjectSvg> = astrology::svg_draw::chart_with_transit(600.0, d, d_t, path_str.as_str(), Language::French);
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
                if r.object_type == DataObjectType::Aspect {
                    match params.aspect {
                        0 => {
                        },
                        1 => {
                            let mut sw_res = false;
                            // Major
                            for a in r.aspects {
                                if a.to_string() == "Conjunction".to_string() || a.to_string() == "Opposition".to_string() || a.to_string() == "Trine".to_string() || a.to_string() == "Square".to_string() || a.to_string() == "Sextile".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        2 => {
                            let mut sw_res = false;
                            // Conjunction
                            for a in r.aspects {
                                if a.to_string() == "Conjunction".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        3 => {
                            let mut sw_res = false;
                            // Opposition
                            for a in r.aspects {
                                if a.to_string() == "Opposition".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        4 => {
                            let mut sw_res = false;
                            // Trigone
                            for a in r.aspects {
                                if a.to_string() == "Trine".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        5 => {
                            let mut sw_res = false;
                            // Square
                            for a in r.aspects {
                                if a.to_string() == "Square".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        6 => {
                            let mut sw_res = false;
                            // Sextile
                            for a in r.aspects {
                                if a.to_string() == "Sextile".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        7 => {
                            let mut sw_res = false;
                            // Minor
                            for a in r.aspects {
                                if a.to_string() == "Inconjunction".to_string() || a.to_string() == "Sesquisquare".to_string() || a.to_string() == "Semisquare".to_string() || a.to_string() == "Semisextile".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        8 => {
                            let mut sw_res = false;
                            // Inconjunction
                            for a in r.aspects {
                                if a.to_string() == "Inconjunction".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        9 => {
                            let mut sw_res = false;
                            // Sesquisquare
                            for a in r.aspects {
                                if a.to_string() == "Sesquisquare".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        10 => {
                            let mut sw_res = false;
                            // Semisquare
                            for a in r.aspects {
                                if a.to_string() == "Semisquare".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        11 => {
                            let mut sw_res = false;
                            // Semisextile
                            for a in r.aspects {
                                if a.to_string() == "Semisextile".to_string() {
                                    sw_res = true;
                                }
                            }
                            if sw_res {
                                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()))
                            }
                        },
                        12 => svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str())),
                        _ => svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str())),
                    }
                 } else {
                    // to do better inside after for real use
                    svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()));
                }
            } else {
                // to do better inside after for real use
                svg_res = format!("{}<image width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\" href=\"data:image/svg+xml;base64,{}\"/>", svg_res, r.size_x, r.size_y, r.pos_x, r.pos_y, encode(r.svg.as_str()));
            }
        }
    } else {
        svg_res = "<svg>".to_string();
    }
    svg_res = format!("{}</svg>", svg_res);
 
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
    month: i32,
    day: i32,
    hour: i32,
    min: i32,
}

/// NewForm for js/ts front
#[derive(Serialize, Deserialize)]
pub struct MyNatalParams {
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    min: i32,
    lat: f32,
    lng: f32,
    aspect: i32,
}
/// NewForm for js/ts front
#[derive(Serialize, Deserialize)]
pub struct MyTransitParams {
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    min: i32,
    lat: f32,
    lng: f32,
    year_t: i32,
    month_t: i32,
    day_t: i32,
    hour_t: i32,
    min_t: i32,
    lat_t: f32,
    lng_t: f32,
    aspect: i32,
}
