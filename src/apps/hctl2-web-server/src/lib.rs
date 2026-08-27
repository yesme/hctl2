//! Loopback-only static file server for browser clients bundled with HCTL2.

#![forbid(unsafe_code)]

use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

/// Installed executable name.
pub const PROGRAM_NAME: &str = "hctl2-web-server";

const LOOPBACK: &str = "127.0.0.1";
const MAX_REQUEST_LINE_BYTES: usize = 16 * 1024;

/// Parsed command-line action.
#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    /// Print help and exit.
    Help(String),
    /// Print version and exit.
    Version(String),
    /// Serve the configured static directory.
    Serve(ServerConfig),
}

/// Static server configuration.
#[derive(Debug, Eq, PartialEq)]
pub struct ServerConfig {
    /// Absolute document root.
    pub root: PathBuf,
    /// Loopback TCP port.
    pub port: u16,
}

/// Parses the command-line interface.
///
/// # Errors
///
/// Returns a human-readable argument error when required values are absent or malformed.
pub fn parse_command(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let arguments: Vec<OsString> = arguments.into_iter().collect();
    if arguments.len() == 1 {
        if arguments[0] == OsStr::new("--help") || arguments[0] == OsStr::new("-h") {
            return Ok(Command::Help(help()));
        }
        if arguments[0] == OsStr::new("--version") || arguments[0] == OsStr::new("-V") {
            return Ok(Command::Version(version()));
        }
    }

    let mut root = None;
    let mut port = None;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        index += 1;
        if argument == OsStr::new("--root") {
            let value = arguments
                .get(index)
                .ok_or_else(|| "--root requires an absolute path".to_owned())?;
            index += 1;
            if root.replace(PathBuf::from(value)).is_some() {
                return Err("--root may only be supplied once".to_owned());
            }
        } else if argument == OsStr::new("--port") {
            let value = arguments
                .get(index)
                .ok_or_else(|| "--port requires a value".to_owned())?;
            index += 1;
            let value = value
                .to_str()
                .ok_or_else(|| "--port must be valid UTF-8".to_owned())?;
            let parsed = value
                .parse::<u16>()
                .map_err(|_| "--port must be between 1 and 65535".to_owned())?;
            if parsed == 0 {
                return Err("--port must be between 1 and 65535".to_owned());
            }
            if port.replace(parsed).is_some() {
                return Err("--port may only be supplied once".to_owned());
            }
        } else {
            return Err(format!(
                "unsupported argument {}",
                Path::new(argument).display()
            ));
        }
    }

    let root = root.ok_or_else(|| "--root is required".to_owned())?;
    if !root.is_absolute() {
        return Err("--root must be an absolute path".to_owned());
    }
    let port = port.ok_or_else(|| "--port is required".to_owned())?;

    Ok(Command::Serve(ServerConfig { root, port }))
}

/// Serves files until the process receives an operating-system termination signal.
///
/// # Errors
///
/// Returns an I/O error when the document root cannot be resolved or the loopback listener cannot
/// be created.
pub fn serve(config: &ServerConfig) -> io::Result<()> {
    let root = config.root.canonicalize()?;
    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "document root is not a directory",
        ));
    }

    let listener = TcpListener::bind((LOOPBACK, config.port))?;
    println!(
        "{PROGRAM_NAME}: serving {} at http://{LOOPBACK}:{}/",
        root.display(),
        config.port
    );

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let root = root.clone();
                thread::spawn(move || {
                    if let Err(error) = handle_connection(stream, &root) {
                        eprintln!("{PROGRAM_NAME}: connection error: {error}");
                    }
                });
            }
            Err(error) => eprintln!("{PROGRAM_NAME}: accept error: {error}"),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, root: &Path) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let request = read_request_line(&stream)?;
    let Some((method, target)) = parse_request_line(&request) else {
        return write_error(&mut stream, 400, "Bad Request", false);
    };
    let head_only = match method {
        "GET" => false,
        "HEAD" => true,
        _ => return write_error(&mut stream, 405, "Method Not Allowed", false),
    };

    let Some(path) = resolve_target(root, target) else {
        return write_error(&mut stream, 404, "Not Found", head_only);
    };
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return write_error(&mut stream, 404, "Not Found", head_only);
        }
        Err(error) => return Err(error),
    };
    let length = file.metadata()?.len();
    let content_type = content_type(&path);

    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Length: {length}\r\nContent-Type: {content_type}\r\nCache-Control: no-cache\r\nService-Worker-Allowed: /\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n"
    )?;
    if !head_only {
        io::copy(&mut BufReader::new(file), &mut stream)?;
    }
    stream.flush()
}

