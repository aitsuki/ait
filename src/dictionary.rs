use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const SOURCE_URL: &str = "https://raw.githubusercontent.com/skywind3000/ECDICT/39cfb9672c0906b63569e0453c2e6abfb9ab182a/ecdict.csv";
const SOURCE_SIZE: u64 = 66_009_548;
const SOURCE_SHA256: &str = "d1ecbdf9ee2b920afb6f430ca6b6bb9f559588cf0dff86e04bd255f9390843cc";
const SCHEMA_VERSION: i64 = 1;

#[derive(Clone)]
pub enum DownloadStatus {
    NotInstalled,
    Downloading(String),
    Ready,
    Failed(String),
}

fn download_state() -> &'static Mutex<Option<DownloadStatus>> {
    static STATE: OnceLock<Mutex<Option<DownloadStatus>>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(None))
}

fn dictionary_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("dev", "aitsu", "ait").ok_or("无法定位词典目录")?;
    Ok(dirs
        .data_local_dir()
        .join("dictionary")
        .join("ecdict-v1.sqlite3"))
}

fn open_dictionary(path: &Path) -> Result<Connection> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version != SCHEMA_VERSION {
        return Err("词典版本不兼容，请重新下载".into());
    }
    Ok(connection)
}

pub fn download_status() -> DownloadStatus {
    download_state()
        .lock()
        .unwrap()
        .get_or_insert_with(|| {
            match dictionary_path().and_then(|path| {
                let connection = open_dictionary(&path)?;
                connection.query_row("SELECT word FROM entries LIMIT 1", [], |_| Ok(()))?;
                Ok(())
            }) {
                Ok(()) => DownloadStatus::Ready,
                Err(_) => DownloadStatus::NotInstalled,
            }
        })
        .clone()
}

fn set_download_message(message: String) {
    *download_state().lock().unwrap() = Some(DownloadStatus::Downloading(message));
}

pub fn start_download() {
    {
        let mut state = download_state().lock().unwrap();
        if matches!(*state, Some(DownloadStatus::Downloading(_))) {
            return;
        }
        *state = Some(DownloadStatus::Downloading("正在连接…".to_string()));
    }
    std::thread::spawn(|| {
        let result = install_dictionary();
        *download_state().lock().unwrap() = Some(match result {
            Ok(()) => DownloadStatus::Ready,
            Err(err) => {
                tracing::warn!(error = %err, "dictionary download failed");
                DownloadStatus::Failed(format!(
                    "无法下载词典，请检查 GitHub 网络连接后重试。\n\n已有词典及翻译功能不受影响。\n\n{err}"
                ))
            }
        });
    });
}

fn install_dictionary() -> Result<()> {
    let path = dictionary_path()?;
    fs::create_dir_all(path.parent().ok_or("词典目录无效")?)?;
    let download_lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("download.lock"))?;
    download_lock
        .try_lock()
        .map_err(|err| format!("无法锁定词典目录，可能有另一个 ait 正在下载：{err}"))?;
    let csv_path = path.with_extension("csv.part");
    let db_path = path.with_extension("sqlite3.part");
    let result = (|| {
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(download_csv(&csv_path))?;
        set_download_message("正在建立索引…".to_string());
        if db_path.exists() {
            fs::remove_file(&db_path)?;
        }
        build_dictionary(&csv_path, &db_path)?;
        fs::write(
            path.with_file_name("ECDICT-LICENSE.txt"),
            include_str!("../docs/ECDICT-LICENSE.txt"),
        )?;
        replace_dictionary(&db_path, &path)?;
        Ok(())
    })();
    let _ = fs::remove_file(csv_path);
    let _ = fs::remove_file(db_path);
    result
}

async fn download_csv(path: &Path) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("ait/", env!("CARGO_PKG_VERSION")))
        .https_only(true)
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(600))
        .build()?;
    let mut response = client.get(SOURCE_URL).send().await?.error_for_status()?;
    if let Some(size) = response.content_length()
        && size != SOURCE_SIZE
    {
        return Err("词典下载大小不符".into());
    }
    let mut file = File::create(path)?;
    let mut hasher = Sha256::new();
    let mut received = 0;
    let mut last_mib = 0;
    while let Some(chunk) = response.chunk().await? {
        received += chunk.len() as u64;
        if received > SOURCE_SIZE {
            return Err("词典下载超出大小限制".into());
        }
        hasher.update(&chunk);
        file.write_all(&chunk)?;
        let mib = received / (1024 * 1024);
        if mib != last_mib {
            last_mib = mib;
            set_download_message(format!("{mib} / 63 MiB"));
        }
    }
    if received != SOURCE_SIZE || format!("{:x}", hasher.finalize()) != SOURCE_SHA256 {
        return Err("词典完整性校验失败，请重试".into());
    }
    file.sync_all()?;
    Ok(())
}

