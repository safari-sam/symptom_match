use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;


#[derive(Debug, Deserialize, Clone)]
struct SubQuestion {
    question: String,
    options: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct Diagnosis {
    diagnosis: String,
    symptoms: Vec<String>,
    required_symptoms: Vec<String>,
    sub_questions: Option<HashMap<String, SubQuestion>>,
    recommended_action: String,
}

#[derive(Debug, Deserialize)]
struct SymptomInput {
    symptoms: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DiagnosisResult {
    diagnosis: String,
    confidence: f64,
    recommended_action: String,
}

#[derive(Clone)]
struct AppState {
    data: Arc<RwLock<Vec<Diagnosis>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = load_data("data.json")?;
    let shared_state = AppState {
        data: Arc::new(RwLock::new(data)),
    };

    let app = Router::new()
        .route("/diagnose", post(handle_diagnosis))
        .with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}


async fn handle_diagnosis(
    State(state): State<AppState>,
    Json(payload): Json<SymptomInput>,
) -> Json<Vec<DiagnosisResult>> {
    let input_symptoms: Vec<String> = payload
        .symptoms
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    let data = state.data.read().await;
    let mut results = Vec::new();

    for diagnosis in data.iter() {
        let total = diagnosis.symptoms.len();
        if total == 0 {
            continue;
        }

        let matched = diagnosis
            .symptoms
            .iter()
            .filter(|s| input_symptoms.contains(&s.to_lowercase()))
            .count();

        if matched == 0 {
            continue;
        }

        let mut score = (matched as f64 / total as f64) * 100.0;

        let required_present = diagnosis
            .required_symptoms
            .iter()
            .all(|r| input_symptoms.contains(&r.to_lowercase()));
        if required_present && !diagnosis.required_symptoms.is_empty() {
            score += 10.0;
        }

        results.push(DiagnosisResult {
            diagnosis: diagnosis.diagnosis.clone(),
            confidence: score,
            recommended_action: diagnosis.recommended_action.clone(),
        });
    }

    results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    Json(results)
}

fn load_data(file_path: &str) -> Result<Vec<Diagnosis>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let data: Vec<Diagnosis> = serde_json::from_str(&content)?;
    Ok(data)
}