fn read_request_line(stream: &TcpStream) -> io::Result<String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .by_ref()
        .take(MAX_REQUEST_LINE_BYTES as u64)
        .read_line(&mut line)?;
    if line.len() >= MAX_REQUEST_LINE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "request line is too long",
        ));
    }
    Ok(line)
}

fn parse_request_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.trim_end_matches(['\r', '\n']).split_ascii_whitespace();
    let method = parts.next()?;
    let target = parts.next()?;
    let version = parts.next()?;
    if parts.next().is_some() || !matches!(version, "HTTP/1.0" | "HTTP/1.1") {
        return None;
    }
    Some((method, target))
}

fn resolve_target(root: &Path, target: &str) -> Option<PathBuf> {
    let path = target.split('?').next()?;
    if !path.starts_with('/') {
        return None;
    }
    let decoded = percent_decode(&path[1..])?;
    let mut relative = PathBuf::new();
    for component in decoded.split('/') {
        if component.is_empty() {
            continue;
        }
        if component == "." || component == ".." || component.contains(['\\', '\0']) {
            return None;
        }
        relative.push(component);
    }
    if relative.as_os_str().is_empty() || path.ends_with('/') {
        relative.push("index.html");
    }

    let candidate = root.join(relative).canonicalize().ok()?;
    candidate.starts_with(root).then_some(candidate)
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = hex_value(*bytes.get(index + 1)?)?;
            let low = hex_value(*bytes.get(index + 2)?)?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(OsStr::to_str) {
        Some("css") => "text/css; charset=utf-8",
        Some("gif") => "image/gif",
        Some("html") => "text/html; charset=utf-8",
        Some("ico") => "image/x-icon",
        Some("jpeg" | "jpg") => "image/jpeg",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("json" | "map") => "application/json; charset=utf-8",
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("ttf") => "font/ttf",
        Some("wasm") => "application/wasm",
        Some("webmanifest") => "application/manifest+json",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn write_error(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    head_only: bool,
) -> io::Result<()> {
    let body = format!("{status} {reason}\n");
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    if !head_only {
        stream.write_all(body.as_bytes())?;
    }
    stream.flush()
}

fn version() -> String {
    format!("{PROGRAM_NAME} {}", env!("CARGO_PKG_VERSION"))
}

fn help() -> String {
    format!(
        "{PROGRAM_NAME} {}\n\nServe a static directory on IPv4 loopback only.\n\nUsage:\n  {PROGRAM_NAME} --root ABSOLUTE_PATH --port PORT\n  {PROGRAM_NAME} --help\n  {PROGRAM_NAME} --version",
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use super::{Command, ServerConfig, content_type, parse_command, parse_request_line};

    #[test]
    fn parses_server_arguments() {
        let command = parse_command([
            OsString::from("--root"),
            OsString::from("/srv/element"),
            OsString::from("--port"),
            OsString::from("6168"),
        ]);

        assert_eq!(
            command,
            Ok(Command::Serve(ServerConfig {
                root: PathBuf::from("/srv/element"),
                port: 6168,
            }))
        );
    }

    #[test]
    fn rejects_relative_root() {
        let error = parse_command([
            OsString::from("--root"),
            OsString::from("relative"),
            OsString::from("--port"),
            OsString::from("6168"),
        ])
        .expect_err("relative path must be rejected");

        assert_eq!(error, "--root must be an absolute path");
    }

    #[test]
    fn parses_http_request_line() {
        assert_eq!(
            parse_request_line("GET /config.json HTTP/1.1\r\n"),
            Some(("GET", "/config.json"))
        );
        assert_eq!(parse_request_line("invalid\r\n"), None);
    }

    #[test]
    fn reports_browser_asset_content_types() {
        assert_eq!(
            content_type(Path::new("bundle.js")),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(content_type(Path::new("decoder.wasm")), "application/wasm");
        assert_eq!(content_type(Path::new("ringtone.mp3")), "audio/mpeg");
        assert_eq!(content_type(Path::new("ringtone.ogg")), "audio/ogg");
    }
}
