use axum::{
    extract::Json,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Write, Read, Seek, SeekFrom, Cursor};
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber;
use uuid::Uuid;
use zip::{ZipWriter, ZipArchive, write::FileOptions, CompressionMethod};
use tempfile::NamedTempFile;
use percent_encoding;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub project_name: String,
    pub project_description: String,
}

#[derive(Debug, Serialize)]
pub struct TemplateData {
    pub username: String,
    pub email: String,
    pub project_name: String,
    pub project_description: String,
    pub generated_id: String,
    pub timestamp: String,
}

impl From<UserInfo> for TemplateData {
    fn from(user_info: UserInfo) -> Self {
        TemplateData {
            username: user_info.username,
            email: user_info.email,
            project_name: user_info.project_name,
            project_description: user_info.project_description,
            generated_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        }
    }
}



// Helper function to fill template content with user data
fn fill_template_content(content: &str, data: &TemplateData) -> String {
    content
        .replace("{{username}}", &data.username)
        .replace("{{email}}", &data.email)
        .replace("{{project_name}}", &data.project_name)
        .replace("{{project_description}}", &data.project_description)
}

// Create server zip file with filled templates
fn create_server_zip(data: &TemplateData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    println!("[DEBUG] Starting server zip creation...");
    let zero_zip_path = "templates/server/zero.zip";
    
    // Check if file exists before reading
    if !std::path::Path::new(zero_zip_path).exists() {
        let error_msg = format!("File not found: {}", zero_zip_path);
        eprintln!("[ERROR] {}", error_msg);
        return Err(error_msg.into());
    }
    
    println!("[DEBUG] Reading zero.zip from: {}", zero_zip_path);
    // Read existing zero.zip
    let zero_zip_data = fs::read(zero_zip_path).map_err(|e| {
        let error_msg = format!("Failed to read {}: {}", zero_zip_path, e);
        eprintln!("[ERROR] {}", error_msg);
        error_msg
    })?;
    
    let mut temp_file = NamedTempFile::new()?;
    
    {
        let mut zip = ZipWriter::new(&mut temp_file);
        let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);
        
        // Copy existing zero.zip contents first
        let cursor = Cursor::new(zero_zip_data);
        let mut archive = ZipArchive::new(cursor)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            
            zip.start_file(&name, options)?;
            let mut buffer = Vec::new();
            std::io::copy(&mut file, &mut buffer)?;
            zip.write_all(&buffer)?;
        }
        
        // Add filled template files
        let license_path = "templates/server/LICENSE";
        if !std::path::Path::new(license_path).exists() {
            let error_msg = format!("File not found: {}", license_path);
            eprintln!("[ERROR] {}", error_msg);
            return Err(error_msg.into());
        }
        let license_content = fs::read_to_string(license_path)?;
        let filled_license = fill_template_content(&license_content, data);
        zip.start_file("LICENSE", options)?;
        zip.write_all(filled_license.as_bytes())?;

        let pyproject_path = "templates/server/pyproject.toml";
        if !std::path::Path::new(pyproject_path).exists() {
            let error_msg = format!("File not found: {}", pyproject_path);
            eprintln!("[ERROR] {}", error_msg);
            return Err(error_msg.into());
        }
        let pyproject_content = fs::read_to_string(pyproject_path)?;
        let filled_pyproject = fill_template_content(&pyproject_content, data);
        zip.start_file("pyproject.toml", options)?;
        zip.write_all(filled_pyproject.as_bytes())?;

        let readme_path = "templates/server/README.md";
        if !std::path::Path::new(readme_path).exists() {
            let error_msg = format!("File not found: {}", readme_path);
            eprintln!("[ERROR] {}", error_msg);
            return Err(error_msg.into());
        }
        let readme_content = fs::read_to_string(readme_path)?;
        let filled_readme = fill_template_content(&readme_content, data);
        zip.start_file("README.md", options)?;
        zip.write_all(filled_readme.as_bytes())?;

        zip.finish()?;
    }
    
    let mut buffer = Vec::new();
    temp_file.seek(SeekFrom::Start(0))?;
    temp_file.read_to_end(&mut buffer)?;
    println!("[DEBUG] Server zip created successfully, size: {} bytes", buffer.len());
    Ok(buffer)
}

