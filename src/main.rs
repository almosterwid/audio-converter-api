use axum::{
    extract::Multipart,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
    use tokio::{fs::File, io::AsyncWriteExt, net::TcpListener};
use tokio::process::Command;

async fn api_root() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Ruby Chan Multi-Format Converter",
        "hint": "POST /api/audio/convert → file + format (mp3,m4a,ogg,flac,aac,wav)"
    }))
}

async fn audio_convert(mut multipart: Multipart) -> Json<Value> {
    let mut input_path: Option<String> = None;
    let mut target_format = "wav".to_string();

    // ---------------------------
    // FIXED MULTIPART LOOP
    // ---------------------------
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("");

        // FILE UPLOAD
        if name == "file" {
            let temp_name = format!("upload_{}", chrono::Utc::now().timestamp_millis());
            let mut file = File::create(&temp_name).await.unwrap();

            let mut stream = field;

            // FIXED CHUNK LOOP
            while let Ok(Some(chunk)) = stream.chunk().await {
                let _ = file.write_all(&chunk).await;
            }

            file.flush().await.unwrap();
            input_path = Some(temp_name);
        }

        // FORMAT FIELD
        else if name == "format" {
            if let Ok(text) = field.text().await {
                let f = text.trim().to_lowercase();
                if ["wav", "mp3", "m4a", "ogg", "flac", "aac"].contains(&f.as_str()) {
                    target_format = f;
                }
            }
        }
    }

    // NO FILE → RETURN ERROR
    let input = match input_path {
        Some(path) => path,
        None => {
            return Json(json!({
                "status": "error",
                "message": "No file uploaded"
            }))
        }
    };

    let output = format!("converted.{}", target_format);

    // RUN FFMPEG
    let success = Command::new("ffmpeg")
        .args(["-y", "-i", &input, &output])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false);

    // CLEAN TEMP INPUT
    let _ = tokio::fs::remove_file(&input).await;

    // SUCCESS RESPONSE
    if success {
        let size = std::fs::metadata(&output)
            .ok()
            .map(|m| m.len());

        Json(json!({
            "status": "success",
            "message": format!("Converted to {target_format}!"),
            "output": output,
            "size_bytes": size
        }))
    } else {
        Json(json!({
            "status": "failed",
            "message": "Conversion failed"
        }))
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api", get(api_root))
        .route("/api/audio/convert", post(audio_convert))
        .layer(axum::extract::DefaultBodyLimit::disable());

    println!("Ruby Chan Multi-Format Converter Running → http://localhost:3000/api");

    axum::serve(
        TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        app,
    )
    .await
    .unwrap();
}
