extern crate download_lp;
extern crate base64;

use std::env;
use std::fs;
use std::path;
use std::io::Read;

const JS: &str = "https://cdn.jsdelivr.net/npm/bootstrap@5.0.0-beta3/dist/js/bootstrap.bundle.min.js";
const CSS: &str = "https://cdn.jsdelivr.net/npm/bootstrap@5.0.0-beta3/dist/css/bootstrap.min.css";
const ICONS: &str = "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.5.0/font/bootstrap-icons.css";

fn main() -> std::io::Result<()> {
  //download bootstrap-js
  let ret = download_lp::download(JS, format!("{}", env::var_os("OUT_DIR").unwrap().into_string().unwrap()));
  match ret {
    Ok(file) => {
      let t = path::Path::new(&env::var_os("OUT_DIR").unwrap().into_string().unwrap()).join(file.0);
      let f = fs::read(t);

      let s = format!("<script>\n{}\n</script>", String::from_utf8_lossy(&f.unwrap()));
      fs::write(path::Path::new(&env::var_os("OUT_DIR").unwrap().into_string().unwrap()).join("inc_js"), s)?;
    },
    Err(_) => {
      fs::write(
        format!("{}/inc_js", env::var_os("OUT_DIR").unwrap().into_string().unwrap()),
        format!("<script src=\"{}\"></script>", JS)
      )?
    }
  }

  //download bootstrap-css
  let ret = download_lp::download(CSS, format!("{}", env::var_os("OUT_DIR").unwrap().into_string().unwrap()));
  match ret {
    Ok(file) => {
      let t = path::Path::new(&env::var_os("OUT_DIR").unwrap().into_string().unwrap()).join(file.0);
      let f = fs::read(t);

      let s = format!("<style>\n{}\n</style>", String::from_utf8_lossy(&f.unwrap()));
      fs::write(path::Path::new(&env::var_os("OUT_DIR").unwrap().into_string().unwrap()).join("inc_css"), s)?;
    },
    Err(_) => {
      fs::write(
        format!("{}/inc_css", env::var_os("OUT_DIR").unwrap().into_string().unwrap()),
        format!("<link href=\"{}\" rel=\"stylesheet\">", CSS)
      )?
    }
  }

  //bootstrap icons, do not download at the moment
  fs::write(
    format!("{}/icon_css", env::var_os("OUT_DIR").unwrap().into_string().unwrap()),
    format!("<link href=\"{}\" rel=\"stylesheet\">", ICONS))?;

  //favicon
  let mut f = fs::File::open("favicon.ico")?;
  let mut _img = Vec::new();
  f.read_to_end(&mut _img)?;
  let b64 = base64::encode(&_img);
  fs::write(
    format!("{}/fav_icon_encoded", env::var_os("OUT_DIR").unwrap().into_string().unwrap()),
    format!("{}", b64))?;

  Ok(())
}
