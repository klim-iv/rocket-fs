#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::env;
use std::fs;

use rocket::config::Config;
use rocket_contrib::serve::StaticFiles;
use rocket::response::content;

use handlebars::Handlebars;
use serde::{Serialize, Deserialize};

#[macro_use]
extern crate serde_json;

macro_rules! CFG_FILE {
  () => ("rocket-fs.json")
}

macro_rules! TEMPLATE_FILE {
  () => ("browse.rocket")
}

const TEMPLATE: &str = include_str!(TEMPLATE_FILE!());

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
  name: String,
  size: u64,
  f_type: u8,
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
          "inc_js": include_str!(concat!(env!("OUT_DIR"), "/inc_js")),
          "inc_css": include_str!(concat!(env!("OUT_DIR"), "/inc_css")),
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

#[derive(Debug, Serialize, Deserialize)]
struct Cfg {
  host: String,
  port: u16,
}

fn main() {
#[cfg(not(debug_assertions))]
  let mut www_cfg = Config::production();

#[cfg(debug_assertions)]
  let mut www_cfg = Config::development();

  let cfg;
  let f = fs::read(CFG_FILE!()).unwrap_or_default();
  if f.len() > 0 {
    cfg = String::from_utf8_lossy(&f).into_owned();
  } else {
    cfg = include_str!(CFG_FILE!()).to_string();
  }

  let cfg: Cfg = serde_json::from_str(&cfg).unwrap();
  let working_dir = env::current_dir();

  match working_dir {
    Ok(wd) => {
      www_cfg.set_address(cfg.host).unwrap();
      www_cfg.set_port(cfg.port);
      www_cfg.set_root(&wd);

      #[cfg(debug_assertions)]
      www_cfg.set_log_level(rocket::config::LoggingLevel::Debug);

      rocket::custom(www_cfg)
        .mount("/files", StaticFiles::from(&wd))
        .mount("/", routes![get_dir_tmpl, ])
        .launch();
    },
    Err(_e) => {
      #[cfg(debug_assertions)]
      print!("ERR ON OPEN DIR: {:?}\n", _e)
    }
  }
}
