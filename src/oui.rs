use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    time::Duration,
};

use curl::easy::Easy;
use pnet_datalink::MacAddr;

const DATABASE_PATH: &str = "/usr/local/share/oui.csv";
const DATABASE_TMP_PATH: &str = "/usr/local/share/oui.csv.tmp";
const DATABASE_URL: &str = "https://standards-oui.ieee.org/oui/oui.csv";
const DATABASE_UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24 * 7);

fn need_download(db_path: &Path) -> bool {
    if !db_path.exists() {
        return true;
    }

    let metadata = db_path
        .metadata()
        .expect("cannot get metadata for oui db file");
    let modified_time = metadata
        .modified()
        .expect("cannot get modified time for oui db file");

    let elapsed_since_modify = modified_time.elapsed().expect("elapsed() failed");
    log::info!(
        "OUI database file last modified {:?} ago",
        elapsed_since_modify
    );
    elapsed_since_modify > DATABASE_UPDATE_INTERVAL
}

fn download(path: &Path) {
    let file = File::create(DATABASE_TMP_PATH).expect("cannot create database");
    let mut writer = BufWriter::new(file);

    let mut easy = Easy::new();
    easy.url(DATABASE_URL).unwrap();
    easy.progress(true).unwrap();
    easy.progress_function(|to_download, downloaded, _to_upload, _uploaded| {
        log::debug!("downloading progress: {} / {}", downloaded, to_download);
        true
    })
    .unwrap();
    easy.write_function(move |data| {
        writer.write_all(data).expect("cannot write to database");
        Ok(data.len())
    })
    .unwrap();
    easy.perform().unwrap();

    assert_eq!(easy.response_code().unwrap(), 200);
    std::fs::rename(DATABASE_TMP_PATH, path).expect("cannot move database file");

    log::info!("OUI database download completed");
}

fn actualize_database(path: &Path) {
    if !need_download(path) {
        log::info!("OUI database is recent enough, not downloading");
        return;
    }

    log::info!(
        "will download OUI database from {} to {}",
        DATABASE_URL,
        path.display()
    );

    download(path);
}

fn load_database(path: &Path) -> HashMap<u32, String> {
    let mut reader = csv::Reader::from_path(path).expect("cannot open database");
    let headers = reader.headers().unwrap();
    assert_eq!(headers.get(1).unwrap(), "Assignment");
    assert_eq!(headers.get(2).unwrap(), "Organization Name");

    let mut result = HashMap::new();

    for record in reader.records() {
        let record = record.expect("cannot read csv record from database");

        let assignment = record.get(1).unwrap();
        let org_name = record.get(2).unwrap();

        let prefix =
            u32::from_str_radix(&assignment.to_lowercase(), 16).expect("cannot parse MAC prefix");
        result.insert(prefix, org_name.to_owned());
    }

    result
}

pub struct OuiDatabase {
    org_by_prefix: HashMap<u32, String>,
}

impl OuiDatabase {
    pub fn new() -> OuiDatabase {
        log::info!("loading OUI database");
        let path = Path::new(DATABASE_PATH);
        actualize_database(path);
        let org_by_prefix = load_database(path);
        log::info!("OUI database loaded");
        Self { org_by_prefix }
    }

    pub fn get_org(&self, mac: &MacAddr) -> Option<&str> {
        let prefix = ((mac.0 as u32) << 16) + ((mac.1 as u32) << 8) + (mac.2 as u32);
        self.org_by_prefix.get(&prefix).map(|s| s.as_str())
    }
}
