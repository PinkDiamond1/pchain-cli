/*
 Copyright (c) 2022 ParallelChain Lab

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use home; 
use serde::{Deserialize, Serialize};
use serde_json::Value;

const PCHAIN_CLI_CONFIG_PATH: &str = ".parallelchain/pchain_cli/config.json";

// Config.json fields for pchain
#[derive(Serialize, Deserialize)]
struct Config {
    target_url: String,
    rich_api_url: String,
    analytics_api_url: String,
    keypair_json_path: String,
}

pub enum ConfigField {
    TargetUrl,
    RichApiUrl,
    AnalyticsApiUrl,
    KeypairJSONPath,
}

impl Into<String> for &ConfigField{
    fn into(self) -> String {
        match self {
            ConfigField::TargetUrl => "target_url".to_string(),
            ConfigField::RichApiUrl => "rich_api_url".to_string(),
            ConfigField::AnalyticsApiUrl => "analytics_api_url".to_string(),
            ConfigField::KeypairJSONPath => "keypair_json_path".to_string(),
        }
    }
}

// Set config first figure out whether the file `HOME/.parallelchain/pchain_cli/config.json` exist.
// If no, it creates the file and path.
// Then it write the corresponding data field to config.json
pub fn set_config(field: ConfigField, field_value: &str){
    let mut default_config_path = PathBuf::from(home::home_dir().expect("Fail to find home directory. The home path might belong to root"));
    default_config_path.push(PCHAIN_CLI_CONFIG_PATH);

    if !Path::new(&default_config_path).is_file(){
        if !Path::new(&default_config_path.parent().unwrap()).exists(){
            match std::fs::create_dir_all(default_config_path.parent().unwrap()){
                Ok(_) => {},
                Err(e) => {
                    println!("Error: Cannot create directory to config. {}", e);
                    std::process::exit(1);
                }
            };
        };
        match std::fs::File::create(&default_config_path.clone()){
            Ok(_) => {},
            Err(e) => {
                println!("Error: Cannot create config file. {}", e);
                std::process::exit(1);
            }
        }
    };
    let config_string: String = String::from_utf8(std::fs::read(&default_config_path).unwrap()).unwrap();
    let mut config: HashMap<String, Value> = HashMap::new();
    if config_string.trim() != "" {
        config = match serde_json::from_str(&config_string){
            Ok(data) => data,
            Err(_) => {
                println!("Error: Incorrect format of json.");
                std::process::exit(1);
            }
        };
    };

    config.insert(Into::<String>::into(&field), Value::from(field_value));

    let config_update_string = match serde_json::to_string_pretty(&config){
        Ok(data) => data,
        Err(_) => {
            println!("Error: Incorrect format of json.");
            std::process::exit(1);
        }
    };

    match std::fs::write(&default_config_path, config_update_string){
        Ok(_) => {println!("{} set.", Into::<String>::into(&field))},
        Err(e) => {
            println!("Error: Failed to update config json. {}", e);
            std::process::exit(1);            
        }
    };
}

// Read config first figure out whether the file `HOME/.parallelchain/pchain_cli/config.json` exist.
// Then it read the corresponding data field to config.json if it exists.
pub fn read_config(config_var: ConfigField) -> String {
    let mut default_config_path = PathBuf::from(home::home_dir().expect("The home path might belong to root"));
    default_config_path.push(PCHAIN_CLI_CONFIG_PATH);

    let config = match std::fs::File::open(default_config_path){
        Ok(json) => json,
        Err(_) => {
            println!("Error: Config file not set. Field `{}` does not exist. Please use `setup` command to complete config file.", Into::<String>::into(&config_var));
            std::process::exit(1);
        }
    };

    let json: serde_json::Value = match serde_json::from_reader(config){
        Ok(input) => input,
        Err(_) => {
            println!("Error: The config file should be a proper json.");
            std::process::exit(1);          
        }
    };

    let field_interested: String = match json.get::<String>(Into::<String>::into(&config_var)){
        Some(value) => value.as_str().unwrap().to_string(),
        None => {
            println!("Error: Field `{}` does not exist. Please fill it up using the `setup` command", Into::<String>::into(&config_var));
            std::process::exit(1);
        }
    };
    field_interested
}
