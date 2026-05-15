use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const INDEX_HTML: &str = include_str!("docs_assets/index.html");
const APP_JS: &str = include_str!("docs_assets/app.js");
const STYLE_CSS: &str = include_str!("docs_assets/style.css");

static CTRL_C_SHUTDOWN: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
pub struct DocsServer {
    port: u16,
    shutdown: Arc<AtomicBool>,
}

impl DocsServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self, explain_json: String) -> Result<(), String> {
        CTRL_C_SHUTDOWN.store(false, Ordering::SeqCst);
        install_ctrlc_handler();

        let listener = TcpListener::bind(("127.0.0.1", self.port)).map_err(|e| {
            format!(
                "error: failed to bind docs server on 127.0.0.1:{}: {}",
                self.port, e
            )
        })?;
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("error: failed to configure docs server: {}", e))?;

        let explain_json = Arc::new(explain_json);
        let stdlib_json = Arc::new(build_stdlib_json());

        while !self.shutdown.load(Ordering::SeqCst) && !CTRL_C_SHUTDOWN.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let explain_json = explain_json.clone();
                    let stdlib_json = stdlib_json.clone();
                    thread::spawn(move || {
                        let _ =
                            handle_connection(stream, explain_json.as_str(), stdlib_json.as_str());
                    });
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(25));
                }
                Err(err) => {
                    return Err(format!("error: docs server accept failed: {}", err));
                }
            }
        }

        println!("\nDocs server stopped.");
        Ok(())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

#[derive(Clone, Copy, Serialize)]
struct StdlibParam {
    name: &'static str,
    ty: &'static str,
}

#[derive(Clone, Copy, Serialize)]
struct StdlibFunction {
    name: &'static str,
    signature: &'static str,
    params: &'static [StdlibParam],
    returns: &'static str,
    effects: &'static [&'static str],
}

#[derive(Clone, Copy, Serialize)]
struct StdlibModule {
    name: &'static str,
    functions: &'static [StdlibFunction],
}

#[derive(Serialize)]
struct StdlibCatalog<'a> {
    schema_version: &'static str,
    modules: &'a [StdlibModule],
}

const IO_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        name: "println",
        signature: "String -> Unit !Io",
        params: &[StdlibParam {
            name: "s",
            ty: "String",
        }],
        returns: "Unit",
        effects: &["Io"],
    },
    StdlibFunction {
        name: "print",
        signature: "String -> Unit !Io",
        params: &[StdlibParam {
            name: "s",
            ty: "String",
        }],
        returns: "Unit",
        effects: &["Io"],
    },
    StdlibFunction {
        name: "read_line",
        signature: "() -> String !Io",
        params: &[],
        returns: "String",
        effects: &["Io"],
    },
];

