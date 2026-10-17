use std::fs::File;
use std::io::{self, Read, Write};
use std::time::Duration;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub data: Option<Vec<u8>>,
    pub silent: bool,
    pub fail: bool,
    pub output_file: Option<String>,
    pub timeout: Duration,
    pub write_out: Option<String>,
    pub head_only: bool,
    pub follow_redirects: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: String::new(),
            headers: Vec::new(),
            data: None,
            silent: false,
            fail: false,
            output_file: None,
            timeout: Duration::from_secs(30),
            write_out: None,
            head_only: false,
            follow_redirects: 5,
        }
    }
}

pub enum Action {
    Run(Config),
    Version,
    Help,
}

pub fn parse_args<I>(args: I) -> Result<Action, String>
where
    I: IntoIterator<Item = String>,
{
    let mut config = Config::default();
    let mut args_iter = args.into_iter().peekable();
    let mut explicit_method = false;

    while let Some(arg) = args_iter.next() {
        if arg == "-h" || arg == "--help" {
            return Ok(Action::Help);
        } else if arg == "-V" || arg == "--version" {
            return Ok(Action::Version);
        } else if arg == "-s" || arg == "--silent" {
            config.silent = true;
        } else if arg == "-f" || arg == "--fail" {
            config.fail = true;
        } else if arg == "-L" || arg == "--location" {
            config.follow_redirects = 10;
        } else if arg == "-I" || arg == "--head" {
            config.head_only = true;
        } else if arg == "-X" || arg == "--request" {
            let val = args_iter
                .next()
                .ok_or_else(|| format!("Option {} requires an argument", arg))?;
            config.method = val.to_uppercase();
            explicit_method = true;
        } else if let Some(val) = arg.strip_prefix("--request=") {
            config.method = val.to_uppercase();
            explicit_method = true;
        } else if arg.starts_with("-X") && arg.len() > 2 {
            config.method = arg[2..].to_uppercase();
            explicit_method = true;
        } else if arg == "-H" || arg == "--header" {
            let val = args_iter
                .next()
                .ok_or_else(|| format!("Option {} requires an argument", arg))?;
            parse_header_pair(&val, &mut config.headers)?;
        } else if let Some(val) = arg.strip_prefix("--header=") {
            parse_header_pair(val, &mut config.headers)?;
        } else if arg == "-d" || arg == "--data" || arg == "--data-raw" {
            let val = args_iter
                .next()
                .ok_or_else(|| format!("Option {} requires an argument", arg))?;
            config.data = Some(read_data_arg(&val)?);
        } else if let Some(val) = arg.strip_prefix("--data=") {
            config.data = Some(read_data_arg(val)?);
        } else if arg == "-o" || arg == "--output" {
            let val = args_iter
                .next()
                .ok_or_else(|| format!("Option {} requires an argument", arg))?;
            config.output_file = Some(val);
        } else if let Some(val) = arg.strip_prefix("--output=") {
            config.output_file = Some(val.to_string());
        } else if arg == "-m" || arg == "--max-time" {
            let val = args_iter
                .next()
                .ok_or_else(|| format!("Option {} requires an argument", arg))?;
            let secs: f64 = val
                .parse()
                .map_err(|_| format!("Invalid timeout: {}", val))?;
            config.timeout = Duration::from_secs_f64(secs.max(0.01));
        } else if let Some(val) = arg.strip_prefix("--max-time=") {
            let secs: f64 = val
                .parse()
                .map_err(|_| format!("Invalid timeout: {}", val))?;
            config.timeout = Duration::from_secs_f64(secs.max(0.01));
        } else if arg == "-w" || arg == "--write-out" {
            let val = args_iter
                .next()
                .ok_or_else(|| format!("Option {} requires an argument", arg))?;
            config.write_out = Some(val);
        } else if let Some(val) = arg.strip_prefix("--write-out=") {
            config.write_out = Some(val.to_string());
        } else if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            // Handle combined short flags like -fsSL, -sS, etc.
            let chars: Vec<char> = arg[1..].chars().collect();
            let mut i = 0;
            while i < chars.len() {
                match chars[i] {
                    's' => config.silent = true,
                    'f' => config.fail = true,
                    'S' => {} // show error if silent fails
                    'L' => config.follow_redirects = 10,
                    'I' => config.head_only = true,
                    'o' => {
                        let rest: String = chars[i + 1..].iter().collect();
                        if !rest.is_empty() {
                            config.output_file = Some(rest);
                        } else {
                            let next_val = args_iter
                                .next()
                                .ok_or_else(|| "Option -o requires an argument".to_string())?;
                            config.output_file = Some(next_val);
                        }
                        break;
                    }
                    'm' => {
                        let rest: String = chars[i + 1..].iter().collect();
                        let val = if !rest.is_empty() {
                            rest
                        } else {
                            args_iter
                                .next()
                                .ok_or_else(|| "Option -m requires an argument".to_string())?
                        };
                        let secs: f64 = val
                            .parse()
                            .map_err(|_| format!("Invalid timeout: {}", val))?;
                        config.timeout = Duration::from_secs_f64(secs.max(0.01));
                        break;
                    }
                    'X' => {
                        let rest: String = chars[i + 1..].iter().collect();
                        let val = if !rest.is_empty() {
                            rest
                        } else {
                            args_iter
                                .next()
                                .ok_or_else(|| "Option -X requires an argument".to_string())?
                        };
                        config.method = val.to_uppercase();
                        explicit_method = true;
                        break;
                    }
                    'H' => {
                        let rest: String = chars[i + 1..].iter().collect();
                        let val = if !rest.is_empty() {
                            rest
                        } else {
                            args_iter
                                .next()
                                .ok_or_else(|| "Option -H requires an argument".to_string())?
                        };
                        parse_header_pair(&val, &mut config.headers)?;
                        break;
                    }
                    'd' => {
                        let rest: String = chars[i + 1..].iter().collect();
                        let val = if !rest.is_empty() {
                            rest
                        } else {
                            args_iter
                                .next()
                                .ok_or_else(|| "Option -d requires an argument".to_string())?
                        };
                        config.data = Some(read_data_arg(&val)?);
                        break;
                    }
                    other => {
                        if !config.silent {
                            eprintln!("Warning: unknown option -{}", other);
                        }
                    }
                }
                i += 1;
            }
        } else if config.url.is_empty() {
            config.url = arg;
        } else {
            return Err(format!("Unexpected argument: {}", arg));
        }
    }

    if config.url.is_empty() {
        return Err("No URL specified. Run with -h for help.".to_string());
    }

    // Determine HTTP method
    if !explicit_method {
        if config.head_only {
            config.method = "HEAD".to_string();
        } else if config.data.is_some() {
            config.method = "POST".to_string();
        } else {
            config.method = "GET".to_string();
        }
    }

    Ok(Action::Run(config))
}

