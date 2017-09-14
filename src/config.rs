use ::std::io::Read;

use ::toml;
use ::serde_derive;

use ::errors::*;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub front_panel_serial: String,
}

pub fn read_config() -> Result<Config> {
    let mut file = ::std::fs::File::open("./config.toml")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}
