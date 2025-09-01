use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware::Logger};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Write, Read, Seek, SeekFrom, Cursor};
use std::path::Path;
use uuid::Uuid;
use zip::{ZipWriter, ZipArchive, write::FileOptions, CompressionMethod};
use tempfile::NamedTempFile;

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

pub struct AppState {
    pub handlebars: Handlebars<'static>,
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
    let zero_zip_path = "templates/server/zero.zip";
    
    // Read existing zero.zip
    let zero_zip_data = fs::read(zero_zip_path)?;
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
        let license_content = fs::read_to_string("templates/server/LICENSE")?;
        let filled_license = fill_template_content(&license_content, data);
        zip.start_file("LICENSE", options)?;
        zip.write_all(filled_license.as_bytes())?;

        let pyproject_content = fs::read_to_string("templates/server/pyproject.toml")?;
        let filled_pyproject = fill_template_content(&pyproject_content, data);
        zip.start_file("pyproject.toml", options)?;
        zip.write_all(filled_pyproject.as_bytes())?;

        let readme_content = fs::read_to_string("templates/server/README.md")?;
        let filled_readme = fill_template_content(&readme_content, data);
        zip.start_file("README.md", options)?;
        zip.write_all(filled_readme.as_bytes())?;

        zip.finish()?;
    }
    
    let mut buffer = Vec::new();
    temp_file.seek(SeekFrom::Start(0))?;
    temp_file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

// Create client zip file with filled templates  
fn create_client_zip(data: &TemplateData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let zero_client_zip_path = "templates/client/zero-client.zip";
    
    // Read existing zero-client.zip
    let zero_client_zip_data = fs::read(zero_client_zip_path)?;
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
        let license_content = fs::read_to_string("templates/client/LICENSE")?;
        let filled_license = fill_template_content(&license_content, data);
        zip.start_file("LICENSE", options)?;
        zip.write_all(filled_license.as_bytes())?;

        let package_content = fs::read_to_string("templates/client/package.json")?;
        let filled_package = fill_template_content(&package_content, data);
        zip.start_file("package.json", options)?;
        zip.write_all(filled_package.as_bytes())?;

        let readme_content = fs::read_to_string("templates/client/README.md")?;
        let filled_readme = fill_template_content(&readme_content, data);
        zip.start_file("README.md", options)?;
        zip.write_all(filled_readme.as_bytes())?;

        zip.finish()?;
    }
    
    let mut buffer = Vec::new();
    temp_file.seek(SeekFrom::Start(0))?;
    temp_file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

// Health check endpoint
async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "rust-template-generator"
    })))
}

// Serve the main form page
async fn index() -> Result<HttpResponse> {
    let html = include_str!("../static/index.html");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

// Generate server zip file endpoint
async fn generate_server_zip(
    user_info: web::Json<UserInfo>,
) -> Result<HttpResponse> {
    let template_data: TemplateData = user_info.into_inner().into();
    
    match create_server_zip(&template_data) {
        Ok(zip_data) => {
            let filename = format!("{}.zip", 
                template_data.project_name.replace(" ", "_").to_lowercase()
            );
            
            Ok(HttpResponse::Ok()
                .insert_header(("Content-Type", "application/zip"))
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .body(zip_data))
        }
        Err(e) => {
            eprintln!("Server zip creation error: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create server zip file"
            })))
        }
    }
}

// Generate client zip file endpoint
async fn generate_client_zip(
    user_info: web::Json<UserInfo>,
) -> Result<HttpResponse> {
    let template_data: TemplateData = user_info.into_inner().into();
    
    match create_client_zip(&template_data) {
        Ok(zip_data) => {
            let filename = format!("{}-client.zip", 
                template_data.project_name.replace(" ", "_").to_lowercase()
            );
            
            Ok(HttpResponse::Ok()
                .insert_header(("Content-Type", "application/zip"))
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .body(zip_data))
        }
        Err(e) => {
            eprintln!("Client zip creation error: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create client zip file"
            })))
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let app_state = web::Data::new(AppState { 
        handlebars: Handlebars::new() 
    });

    println!("🚀 Server starting at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health))
            .route("/generate-server-zip", web::post().to(generate_server_zip))
            .route("/generate-client-zip", web::post().to(generate_client_zip))
            .service(Files::new("/static", "./static").prefer_utf8(true))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}