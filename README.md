# Claude Certified Architect Learning Platform

Self-hosted course platform for learning Claude API architecture patterns.

## Quick Start

### Run locally
```bash
cargo run
# → http://localhost:8080
```

### Docker
```bash
docker build -t architect-platform .
docker run -p 8080:8080 architect-platform
```

## Content Format

Edit `guide_en.MD` to customize course content:

```markdown
# Chapter 1: Getting Started

## 1.1 Introduction
Your content here...

## 1.2 Next Section
More content...
```

- `# Chapter X: Title` → creates a chapter
- `## X.Y Title` → creates a section

## API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/course` | GET | Full course content |
| `/api/progress` | GET | User progress |
| `/api/progress/:id` | POST | Toggle section completion |
| `/api/health` | GET | Health check |

## Tech Stack

- **Backend**: Rust + Axum
- **Frontend**: Vanilla JS SPA
- **Storage**: Local JSON file
- **Design**: Minimal Brutalist (dark)