fn parse_header_pair(raw: &str, headers: &mut Vec<(String, String)>) -> Result<(), String> {
    if let Some((k, v)) = raw.split_once(':') {
        headers.push((k.trim().to_string(), v.trim().to_string()));
        Ok(())
    } else {
        Err(format!(
            "Invalid header format '{}', expected 'Name: Value'",
            raw
        ))
    }
}

fn read_data_arg(val: &str) -> Result<Vec<u8>, String> {
    if val == "-" {
        let mut buf = Vec::new();
        io::stdin()
            .read_to_end(&mut buf)
            .map_err(|e| format!("Failed to read data from stdin: {}", e))?;
        Ok(buf)
    } else if let Some(path) = val.strip_prefix('@') {
        std::fs::read(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))
    } else {
        Ok(val.as_bytes().to_vec())
    }
}

pub fn print_help(bin_name: &str) {
    println!(
        "★ Svyaz (sv) v{} — Micro HTTP/HTTPS client in pure Rust ★",
        VERSION
    );
    println!("Gosplan Ministry of Communications (Связь СССР)\n");
    println!("Usage:");
    println!("  {} [options] <URL>\n", bin_name);
    println!("Options:");
    println!("  -X, --request <METHOD>   Specify request method (GET, POST, PUT, PATCH, HEAD)");
    println!(
        "  -H, --header <HEADER>    Pass custom header to server (e.g. 'Authorization: token xxx')"
    );
    println!("  -d, --data <DATA>        HTTP POST/PUT/PATCH data (use @file to load from file, - for stdin)");
    println!("  -s, --silent             Silent mode (do not output errors or progress)");
    println!("  -f, --fail               Fail silently (exit code 22) on HTTP 4xx/5xx errors");
    println!("  -o, --output <FILE>      Write output to <FILE> instead of stdout");
    println!("  -m, --max-time <SECONDS> Maximum time allowed for the transfer");
    println!("  -w, --write-out <FORMAT> Output format after completion (supports %{{http_code}})");
    println!("  -I, --head               Show header status only (HEAD request)");
    println!("  -L, --location           Follow HTTP 3xx redirects");
    println!("  -V, --version            Display version information");
    println!("  -h, --help               Display this help text\n");
    println!("Examples:");
    println!("  {} https://api.github.com/zen", bin_name);
    println!(
        "  {} -s -f -m 3 https://codeberg.org/api/v1/version",
        bin_name
    );
    println!("  {} -X POST -H 'Content-Type: application/json' -d '{{\"hello\":\"world\"}}' https://httpbin.org/post", bin_name);
    println!(
        "  {} -s -o /dev/null -w '%{{http_code}}' -X PATCH https://api.example.com/item",
        bin_name
    );
}

