import { QUESTIONS } from './questions.js';
import express from 'express';
import cors from 'cors';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const app = express();
const PORT = 8080;

app.use(cors());
app.use(express.json());

// Data models
let course = { title: 'Claude Certified Architect Guide', chapters: [], parts: [] };
let quizData = { questions: QUESTIONS };
let progress = { completed_sections: [], quiz_scores: [] };
const dataDir = path.join(__dirname, 'data');
const progressPath = path.join(dataDir, 'progress.json');

// Regex patterns
const CHAPTER_REGEX = /^# Chapter\s+(\d+):\s+(.+)$/;
const SECTION_REGEX = /^##\s+(\d+\.\d+)\s+(.+)$/;
const PART_REGEX = /^# PART [IVX]+:/;

function slugify(text) {
  return text.toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '');
}

function parseCourse(markdown) {
  const result = { 
    title: 'Claude Certified Architect Guide', 
    parts: [],
    chapters: [] 
  };
  const lines = markdown.split('\n');
  
  let currentPart = null;
  let currentChapter = null;
  let currentSection = null;
  let currentContent = [];
  let currentPartNum = 0;
  let currentChapterNum = 0;

  const processSection = () => {
    if (currentSection) {
      currentSection.content = currentContent.join('\n').trim();
      if (currentChapter) {
        currentChapter.sections.push(currentSection);
      }
    }
    currentContent = [];
    currentSection = null;
  };

  const processChapter = () => {
    processSection();
    if (currentChapter) {
      if (currentPart) {
        currentPart.chapters.push(currentChapter);
      } else {
        result.chapters.push(currentChapter);
      }
    }
    currentChapter = null;
  };

  for (const line of lines) {
    const partMatch = line.match(/^# PART ([IVXLC]+):\s*(.+)$/);
    if (partMatch) {
      if (currentPart && currentPart.chapters.length > 0) {
        result.parts.push(currentPart);
      }
      currentPartNum++;
      currentPart = {
        id: `part-${currentPartNum}`,
        title: `${partMatch[1]}: ${partMatch[2].trim()}`,
        chapters: []
      };
      continue;
    }

    const chapterMatch = CHAPTER_REGEX.exec(line);
    if (chapterMatch) {
      processChapter();
      currentChapterNum++;
      currentChapter = {
        id: `chapter-${slugify(chapterMatch[1])}`,
        title: `Chapter ${chapterMatch[1]}: ${chapterMatch[2].trim()}`,
        sections: []
      };
      continue;
    }

    const sectionMatch = SECTION_REGEX.exec(line);
    if (sectionMatch) {
      processSection();
      currentSection = {
        id: slugify(sectionMatch[1]),
        title: sectionMatch[2].trim(),
        content: ''
      };
      continue;
    }

    currentContent.push(line);
  }

  processChapter();
  if (currentPart && currentPart.chapters.length > 0) {
    result.parts.push(currentPart);
  }
  return result;
}

function parseQuiz() {
  // Questions will be loaded from quiz.js
}

function loadProgress() {
  if (!fs.existsSync(dataDir)) {
    fs.mkdirSync(dataDir, { recursive: true });
  }
  if (fs.existsSync(progressPath)) {
    try {
      progress = JSON.parse(fs.readFileSync(progressPath, 'utf8'));
    } catch (e) {
      console.error('Failed to load progress:', e);
    }
  }
}

function saveProgress() {
  fs.writeFileSync(progressPath, JSON.stringify(progress, null, 2));
}

// Initialize
const markdown = fs.readFileSync(path.join(__dirname, 'guide_en.MD'), 'utf8');
course = parseCourse(markdown);
console.log(`Parsed ${course.parts.length} parts, ${course.chapters.length + course.parts.reduce((a,p)=>a+p.chapters.length,0)} chapters`);

loadProgress();

// API routes
app.get('/api/course', (req, res) => res.json(course));
app.get('/api/progress', (req, res) => res.json(progress));
app.get('/api/quiz', (req, res) => res.json(quizData));
app.get('/api/curriculum', (req, res) => {
  const curriculum = JSON.parse(fs.readFileSync(path.join(__dirname, 'curriculum.json'), 'utf8'));
  res.json(curriculum);
});

app.post('/api/progress/lesson', (req, res) => {
  const { lesson_id, completed } = req.body;
  if (!progress.completed_lessons) progress.completed_lessons = [];
  
  if (completed) {
    if (!progress.completed_lessons.includes(lesson_id)) {
      progress.completed_lessons.push(lesson_id);
    }
  } else {
    progress.completed_lessons = progress.completed_lessons.filter(id => id !== lesson_id);
  }
  
  saveProgress();
  res.json(progress);
});

app.post('/api/progress/:section_id', (req, res) => {
  const { section_id } = req.params;
  
  if (progress.completed_sections.includes(section_id)) {
    progress.completed_sections = progress.completed_sections.filter(id => id !== section_id);
  } else {
    progress.completed_sections.push(section_id);
  }
  
  saveProgress();
  res.json(progress);
});

app.post('/api/quiz/:question_id/submit', (req, res) => {
  const { question_id } = req.params;
  const { answer } = req.body;
  
  // Find question and check answer
  const question = quizData.questions.find(q => q.global_n === parseInt(question_id));
  if (!question) {
    return res.status(404).json({ error: 'Question not found' });
  }
  
  const isCorrect = answer === question.correct;
  const scoreRecord = {
    question_id: parseInt(question_id),
    answer,
    correct: isCorrect,
    timestamp: new Date().toISOString()
  };
  
  // Update or add score
  const existingIdx = progress.quiz_scores.findIndex(s => s.question_id === parseInt(question_id));
  if (existingIdx >= 0) {
    progress.quiz_scores[existingIdx] = scoreRecord;
  } else {
    progress.quiz_scores.push(scoreRecord);
  }
  
  saveProgress();
  res.json({ 
    correct: isCorrect, 
    correctAnswer: question.correct,
    explanation: question.explanation 
  });
});

app.get('/api/quiz/reset', (req, res) => {
  progress.quiz_scores = [];
  saveProgress();
  res.json({ success: true });
});

app.get('/api/health', (req, res) => res.send('OK'));

// Serve static files
app.use(express.static(__dirname));

app.listen(PORT, () => {
  console.log(`LMS running at http://localhost:${PORT}`);
});