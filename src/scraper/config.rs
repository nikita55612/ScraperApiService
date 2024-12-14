#![allow(warnings)]
use once_cell::sync::OnceCell;
use serde::Deserialize;
use super::super::utils::read_file;


static INIT: OnceCell<Config> = OnceCell::new();

pub fn get() -> &'static Config {
    INIT.get_or_init(|| {
        match read_file("Config.toml") {
            Ok(cfg) => toml::from_str::<Config>(&cfg)
                .unwrap_or_else(|_| {
                    eprint!("Fail to parse config!");
                    Config::default()
                }
            ),
            Err(_) => {
                eprint!("Fail to read config!");
                Config::default()
            }
        }
    })
}

#[derive(Deserialize, Default, Debug)]
pub struct Config {

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_config() {
        println!("{:#?}", get());
        assert_eq!(true, true);
    }
}