const LIST_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        name: "map",
        signature: "List<A> -> (A -> B) -> List<B>",
        params: &[
            StdlibParam {
                name: "xs",
                ty: "List<A>",
            },
            StdlibParam {
                name: "f",
                ty: "A -> B",
            },
        ],
        returns: "List<B>",
        effects: &[],
    },
    StdlibFunction {
        name: "filter",
        signature: "List<A> -> (A -> Bool) -> List<A>",
        params: &[
            StdlibParam {
                name: "xs",
                ty: "List<A>",
            },
            StdlibParam {
                name: "pred",
                ty: "A -> Bool",
            },
        ],
        returns: "List<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "fold",
        signature: "List<A> -> B -> (B -> A -> B) -> B",
        params: &[
            StdlibParam {
                name: "xs",
                ty: "List<A>",
            },
            StdlibParam {
                name: "init",
                ty: "B",
            },
            StdlibParam {
                name: "f",
                ty: "B -> A -> B",
            },
        ],
        returns: "B",
        effects: &[],
    },
    StdlibFunction {
        name: "first",
        signature: "List<A> -> Option<A>",
        params: &[StdlibParam {
            name: "xs",
            ty: "List<A>",
        }],
        returns: "Option<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "last",
        signature: "List<A> -> Option<A>",
        params: &[StdlibParam {
            name: "xs",
            ty: "List<A>",
        }],
        returns: "Option<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "length",
        signature: "List<A> -> Int",
        params: &[StdlibParam {
            name: "xs",
            ty: "List<A>",
        }],
        returns: "Int",
        effects: &[],
    },
    StdlibFunction {
        name: "concat",
        signature: "List<A> -> List<A> -> List<A>",
        params: &[
            StdlibParam {
                name: "left",
                ty: "List<A>",
            },
            StdlibParam {
                name: "right",
                ty: "List<A>",
            },
        ],
        returns: "List<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "range",
        signature: "Int -> Int -> List<Int>",
        params: &[
            StdlibParam {
                name: "start",
                ty: "Int",
            },
            StdlibParam {
                name: "end",
                ty: "Int",
            },
        ],
        returns: "List<Int>",
        effects: &[],
    },
    StdlibFunction {
        name: "take",
        signature: "List<A> -> Int -> List<A>",
        params: &[
            StdlibParam {
                name: "xs",
                ty: "List<A>",
            },
            StdlibParam {
                name: "n",
                ty: "Int",
            },
        ],
        returns: "List<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "drop",
        signature: "List<A> -> Int -> List<A>",
        params: &[
            StdlibParam {
                name: "xs",
                ty: "List<A>",
            },
            StdlibParam {
                name: "n",
                ty: "Int",
            },
        ],
        returns: "List<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "zip",
        signature: "List<A> -> List<B> -> List<{ left: A right: B }>",
        params: &[
            StdlibParam {
                name: "left",
                ty: "List<A>",
            },
            StdlibParam {
                name: "right",
                ty: "List<B>",
            },
        ],
        returns: "List<{ left: A right: B }>",
        effects: &[],
    },
    StdlibFunction {
        name: "join",
        signature: "List<String> -> String -> String",
        params: &[
            StdlibParam {
                name: "xs",
                ty: "List<String>",
            },
            StdlibParam {
                name: "sep",
                ty: "String",
            },
        ],
        returns: "String",
        effects: &[],
    },
    StdlibFunction {
        name: "sort",
        signature: "List<A> -> List<A>",
        params: &[StdlibParam {
            name: "xs",
            ty: "List<A>",
        }],
        returns: "List<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "find",
        signature: "List<A> -> (A -> Bool) -> Option<A>",
        params: &[
            StdlibParam {
                name: "xs",
                ty: "List<A>",
            },
            StdlibParam {
                name: "pred",
                ty: "A -> Bool",
            },
        ],
        returns: "Option<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "any",
        signature: "List<A> -> (A -> Bool) -> Bool",
        params: &[
            StdlibParam {
                name: "xs",
                ty: "List<A>",
            },
            StdlibParam {
                name: "pred",
                ty: "A -> Bool",
            },
        ],
        returns: "Bool",
        effects: &[],
    },
    StdlibFunction {
        name: "all",
        signature: "List<A> -> (A -> Bool) -> Bool",
        params: &[
            StdlibParam {
                name: "xs",
                ty: "List<A>",
            },
            StdlibParam {
                name: "pred",
                ty: "A -> Bool",
            },
        ],
        returns: "Bool",
        effects: &[],
    },
];

const OPTION_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        name: "unwrap_or",
        signature: "Option<A> -> A -> A",
        params: &[
            StdlibParam {
                name: "value",
                ty: "Option<A>",
            },
            StdlibParam {
                name: "default",
                ty: "A",
            },
        ],
        returns: "A",
        effects: &[],
    },
    StdlibFunction {
        name: "map",
        signature: "Option<A> -> (A -> B) -> Option<B>",
        params: &[
            StdlibParam {
                name: "value",
                ty: "Option<A>",
            },
            StdlibParam {
                name: "f",
                ty: "A -> B",
            },
        ],
        returns: "Option<B>",
        effects: &[],
    },
    StdlibFunction {
        name: "is_some",
        signature: "Option<A> -> Bool",
        params: &[StdlibParam {
            name: "value",
            ty: "Option<A>",
        }],
        returns: "Bool",
        effects: &[],
    },
    StdlibFunction {
        name: "is_none",
        signature: "Option<A> -> Bool",
        params: &[StdlibParam {
            name: "value",
            ty: "Option<A>",
        }],
        returns: "Bool",
        effects: &[],
    },
];

