## File server ROCKET-FS

This is simplest file-server for exporting contents of local folder over HTTP

* written on [Rust](https://www.rust-lang.org/)
* based on [Rocket](https://rocket.rs/) framework
* UI based on [Bootstrap](https://getbootstrap.com/)

### How to use

* build binary ( **rocket-fs** )
* put it on some dir from **$PATH** variable
* go to folder that you want to export: `cd <export-dir>`
* start file-server: `rocket-fs`
* for customize - in <*export-dir*> create and edit *config files*

### Config files

It's possible to customize ***port/host/view*** via config files, placed  in exported folder:

* **rocket-fs.json**, this file used for configure **web-server** params (*host/port*), [default values](https://github.com/klim-iv/rocket-fs/blob/master/src/rocket-fs.json)
* **browse.rocket**, this file used for configure **page-view**, [default view](https://github.com/klim-iv/rocket-fs/blob/master/src/browse.rocket)
