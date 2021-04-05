#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::env;
use std::fs;

use rocket::config::{Config, LoggingLevel};
use rocket_contrib::serve::StaticFiles;
use rocket::response::content;

use handlebars::Handlebars;
use serde::ser::{Serialize, SerializeStruct};

#[macro_use]
extern crate serde_json;

const TEMPLATE: &str = include_str!("browse.rocket");

#[derive(Debug)]
struct FileInfo {
  name: String,
  size: u64,
  f_type: u8,
}

impl Serialize for FileInfo {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    let mut f_info = serializer.serialize_struct("FileInfo", 2)?;
    f_info.serialize_field("name", &self.name)?;
    f_info.serialize_field("size", &self.size)?;
    f_info.serialize_field("type", &self.f_type)?;
    f_info.end()
  }
}

#[get("/")]
fn get_dir_tmpl() -> content::Html<String> {
  let working_dir = env::current_dir();
  match working_dir {
    Ok(wd) => {
      let mut out = Vec::new();
      let d = fs::read_dir(&wd);
      match d {
        Ok(files) => {
          for r in files {
              if let Ok(f) = r {
                if let Ok(ft) = f.file_type() {
                  if !ft.is_dir() {
                    if let Ok(m) = f.metadata() {
                      let file_data = FileInfo {
                        name : f.file_name().into_string().unwrap_or("_".to_string()),
                        size : m.len(),
                        f_type : match m.file_type().is_file() {
                          true => 1,
                          false => 2,
                        },
                      };
                      out.push(file_data);
                    }
                  }
                }
              }
          }
        },
        Err(e) => {
          print!("ERR IN READ DIR: {:?}\n", e)
        }
      }

      let mut template = TEMPLATE.to_string();
      let f = fs::read(wd.join("/browse.rocket"));

      match f {
        Ok(s) => template = String::from_utf8_lossy(&s).into_owned(),
        Err(_) => (),
      }

      let hb = Handlebars::new();
      let rhb = hb.render_template(&template, &json!(
        {
          "title": "dir",
          "content": out,
        }
      ));

      let out : String;

      match rhb {
        Ok(c) => out = c,
        Err(e) => out = format!("ERR IN TEMPLATE: {:?}\n", e),
      }
      content::Html(out)
    },
    Err(e) => {
      content::Html(format!("ERR ON OPEN DIR: {:?}\n", e))
    }
  }

}

fn main() {
  //let mut cfg = Config::production();
  let mut cfg = Config::development();
  let working_dir = env::current_dir();

  match working_dir {
    Ok(wd) => {
//      cfg.set_address("172.11.111.11");
      cfg.set_address("0.0.0.0");
      cfg.set_port(9090);
      cfg.set_root(&wd);
      cfg.set_log_level(LoggingLevel::Debug);

      rocket::custom(cfg)
        .mount("/files", StaticFiles::from(&wd))
        .mount("/", routes![get_dir_tmpl, ])
        .launch();
    },
    Err(e) => {
      print!("ERR ON OPEN DIR: {:?}\n", e)
    }
  }
}
