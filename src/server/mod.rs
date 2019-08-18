use std::fs;
use std::io::Write;
use std::net::Ipv4Addr;

use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{error, guard, middleware, web, App, Error, HttpResponse, HttpServer};
use futures::future::{err, Either};
use futures::{Future, Stream};

pub fn save_file(field: Field) -> impl Future<Item = i64, Error = Error> {
    let file_path_string = match field.content_disposition().unwrap().get_filename() {
        Some(filename) => format!("uploads/{}", filename.replace(' ', "_").to_string()),
        None => {
            return Either::A(err(error::ErrorInternalServerError(
                "Couldn't read the filename.",
            )))
        }
    };

    let file = match fs::File::create(file_path_string) {
        Ok(file) => file,
        Err(e) => return Either::A(err(error::ErrorInternalServerError(e))),
    };
    Either::B(
        field
            .fold((file, 0i64), move |(mut file, mut acc), bytes| {
                // fs operations are blocking, we have to execute writes
                // on threadpool
                web::block(move || {
                    file.write_all(bytes.as_ref()).map_err(|e| {
                        println!("file.write_all failed: {:?}", e);
                        MultipartError::Payload(error::PayloadError::Io(e))
                    })?;
                    acc += bytes.len() as i64;
                    Ok((file, acc))
                })
                .map_err(|e: error::BlockingError<MultipartError>| match e {
                    error::BlockingError::Error(e) => e,
                    error::BlockingError::Canceled => MultipartError::Incomplete,
                })
            })
            .map(|(_, acc)| acc)
            .map_err(|e| {
                println!("save_file failed, {:?}", e);
                error::ErrorInternalServerError(e)
            }),
    )
}

pub fn upload(multipart: Multipart) -> impl Future<Item = HttpResponse, Error = Error> {
    multipart
        .map_err(error::ErrorInternalServerError)
        .map(|field| save_file(field).into_stream())
        .flatten()
        .collect()
        .map(|sizes| HttpResponse::Ok().json(sizes))
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
}

fn p404() -> HttpResponse {
    let html = r#"<html>
        <head><title>Not Found</title></head>
        <body>
            <h1>Has somebody seen my resource? I don't find it anywhere!</h1>
        </body>
    </html>"#;
    HttpResponse::NotFound().body(html)
}

pub fn start(ip: Ipv4Addr, port: String) {
    let address = format!("{}:{}", ip, port);
    let server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            // .service(
            //     web::resource("")
            //         .route(web::get().to(upload_front_end))
            //         .route(web::post().to_async(upload)),
            // )
            .service(web::resource("/upload").route(web::post().to_async(upload)))
            .service(actix_files::Files::new("/uploads", "./uploads/").show_files_listing())
            .service(actix_files::Files::new("/", "./static/").index_file("index.html"))
            .default_service(
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET`
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    ),
            )
    })
    .bind(&address)
    .unwrap();

    println!("NAS bound to address: {}", address);
    server.run().unwrap();
}