// Create client zip file with filled templates  
fn create_client_zip(data: &TemplateData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    println!("[DEBUG] Starting client zip creation...");
    let zero_client_zip_path = "templates/client/zero-client.zip";
    
    // Check if file exists before reading
    if !std::path::Path::new(zero_client_zip_path).exists() {
        let error_msg = format!("File not found: {}", zero_client_zip_path);
        eprintln!("[ERROR] {}", error_msg);
        return Err(error_msg.into());
    }
    
    println!("[DEBUG] Reading zero-client.zip from: {}", zero_client_zip_path);
    // Read existing zero-client.zip
    let zero_client_zip_data = fs::read(zero_client_zip_path).map_err(|e| {
        let error_msg = format!("Failed to read {}: {}", zero_client_zip_path, e);
        eprintln!("[ERROR] {}", error_msg);
        error_msg
    })?;
    
    let mut temp_file = NamedTempFile::new()?;
    
    {
        let mut zip = ZipWriter::new(&mut temp_file);
        let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);
        
        // Copy existing zero-client.zip contents first
        let cursor = Cursor::new(zero_client_zip_data);
        let mut archive = ZipArchive::new(cursor)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            
            zip.start_file(&name, options)?;
            let mut buffer = Vec::new();
            std::io::copy(&mut file, &mut buffer)?;
            zip.write_all(&buffer)?;
        }
        
        // Add filled template files
        let license_path = "templates/client/LICENSE";
        if !std::path::Path::new(license_path).exists() {
            let error_msg = format!("File not found: {}", license_path);
            eprintln!("[ERROR] {}", error_msg);
            return Err(error_msg.into());
        }
        let license_content = fs::read_to_string(license_path)?;
        let filled_license = fill_template_content(&license_content, data);
        zip.start_file("LICENSE", options)?;
        zip.write_all(filled_license.as_bytes())?;

        let package_path = "templates/client/package.json";
        if !std::path::Path::new(package_path).exists() {
            let error_msg = format!("File not found: {}", package_path);
            eprintln!("[ERROR] {}", error_msg);
            return Err(error_msg.into());
        }
        let package_content = fs::read_to_string(package_path)?;
        let filled_package = fill_template_content(&package_content, data);
        zip.start_file("package.json", options)?;
        zip.write_all(filled_package.as_bytes())?;

        let readme_path = "templates/client/README.md";
        if !std::path::Path::new(readme_path).exists() {
            let error_msg = format!("File not found: {}", readme_path);
            eprintln!("[ERROR] {}", error_msg);
            return Err(error_msg.into());
        }
        let readme_content = fs::read_to_string(readme_path)?;
        let filled_readme = fill_template_content(&readme_content, data);
        zip.start_file("README.md", options)?;
        zip.write_all(filled_readme.as_bytes())?;

        zip.finish()?;
    }
    
    let mut buffer = Vec::new();
    temp_file.seek(SeekFrom::Start(0))?;
    temp_file.read_to_end(&mut buffer)?;
    println!("[DEBUG] Client zip created successfully, size: {} bytes", buffer.len());
    Ok(buffer)
}

// Health check endpoint
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "rust-template-generator"
    }))
}

// Serve the main form page
async fn index() -> impl IntoResponse {
    let html = include_str!("../static/index.html");
    Html(html)
}