const STRING_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        name: "length",
        signature: "String -> Int",
        params: &[StdlibParam {
            name: "value",
            ty: "String",
        }],
        returns: "Int",
        effects: &[],
    },
    StdlibFunction {
        name: "concat",
        signature: "String -> String -> String",
        params: &[
            StdlibParam {
                name: "left",
                ty: "String",
            },
            StdlibParam {
                name: "right",
                ty: "String",
            },
        ],
        returns: "String",
        effects: &[],
    },
    StdlibFunction {
        name: "slice",
        signature: "String -> Int -> Int -> String",
        params: &[
            StdlibParam {
                name: "value",
                ty: "String",
            },
            StdlibParam {
                name: "start",
                ty: "Int",
            },
            StdlibParam {
                name: "end",
                ty: "Int",
            },
        ],
        returns: "String",
        effects: &[],
    },
    StdlibFunction {
        name: "char_at",
        signature: "String -> Int -> Option<String>",
        params: &[
            StdlibParam {
                name: "value",
                ty: "String",
            },
            StdlibParam {
                name: "index",
                ty: "Int",
            },
        ],
        returns: "Option<String>",
        effects: &[],
    },
    StdlibFunction {
        name: "contains",
        signature: "String -> String -> Bool",
        params: &[
            StdlibParam {
                name: "value",
                ty: "String",
            },
            StdlibParam {
                name: "needle",
                ty: "String",
            },
        ],
        returns: "Bool",
        effects: &[],
    },
    StdlibFunction {
        name: "split",
        signature: "String -> String -> List<String>",
        params: &[
            StdlibParam {
                name: "value",
                ty: "String",
            },
            StdlibParam {
                name: "sep",
                ty: "String",
            },
        ],
        returns: "List<String>",
        effects: &[],
    },
    StdlibFunction {
        name: "trim",
        signature: "String -> String",
        params: &[StdlibParam {
            name: "value",
            ty: "String",
        }],
        returns: "String",
        effects: &[],
    },
    StdlibFunction {
        name: "starts_with",
        signature: "String -> String -> Bool",
        params: &[
            StdlibParam {
                name: "value",
                ty: "String",
            },
            StdlibParam {
                name: "prefix",
                ty: "String",
            },
        ],
        returns: "Bool",
        effects: &[],
    },
    StdlibFunction {
        name: "ends_with",
        signature: "String -> String -> Bool",
        params: &[
            StdlibParam {
                name: "value",
                ty: "String",
            },
            StdlibParam {
                name: "suffix",
                ty: "String",
            },
        ],
        returns: "Bool",
        effects: &[],
    },
];

const STREAM_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        name: "of",
        signature: "List<A> -> Stream<A>",
        params: &[StdlibParam {
            name: "xs",
            ty: "List<A>",
        }],
        returns: "Stream<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "from",
        signature: "List<A> -> Stream<A>",
        params: &[StdlibParam {
            name: "xs",
            ty: "List<A>",
        }],
        returns: "Stream<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "gen",
        signature: "A -> (A -> A) -> Stream<A>",
        params: &[
            StdlibParam {
                name: "seed",
                ty: "A",
            },
            StdlibParam {
                name: "step",
                ty: "A -> A",
            },
        ],
        returns: "Stream<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "map",
        signature: "Stream<A> -> (A -> B) -> Stream<B>",
        params: &[
            StdlibParam {
                name: "stream",
                ty: "Stream<A>",
            },
            StdlibParam {
                name: "f",
                ty: "A -> B",
            },
        ],
        returns: "Stream<B>",
        effects: &[],
    },
    StdlibFunction {
        name: "filter",
        signature: "Stream<A> -> (A -> Bool) -> Stream<A>",
        params: &[
            StdlibParam {
                name: "stream",
                ty: "Stream<A>",
            },
            StdlibParam {
                name: "pred",
                ty: "A -> Bool",
            },
        ],
        returns: "Stream<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "take",
        signature: "Stream<A> -> Int -> List<A>",
        params: &[
            StdlibParam {
                name: "stream",
                ty: "Stream<A>",
            },
            StdlibParam {
                name: "n",
                ty: "Int",
            },
        ],
        returns: "List<A>",
        effects: &[],
    },
    StdlibFunction {
        name: "to_list",
        signature: "Stream<A> -> List<A>",
        params: &[StdlibParam {
            name: "stream",
            ty: "Stream<A>",
        }],
        returns: "List<A>",
        effects: &[],
    },
];