fn build_dictionary(csv_path: &Path, db_path: &Path) -> Result<()> {
    let mut reader = csv::Reader::from_path(csv_path)?;
    let headers = reader.headers()?.clone();
    let column = |name: &str| {
        headers
            .iter()
            .position(|header| header == name)
            .ok_or_else(|| format!("词典缺少字段：{name}"))
    };
    let word_col = column("word")?;
    let phonetic_col = column("phonetic")?;
    let translation_col = column("translation")?;
    let exchange_col = column("exchange")?;
    let mut connection = Connection::open(db_path)?;
    // Only the disposable build file is modified; the installed dictionary stays read-only.
    connection.execute_batch(
        "PRAGMA journal_mode=OFF;
         CREATE TABLE entries (word TEXT PRIMARY KEY, phonetic TEXT NOT NULL, translation TEXT NOT NULL);
         CREATE TABLE forms (form TEXT NOT NULL, word TEXT NOT NULL, PRIMARY KEY (form, word)) WITHOUT ROWID;",
    )?;
    let transaction = connection.transaction()?;
    {
        let mut insert =
            transaction.prepare("INSERT OR IGNORE INTO entries VALUES (?1, ?2, ?3)")?;
        let mut insert_form = transaction.prepare("INSERT OR IGNORE INTO forms VALUES (?1, ?2)")?;
        for record in reader.records() {
            let record = record?;
            let word = record[word_col].trim().to_ascii_lowercase();
            let translation = record[translation_col].trim();
            if word.is_empty() || translation.is_empty() {
                continue;
            }
            insert.execute(params![word, record[phonetic_col].trim(), translation])?;
            for exchange in record[exchange_col].split('/') {
                let Some((kind, forms)) = exchange.split_once(':') else {
                    continue;
                };
                if matches!(kind, "p" | "d" | "i" | "3" | "r" | "t" | "s") {
                    for form in forms.split(',') {
                        let form = form.trim().to_ascii_lowercase();
                        if !form.is_empty() {
                            insert_form.execute(params![form, word])?;
                        }
                    }
                } else if kind == "0" {
                    for lemma in forms.split(',') {
                        let lemma = lemma.trim().to_ascii_lowercase();
                        if !lemma.is_empty() {
                            insert_form.execute(params![word, lemma])?;
                        }
                    }
                }
            }
        }
    }
    transaction.commit()?;
    connection.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    let count: i64 = connection.query_row("SELECT count(*) FROM entries", [], |row| row.get(0))?;
    let check: String = connection.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if count < 100_000 || check != "ok" {
        return Err("生成的词典未通过校验".into());
    }
    connection.close().map_err(|(_, err)| err)?;
    fs::OpenOptions::new()
        .write(true)
        .open(db_path)?
        .sync_all()?;
    Ok(())
}

#[cfg(windows)]
fn replace_dictionary(from: &Path, to: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };
    use windows::core::PCWSTR;
    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        MoveFileExW(
            PCWSTR(from.as_ptr()),
            PCWSTR(to.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn replace_dictionary(from: &Path, to: &Path) -> Result<()> {
    fs::rename(from, to)?;
    Ok(())
}

// shortcut: only ASCII English words qualify, expand when accented headword lookup is needed.
pub fn lookup(text: &str, target_language: &str) -> Option<String> {
    if !matches!(
        target_language.to_ascii_lowercase().as_str(),
        "zh" | "zh-cn" | "zh-hans"
    ) {
        return None;
    }
    let word = text.trim().replace('’', "'").to_ascii_lowercase();
    if word.len() > 128
        || !word
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic() || byte == b'\'' || byte == b'-')
        || !word.starts_with(|c: char| c.is_ascii_alphabetic())
        || !word.ends_with(|c: char| c.is_ascii_alphabetic())
    {
        return None;
    }
    match lookup_inner(&word) {
        Ok(entry) => entry,
        Err(err) => {
            tracing::debug!(error = %err, "local dictionary unavailable, falling back to translation");
            None
        }
    }
}

fn lookup_inner(word: &str) -> Result<Option<String>> {
    let path = dictionary_path()?;
    if !path.is_file() {
        return Ok(None);
    }
    let connection = open_dictionary(&path)?;
    let format_entry = |row: &rusqlite::Row<'_>| -> rusqlite::Result<String> {
        let word: String = row.get(0)?;
        let phonetic: String = row.get(1)?;
        let translation: String = row.get(2)?;
        let heading = if phonetic.is_empty() {
            word
        } else {
            format!("{word}  /{phonetic}/")
        };
        Ok(format!(
            "{heading}\r\n{}",
            translation.replace("\\n", "\r\n")
        ))
    };
    let exact = connection
        .query_row(
            "SELECT word, phonetic, translation FROM entries WHERE word = ?1",
            [word],
            format_entry,
        )
        .optional()?;
    let mut entries: Vec<_> = exact.into_iter().collect();
    let mut statement = connection.prepare(
        "SELECT e.word, e.phonetic, e.translation FROM forms f JOIN entries e ON e.word = f.word
         WHERE f.form = ?1 AND e.word != ?1 ORDER BY e.word LIMIT 8",
    )?;
    for entry in statement.query_map([word], format_entry)? {
        entries.push(entry?);
    }
    Ok((!entries.is_empty()).then(|| entries.join("\r\n────────\r\n")))
}
