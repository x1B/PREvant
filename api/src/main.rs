/*-
 * ========================LICENSE_START=================================
 * PREvant
 * %%
 * Copyright (C) 2018 aixigo AG
 * %%
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 * =========================LICENSE_END==================================
 */
#![feature(proc_macro_hygiene, decl_macro, try_from)]

extern crate crossbeam_utils;
extern crate dkregistry;
extern crate goji;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate multimap;
extern crate regex;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate shiplift;
extern crate tokio;
extern crate tokio_core;
extern crate toml;
extern crate url;

use std::fs::File;

use rocket_contrib::json::Json;
use serde_yaml::{from_reader, to_string, Value};
use shiplift::{ContainerListOptions, Docker};

use models::request_info::RequestInfo;

mod apps;
mod commands;
mod models;
mod service_images;
mod services;
mod webhooks;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppsStatus {
    root_url: String,
    swagger_ui_available: bool,
    portainer_available: bool,
}

fn is_container_available(container_image_pattern: &str) -> bool {
    let docker = Docker::new();
    let containers = docker.containers();

    let running_containers = match containers.list(&ContainerListOptions::builder().build()) {
        Err(e) => {
            error!("Cannot list running containers: {}", e);
            return false;
        }
        Ok(containers) => containers,
    };

    running_containers
        .iter()
        .any(|c| c.image.starts_with(container_image_pattern))
}

#[get("/", format = "application/json")]
fn index(request_info: RequestInfo) -> Json<AppsStatus> {
    Json(AppsStatus {
        root_url: request_info.get_base_url().clone().into_string(),
        swagger_ui_available: is_container_available("swaggerapi/swagger-ui"),
        portainer_available: is_container_available("portainer/portainer"),
    })
}

#[get("/swagger.yaml")]
fn swagger(request_info: RequestInfo) -> String {
    let mut f = File::open("swagger.yaml").unwrap();

    let mut v: Value = from_reader(&mut f).unwrap();

    let mut url = request_info.get_base_url().clone();
    url.set_path("/api");
    v["servers"][0]["url"] = Value::String(String::from(url.to_string()));

    to_string(&v).unwrap()
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/", routes![swagger])
        .mount("/", routes![apps::apps])
        .mount("/", routes![apps::tickets])
        .mount("/", routes![apps::create_app])
        .mount("/", routes![apps::delete_app])
        .mount("/", routes![webhooks::webhooks])
        .mount("/", routes![service_images::service_images])
        .launch();
}
