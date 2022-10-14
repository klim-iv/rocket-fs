#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::env;
use std::fs;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::SystemTime;

use rocket::config::Config;
use rocket_contrib::serve::{StaticFiles, Options};
use rocket::response::{Redirect, content};

use handlebars::Handlebars;
use serde::{Serialize, Deserialize};

use urlencoding::{encode, decode};

#[macro_use]
extern crate serde_json;

extern crate chrono;
use chrono::offset::Utc;
use chrono::DateTime;

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
  created: String,
}

#[get("/")]
fn index() -> Redirect {
  Redirect::to(format!("{}/?dir=.", &Cfg::current().prefix))
}

#[get("/?<dir>")]
fn get_dir_tmpl(dir:String) -> content::Html<String> {
  let working_dir = env::current_dir();
  let _cn = working_dir.as_ref().unwrap().canonicalize().unwrap();
  let dir_name = decode(&dir).unwrap().to_string();
  match working_dir {
    Ok(_wd) => {
      let wd = _wd.join(dir_name.clone());
      let _cn_wd = wd.canonicalize().unwrap();
      if !_cn_wd.starts_with(_cn) {
          return content::Html(format!("ERR ON OPEN DIR: {:?}\n", dir))
      }
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
                  let datetime: DateTime<Utc> = m.created().unwrap().into();
                  let file_data = FileInfo {
                    name : encode(&r.file_name().into_string().unwrap_or("_".to_string())).to_string(),
                    size : m.len(),
                    f_type : match m.file_type().is_file() {
                      true => 1,
                      false => 2,
                    },
                    created : datetime.format("%d-%m-%Y %T").to_string(),
                  };
                  out.push(file_data);
                }
              } else {
                let file_data = FileInfo {
                  name : encode(&r.file_name().into_string().unwrap_or("_".to_string())).to_string(),
                  size : 0,
                  f_type : 2,
                  created : format!("{:?}", SystemTime::UNIX_EPOCH),
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
  prefix: String,
  host: String,
  port: u16,
}

impl Cfg {
  pub fn current() -> Arc<Cfg> {
    CURRENT_CONFIG.with(|c| c.read().unwrap().clone())
  }
}

impl Default for Cfg {
    fn default() -> Self {
      let cfg;
      let f = fs::read(CFG_FILE!()).unwrap_or_default();
      if f.len() > 0 {
        cfg = String::from_utf8_lossy(&f).into_owned();
      } else {
        cfg = include_str!(CFG_FILE!()).to_string();
      }

      let cfg: Cfg = serde_json::from_str(&cfg).unwrap();
      cfg
  }
}

thread_local! {
  static CURRENT_CONFIG: RwLock<Arc<Cfg>> = RwLock::new(Default::default());
}


fn main() {
#[cfg(not(debug_assertions))]
  let mut www_cfg = Config::production();

#[cfg(debug_assertions)]
  let mut www_cfg = Config::development();

  let working_dir = env::current_dir();

  match working_dir {
    Ok(wd) => {
      www_cfg.set_address(&Cfg::current().host).unwrap();
      www_cfg.set_port(Cfg::current().port);
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