pub fn execute(config: Config) -> Result<i32, String> {
    let agent: ureq::Agent = ureq::AgentBuilder::new()
        .timeout(config.timeout)
        .redirects(config.follow_redirects)
        .build();

    let mut req = agent.request(&config.method, &config.url);

    // Apply headers
    let mut has_user_agent = false;
    for (k, v) in &config.headers {
        if k.eq_ignore_ascii_case("user-agent") {
            has_user_agent = true;
        }
        req = req.set(k, v);
    }
    if !has_user_agent {
        req = req.set("User-Agent", &format!("svyaz/{}", VERSION));
    }

    // Execute request
    let response_result = if let Some(ref body) = config.data {
        req.send_bytes(body)
    } else {
        req.call()
    };

    let (status_code, reader) = match response_result {
        Ok(resp) => {
            let status = resp.status();
            (status, Some(resp.into_reader()))
        }
        Err(ureq::Error::Status(status, resp)) => (status, Some(resp.into_reader())),
        Err(ureq::Error::Transport(err)) => {
            if !config.silent {
                eprintln!("svyaz: transport error: {}", err);
            }
            return Ok(7); // curl exit code 7 = Failed to connect to host
        }
    };

    // Fail flag check
    if config.fail && status_code >= 400 {
        if !config.silent {
            eprintln!("svyaz: HTTP error {}", status_code);
        }
        // Write-out still printed if requested
        if let Some(ref fmt) = config.write_out {
            let mut out = fmt.replace("%{http_code}", &status_code.to_string());
            out = out.replace("\\n", "\n").replace("\\t", "\t");
            print!("{}", out);
            let _ = io::stdout().flush();
        }
        return Ok(22); // curl exit code 22 = HTTP page not retrieved
    }

    // Read body if not HEAD
    let mut body_bytes = Vec::new();
    if !config.head_only {
        if let Some(mut r) = reader {
            let _ = r.read_to_end(&mut body_bytes);
        }
    }

    // Output body
    if let Some(ref dest) = config.output_file {
        if dest != "/dev/null" {
            let mut file = File::create(dest)
                .map_err(|e| format!("Failed to create output file '{}': {}", dest, e))?;
            file.write_all(&body_bytes)
                .map_err(|e| format!("Failed to write to file '{}': {}", dest, e))?;
        }
    } else {
        io::stdout()
            .write_all(&body_bytes)
            .map_err(|e| format!("Failed to write to stdout: {}", e))?;
        let _ = io::stdout().flush();
    }

    // Format write-out
    if let Some(ref fmt) = config.write_out {
        let mut out = fmt.replace("%{http_code}", &status_code.to_string());
        out = out.replace("\\n", "\n").replace("\\t", "\t");
        print!("{}", out);
        let _ = io::stdout().flush();
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_get() {
        let args = vec!["https://example.com/api".to_string()];
        match parse_args(args).expect("parse failed") {
            Action::Run(cfg) => {
                assert_eq!(cfg.url, "https://example.com/api");
                assert_eq!(cfg.method, "GET");
                assert!(!cfg.silent);
                assert!(!cfg.fail);
            }
            _ => panic!("Expected Action::Run"),
        }
    }

    #[test]
    fn test_parse_combined_flags() {
        let args = vec![
            "-fsSL".to_string(),
            "-o".to_string(),
            "/tmp/output.tar.gz".to_string(),
            "https://example.com/archive".to_string(),
        ];
        match parse_args(args).expect("parse failed") {
            Action::Run(cfg) => {
                assert_eq!(cfg.url, "https://example.com/archive");
                assert!(cfg.fail);
                assert!(cfg.silent);
                assert_eq!(cfg.follow_redirects, 10);
                assert_eq!(cfg.output_file, Some("/tmp/output.tar.gz".to_string()));
            }
            _ => panic!("Expected Action::Run"),
        }
    }

    #[test]
    fn test_parse_post_with_headers_and_data() {
        let args = vec![
            "-X".to_string(),
            "POST".to_string(),
            "-H".to_string(),
            "Authorization: token secret123".to_string(),
            "-H".to_string(),
            "Content-Type: application/json".to_string(),
            "-d".to_string(),
            "{\"title\":\"bug\"}".to_string(),
            "https://codeberg.org/api/v1/repos/test/issues".to_string(),
        ];
        match parse_args(args).expect("parse failed") {
            Action::Run(cfg) => {
                assert_eq!(cfg.method, "POST");
                assert_eq!(cfg.headers.len(), 2);
                assert_eq!(
                    cfg.headers[0],
                    ("Authorization".to_string(), "token secret123".to_string())
                );
                assert_eq!(
                    cfg.headers[1],
                    ("Content-Type".to_string(), "application/json".to_string())
                );
                assert_eq!(cfg.data, Some(b"{\"title\":\"bug\"}".to_vec()));
            }
            _ => panic!("Expected Action::Run"),
        }
    }

    #[test]
    fn test_parse_write_out_and_timeout() {
        let args = vec![
            "-s".to_string(),
            "-o".to_string(),
            "/dev/null".to_string(),
            "-w".to_string(),
            "%{http_code}".to_string(),
            "-m".to_string(),
            "5".to_string(),
            "https://example.com/health".to_string(),
        ];
        match parse_args(args).expect("parse failed") {
            Action::Run(cfg) => {
                assert!(cfg.silent);
                assert_eq!(cfg.output_file, Some("/dev/null".to_string()));
                assert_eq!(cfg.write_out, Some("%{http_code}".to_string()));
                assert_eq!(cfg.timeout, Duration::from_secs(5));
            }
            _ => panic!("Expected Action::Run"),
        }
    }
}