const DEBUG_FUNCTIONS: &[StdlibFunction] = &[StdlibFunction {
    name: "show",
    signature: "A -> String",
    params: &[StdlibParam {
        name: "value",
        ty: "A",
    }],
    returns: "String",
    effects: &[],
}];

const STDLIB_CATALOG: &[StdlibModule] = &[
    StdlibModule {
        name: "IO",
        functions: IO_FUNCTIONS,
    },
    StdlibModule {
        name: "List",
        functions: LIST_FUNCTIONS,
    },
    StdlibModule {
        name: "Option",
        functions: OPTION_FUNCTIONS,
    },
    StdlibModule {
        name: "String",
        functions: STRING_FUNCTIONS,
    },
    StdlibModule {
        name: "Stream",
        functions: STREAM_FUNCTIONS,
    },
    StdlibModule {
        name: "Debug",
        functions: DEBUG_FUNCTIONS,
    },
];

pub fn build_stdlib_json() -> String {
    let catalog = StdlibCatalog {
        schema_version: "3.1",
        modules: STDLIB_CATALOG,
    };
    serde_json::to_string(&catalog).expect("stdlib catalog json")
}

fn handle_connection(
    mut stream: TcpStream,
    explain_json: &str,
    stdlib_json: &str,
) -> std::io::Result<()> {
    let mut request_line = String::new();
    {
        let mut reader = BufReader::new(&mut stream);
        reader.read_line(&mut request_line)?;
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");

    if method != "GET" {
        write_response(
            &mut stream,
            405,
            "text/plain; charset=utf-8",
            "Method Not Allowed",
        )?;
        return Ok(());
    }

    match path {
        "/" => write_response(&mut stream, 200, "text/html; charset=utf-8", INDEX_HTML)?,
        "/api/explain" => write_response(
            &mut stream,
            200,
            "application/json; charset=utf-8",
            explain_json,
        )?,
        "/api/stdlib" => write_response(
            &mut stream,
            200,
            "application/json; charset=utf-8",
            stdlib_json,
        )?,
        "/static/app.js" => write_response(
            &mut stream,
            200,
            "application/javascript; charset=utf-8",
            APP_JS,
        )?,
        "/static/style.css" => {
            write_response(&mut stream, 200, "text/css; charset=utf-8", STYLE_CSS)?
        }
        _ => write_response(&mut stream, 404, "text/plain; charset=utf-8", "Not Found")?,
    }

    Ok(())
}

fn write_response(
    stream: &mut TcpStream,
    status_code: u16,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    let status_text = match status_code {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    };
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status_code,
        status_text,
        content_type,
        body.len(),
        body
    )?;
    stream.flush()
}

fn install_ctrlc_handler() {
    #[cfg(unix)]
    unsafe {
        install_ctrlc_handler_unix();
    }
    #[cfg(windows)]
    unsafe {
        install_ctrlc_handler_windows();
    }
}

#[cfg(unix)]
unsafe fn install_ctrlc_handler_unix() {
    unsafe extern "C" fn handle_sigint(_signal: i32) {
        CTRL_C_SHUTDOWN.store(true, Ordering::SeqCst);
    }

    unsafe extern "C" {
        fn signal(sig: i32, handler: usize) -> usize;
    }

    const SIGINT: i32 = 2;
    let _ = signal(SIGINT, handle_sigint as usize);
}

#[cfg(windows)]
unsafe fn install_ctrlc_handler_windows() {
    unsafe extern "system" fn handle_ctrl(ctrl_type: u32) -> i32 {
        const CTRL_C_EVENT: u32 = 0;
        if ctrl_type == CTRL_C_EVENT {
            CTRL_C_SHUTDOWN.store(true, Ordering::SeqCst);
            1
        } else {
            0
        }
    }

    unsafe extern "system" {
        fn SetConsoleCtrlHandler(
            handler: Option<unsafe extern "system" fn(u32) -> i32>,
            add: i32,
        ) -> i32;
    }

    let _ = unsafe { SetConsoleCtrlHandler(Some(handle_ctrl), 1) };
}
