extern crate download_lp;

use std::env;
use std::fs;
use std::path;

const JS: &str = "https://cdn.jsdelivr.net/npm/bootstrap@5.0.0-beta3/dist/js/bootstrap.bundle.min.js";
const CSS: &str = "https://cdn.jsdelivr.net/npm/bootstrap@5.0.0-beta3/dist/css/bootstrap.min.css";

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

  Ok(())
}
