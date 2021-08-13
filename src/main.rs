#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::env;
use std::fs;

use rocket::config::Config;
use rocket_contrib::serve::{StaticFiles, Options};
use rocket::response::{Redirect, content};

use handlebars::Handlebars;
use serde::{Serialize, Deserialize};

use urlencoding::{encode, decode};

#[macro_use]
extern crate serde_json;

macro_rules! CFG_FILE {
  () => ("rocket-fs.json")
}

macro_rules! TEMPLATE_FILE {
  () => ("browse.rocket")
}

const TEMPLATE: &str = include_str!(TEMPLATE_FILE!());

fn escape_fn(s:&str) -> String {
  return decode(s).unwrap().to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
  name: String,
  size: u64,
  f_type: u8,
}

#[get("/")]
fn index() -> Redirect {
  Redirect::to("/?dir=.")
}

#[get("/?<dir>")]
fn get_dir_tmpl(dir:String) -> content::Html<String> {
  let working_dir = env::current_dir();
  let dir_name = decode(&dir).unwrap().to_string();
  match working_dir {
    Ok(_wd) => {
      let wd = _wd.join(dir_name.clone());
      let mut dirs = Vec::new();
      let mut out = Vec::new();
      let d = fs::read_dir(&wd);
      match d {
        Ok(_files) => {
          let mut files: Vec<_> = _files.map(|f| f.unwrap()).collect();
          files.sort_by_key(|k|
            if k.metadata().unwrap().is_dir() {
              "0_".to_owned() + &k.file_name().into_string().unwrap()
            }
            else {
              "1_".to_owned() + &k.file_name().into_string().unwrap()
            }
          );
          for r in files {
            if let Ok(ft) = r.file_type() {
              if !ft.is_dir() {
                if let Ok(m) = r.metadata() {
                  let file_data = FileInfo {
                    name : encode(&r.file_name().into_string().unwrap_or("_".to_string())).to_string(),
                    size : m.len(),
                    f_type : match m.file_type().is_file() {
                      true => 1,
                      false => 2,
                    },
                  };
                  out.push(file_data);
                }
              } else {
                let file_data = FileInfo {
                  name : encode(&r.file_name().into_string().unwrap_or("_".to_string())).to_string(),
                  size : 0,
                  f_type : 2,
                };
                dirs.push(file_data);
              }
            }
          }
        },
        Err(e) => {
          print!("ERR IN READ DIR: {:?}\n", e)
        }
      }

      let mut template = TEMPLATE.to_string();
      let f = fs::read(_wd.join("browse.rocket"));

      match f {
        Ok(s) => template = String::from_utf8_lossy(&s).into_owned(),
        Err(_) => (),
      }

      let mut hb = Handlebars::new();
      hb.register_escape_fn(escape_fn);
      let rhb = hb.render_template(&template, &json!(
        {
          "title": dir_name,
          "dirs": dirs,
          "content": out,
          "inc_js": include_str!(concat!(env!("OUT_DIR"), "/inc_js")),
          "inc_css": include_str!(concat!(env!("OUT_DIR"), "/inc_css")),
          "icon_css": include_str!(concat!(env!("OUT_DIR"), "/icon_css")),
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

      let options = Options::Index | Options::DotFiles;
      rocket::custom(www_cfg)
        .mount("/files", StaticFiles::new(&wd, options))
        .mount("/", routes![index, get_dir_tmpl, ])
        .launch();
    },
    Err(_e) => {
      #[cfg(debug_assertions)]
      print!("ERR ON OPEN DIR: {:?}\n", _e)
    }
  }
}
