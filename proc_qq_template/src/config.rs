use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::path::Path;
use std::process::exit;

const CONFIG_FILE_PATH: &'static str = "bot.yml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub uin: i64,
    pub password_md5: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mongo {
    pub host: String,
    pub port: u32,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Redis {
    pub host: String,
    pub port: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub account: Account,
    pub mongo: Mongo,
    pub redis: Redis,
}

pub(crate) async fn load_config() -> anyhow::Result<Config> {
    let mut config = Config {
        account: Account {
            uin: 123456789,
            password_md5: "echo -n password | md5".to_string(),
        },
        mongo: Mongo {
            host: "127.0.0.1".to_string(),
            port: 27017,
            database: "bot".to_string(),
        },
        redis: Redis {
            host: "127.0.0.1".to_string(),
            port: 6379,
        },
    };
    if Path::new(CONFIG_FILE_PATH).exists() {
        config = serde_yaml::from_str(&std::fs::read_to_string(CONFIG_FILE_PATH)?)?;
    } else {
        let data = serde_yaml::to_string(&config)?;
        std::fs::write(CONFIG_FILE_PATH, data)?;
    };
    if config.account.uin == 123456789 {
        println!("请修改 bot.yml");
        exit(0);
    };
    Ok(config)
}