// Generate server zip file endpoint
async fn generate_server_zip(
    Json(user_info): Json<UserInfo>,
) -> impl IntoResponse {
    println!("[DEBUG] Received request to generate server zip for user: {}", user_info.username);
    let template_data: TemplateData = user_info.into();
    
    match create_server_zip(&template_data) {
        Ok(zip_data) => {
            let filename = format!("{}.zip", 
                template_data.project_name.replace(" ", "_").to_lowercase()
            );
            
            println!("[DEBUG] Successfully created server zip: {}, size: {} bytes", filename, zip_data.len());
            
            // Use RFC 5987 encoding for international filenames
            let encoded_filename = percent_encoding::utf8_percent_encode(
                &filename, 
                percent_encoding::NON_ALPHANUMERIC
            ).to_string();
            
            let headers = [
                (header::CONTENT_TYPE, "application/zip"),
                (header::CONTENT_DISPOSITION, &format!("attachment; filename*=UTF-8''{}", encoded_filename)),
            ];
            
            (StatusCode::OK, headers, zip_data).into_response()
        }
        Err(e) => {
            eprintln!("[ERROR] Server zip creation error: {}", e);
            println!("[ERROR] Full error details: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Failed to create server zip file: {}", e)
            }))).into_response()
        }
    }
}

// Generate client zip file endpoint
async fn generate_client_zip(
    Json(user_info): Json<UserInfo>,
) -> impl IntoResponse {
    println!("[DEBUG] Received request to generate client zip for user: {}", user_info.username);
    let template_data: TemplateData = user_info.into();
    
    match create_client_zip(&template_data) {
        Ok(zip_data) => {
            let filename = format!("{}-client.zip", 
                template_data.project_name.replace(" ", "_").to_lowercase()
            );
            
            println!("[DEBUG] Successfully created client zip: {}, size: {} bytes", filename, zip_data.len());
            
            // Use RFC 5987 encoding for international filenames
            let encoded_filename = percent_encoding::utf8_percent_encode(
                &filename, 
                percent_encoding::NON_ALPHANUMERIC
            ).to_string();
            
            let headers = [
                (header::CONTENT_TYPE, "application/zip"),
                (header::CONTENT_DISPOSITION, &format!("attachment; filename*=UTF-8''{}", encoded_filename)),
            ];
            
            (StatusCode::OK, headers, zip_data).into_response()
        }
        Err(e) => {
            eprintln!("[ERROR] Client zip creation error: {}", e);
            println!("[ERROR] Full error details: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Failed to create client zip file: {}", e)
            }))).into_response()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Print debugging information
    println!("[DEBUG] ============ Starting ZeroHub Server ============");
    
    // Print current working directory
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    println!("[DEBUG] Current working directory: {:?}", current_dir);
    
    // Check if template directories exist
    let templates_dir = std::path::Path::new("templates");
    let server_dir = std::path::Path::new("templates/server");
    let client_dir = std::path::Path::new("templates/client");
    let static_dir = std::path::Path::new("static");
    
    println!("[DEBUG] Checking template directories:");
    println!("[DEBUG] - templates/ exists: {}", templates_dir.exists());
    println!("[DEBUG] - templates/server/ exists: {}", server_dir.exists());
    println!("[DEBUG] - templates/client/ exists: {}", client_dir.exists());
    println!("[DEBUG] - static/ exists: {}", static_dir.exists());
    
    // Check specific template files
    let files_to_check = [
        "templates/server/zero.zip",
        "templates/server/LICENSE",
        "templates/server/pyproject.toml",
        "templates/server/README.md",
        "templates/client/zero-client.zip",
        "templates/client/LICENSE",
        "templates/client/package.json",
        "templates/client/README.md",
        "static/index.html",
    ];
    
    println!("[DEBUG] Checking template files:");
    for file_path in &files_to_check {
        let exists = std::path::Path::new(file_path).exists();
        println!("[DEBUG] - {} exists: {}", file_path, exists);
    }
    
    println!("[DEBUG] ===============================================");

    // Build the router
    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/generate-server-zip", post(generate_server_zip))
        .route("/generate-client-zip", post(generate_client_zip))
        .nest_service("/static", ServeDir::new("./static"))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
        );

    println!("🚀 Server starting at http://localhost:8080");

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}