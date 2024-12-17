use std::{
    convert::Infallible,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::Arc,
};

use axum::{
    extract::State,
    response::{sse::Event, IntoResponse, Sse},
    routing::get,
    Router,
};
use clap::Parser;
use futures::stream::Stream;
use tempdir::TempDir;
use tokio::{
    net::TcpListener,
    sync::broadcast::{self, error::SendError, Sender},
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::services::ServeFile;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Path to source file.
    input: String,

    /// Hostname or IP address of the webserver.
    #[arg(long, default_value = "localhost")]
    host: String,

    /// Port number of the webserver.
    /// Random port is used if not specified.
    #[arg(long, default_value_t = 0)]
    port: u16,

    #[arg(/*trailing_var_arg = true,*/ allow_hyphen_values = true, hide = true)]
    inner: Vec<String>,
}

#[derive(Clone)]
struct AppState {
    tx: Arc<Sender<&'static str>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (tx, _) = broadcast::channel(8);
    let tx = Arc::new(tx);

    let temp_dir = TempDir::new("typst-build").unwrap();
    let preview_file_path = temp_dir.path().join("preview.pdf");

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/notifications", get(notification_handler))
        .route_service("/preview.pdf", ServeFile::new(&preview_file_path))
        .with_state(AppState { tx: tx.clone() });

    let axum_task = tokio::spawn(async move {
        let listener = TcpListener::bind(format!("{}:{}", args.host, args.port))
            .await
            .unwrap();

        let addr = listener.local_addr().unwrap();

        let host = if addr.ip().is_loopback() {
            "localhost".to_string()
        } else if addr.ip().is_unspecified() {
            hostname::get().unwrap().into_string().unwrap()
        } else {
            addr.ip().to_string()
        };

        println!("Open http://{}:{} in your browser.", host, addr.port());

        axum::serve(listener, app).await.unwrap();
    });

    let mut cmd = Command::new("typst")
        .stderr(Stdio::piped())
        .arg("watch")
        .args(args.inner)
        .arg(args.input)
        .arg(&preview_file_path)
        .spawn()
        .unwrap();

    let stdout = cmd.stderr.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);

    for line in stdout_reader.lines() {
        let line = line.unwrap();
        if line.contains("compiled successfully") {
            match tx.send("update") {
                Ok(_) => {}
                Err(SendError(_)) => {} // -> no active receivers
            };
        } else if line.starts_with("error:") {
            eprintln!("{}", line);
        }
    }

    cmd.wait().unwrap();
    axum_task.await.unwrap();
}

async fn index_handler() -> impl IntoResponse {
    let content = include_bytes!("../web/dist/index.html");
    ([("Content-Type", "text/html;charset=utf-8")], content)
}

async fn notification_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).map(|ev| Ok(Event::default().data(ev.unwrap())));
    Sse::new(stream)
}
