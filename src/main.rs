use axum::{
    extract::Json,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, net::SocketAddr};

#[derive(Debug, Deserialize)]
struct SubQuestion {
    question: String,
    options: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
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
    follow_ups: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
struct DiagnosisResult {
    diagnosis: String,
    likelihood: f64,
    recommended_action: String,
}

fn load_data(file_path: &str) -> Vec<Diagnosis> {
    let file_content = fs::read_to_string(file_path)
        .expect("Failed to read JSON file");
    serde_json::from_str(&file_content)
        .expect("Failed to parse JSON data")
}

async fn diagnose(Json(payload): Json<SymptomInput>) -> impl IntoResponse {
    let data = load_data("data.json");
    let symptoms = payload.symptoms;
    let follow_ups = payload.follow_ups.unwrap_or_default();

    let mut results = Vec::new();

    for diagnosis in &data {
        if let Some(sub_questions) = &diagnosis.sub_questions {
            for symptom in &symptoms {
                if let Some(sub_question) = sub_questions.get(symptom) {
                    if let Some(answer) = follow_ups.get(symptom) {
                        if let Some(sub_diagnoses) = sub_question.options.get(answer) {
                            for diag in sub_diagnoses {
                                results.push(DiagnosisResult {
                                    diagnosis: diag.clone(),
                                    likelihood: 100.0,
                                    recommended_action: diagnosis.recommended_action.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Json(results)
}

#[tokio::main]
async fn main() {
    // ✅ Use Railway's dynamic port if available
    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running at http://{}", addr);

    let app = axum::Router::new().route("/diagnose", axum::routing::post(diagnose));

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app,
    )
    .await
    .unwrap();
}

