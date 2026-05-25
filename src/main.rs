use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub title: String,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProgress {
    pub completed_sections: Vec<String>,
}

struct AppState {
    course: Course,
    progress: Mutex<UserProgress>,
    progress_path: PathBuf,
}

static CHAPTER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^# Chapter\s+(\d+):\s+(.+)$").unwrap()
});

static SECTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^##\s+(\d+\.\d+)\s+(.+)$").unwrap()
});

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn parse_course(markdown: &str) -> Course {
    let mut course = Course {
        title: "Claude Certified Architect Guide".to_string(),
        chapters: Vec::new(),
    };

    let lines: Vec<&str> = markdown.lines().collect();
    let mut current_chapter: Option<Chapter> = None;
    let mut current_section: Option<Section> = None;
    let mut current_content: Vec<String> = Vec::new();

    let process_section = |current_section: &mut Option<Section>, content: &mut Vec<String>| {
        if let Some(mut section) = current_section.take() {
            section.content = content.join("\n").trim().to_string();
            if let Some(ref mut chapter) = current_chapter {
                chapter.sections.push(section);
            }
        }
        content.clear();
    };

    let process_chapter = |current_chapter: &mut Option<Chapter>,
                           current_section: &mut Option<Section>,
                           content: &mut Vec<String>| {
        process_section(current_section, content);
        if let Some(chapter) = current_chapter.take() {
            course.chapters.push(chapter);
        }
    };

    for line in lines {
        if let Some(caps) = CHAPTER_REGEX.captures(line) {
            process_chapter(&mut current_chapter, &mut current_section, &mut current_content);
            let chapter_num = caps.get(1).map(|m| m.as_str()).unwrap_or("0");
            let chapter_title = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            current_chapter = Some(Chapter {
                id: format!("chapter-{}", slugify(chapter_num)),
                title: format!("Chapter {}: {}", chapter_num, chapter_title),
                sections: Vec::new(),
            });
        } else if let Some(caps) = SECTION_REGEX.captures(line) {
            process_section(&mut current_section, &mut current_content);
            let section_id = caps.get(1).map(|m| m.as_str()).unwrap_or("0.0");
            let section_title = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            current_section = Some(Section {
                id: slugify(section_id),
                title: section_title.to_string(),
                content: String::new(),
            });
        } else {
            current_content.push(line.to_string());
        }
    }

    process_chapter(&mut current_chapter, &mut current_section, &mut current_content);
    course
}

fn load_progress(path: &PathBuf) -> UserProgress {
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(e) => {
                error!("Failed to read progress file: {}", e);
                UserProgress::default()
            }
        }
    } else {
        UserProgress::default()
    }
}

fn save_progress(path: &PathBuf, progress: &UserProgress) -> Result<(), String> {
    let content = serde_json::to_string_pretty(progress).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

async fn get_course(State(state): State<AppState>) -> Json<Course> {
    Json(state.course.clone())
}

async fn get_progress(State(state): State<AppState>) -> Json<UserProgress> {
    let progress = state.progress.lock().unwrap();
    Json(progress.clone())
}

async fn toggle_section(
    State(state): State<AppState>,
    Path(section_id): Path<String>,
) -> Result<Json<UserProgress>, (StatusCode, String)> {
    let mut progress = state.progress.lock().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let completed_set: HashSet<_> = progress.completed_sections.iter().cloned().collect();
    let mut is_now_completed = false;

    if completed_set.contains(&section_id) {
        progress.completed_sections.retain(|id| id != &section_id);
    } else {
        progress.completed_sections.push(section_id.clone());
        is_now_completed = true;
    }

    save_progress(&state.progress_path, &progress).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    info!(
        "Section '{}' marked as {}",
        section_id,
        if is_now_completed { "completed" } else { "incomplete" }
    );

    Ok(Json(progress.clone()))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let guide_path = PathBuf::from("guide_en.MD");
    let markdown = fs::read_to_string(&guide_path).expect("Failed to read guide_en.MD");

    let course = parse_course(&markdown);
    info!("Parsed {} chapters", course.chapters.len());

    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude-architect-platform");
    fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    let progress_path = data_dir.join("progress.json");
    let progress = load_progress(&progress_path);
    info!(
        "Loaded {} completed sections",
        progress.completed_sections.len()
    );

    let state = AppState {
        course,
        progress: Mutex::new(progress),
        progress_path,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/course", get(get_course))
        .route("/api/progress", get(get_progress))
        .route("/api/progress/:section_id", post(toggle_section))
        .route("/api/health", get(|| async { (StatusCode::OK, "OK") }))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("Server running on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}