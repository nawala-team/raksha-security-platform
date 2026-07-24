"""Raksha ML Engine - FastAPI application for model training and inference."""

import os
from contextlib import asynccontextmanager
from typing import Any

import structlog
from fastapi import FastAPI, HTTPException, BackgroundTasks
from pydantic import BaseModel, Field
from prometheus_client import Counter, Histogram, generate_latest
from fastapi.responses import PlainTextResponse

from src.models.anomaly_detector import AnomalyDetector
from src.models.user_behavior import UserBehaviorAnalyzer
from src.models.network_analyzer import NetworkAnalyzer
from src.pipeline.trainer import TrainingPipeline
from src.pipeline.feature_engineering import FeatureEngineer

logger = structlog.get_logger(__name__)

# Prometheus metrics
PREDICTION_COUNTER = Counter(
    "raksha_ml_predictions_total", "Total predictions", ["model_type", "result"]
)
PREDICTION_LATENCY = Histogram(
    "raksha_ml_prediction_latency_seconds", "Prediction latency", ["model_type"]
)
TRAINING_COUNTER = Counter(
    "raksha_ml_training_runs_total", "Training runs", ["model_type", "status"]
)

models: dict[str, Any] = {}


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Initialize models on startup."""
    logger.info("raksha_ml_engine_starting", version="0.1.0")
    model_dir = os.getenv("MODEL_DIR", "./models")
    if os.path.isdir(model_dir):
        try:
            models["anomaly_detector"] = AnomalyDetector.load(model_dir)
            models["user_behavior"] = UserBehaviorAnalyzer.load(model_dir)
            models["network_analyzer"] = NetworkAnalyzer.load(model_dir)
            logger.info("models_loaded", count=len(models))
        except FileNotFoundError:
            logger.warning("no_pretrained_models", model_dir=model_dir)
    yield
    logger.info("raksha_ml_engine_shutting_down")


app = FastAPI(
    title="Raksha ML Engine",
    description="Security anomaly detection and threat intelligence ML platform",
    version="0.1.0",
    lifespan=lifespan,
)


# --- Request/Response Models ---


class PredictionRequest(BaseModel):
    model_type: str = Field(..., description="Model: anomaly_detector, user_behavior, network_analyzer")
    features: dict[str, Any] = Field(..., description="Feature dictionary for prediction")


class PredictionResponse(BaseModel):
    anomaly_score: float = Field(..., description="Anomaly score 0.0-1.0")
    is_anomaly: bool
    confidence: float
    details: dict[str, Any] = Field(default_factory=dict)


class TrainingRequest(BaseModel):
    model_type: str
    data_source: str = Field(..., description="Path or URI to training data")
    hyperparameters: dict[str, Any] = Field(default_factory=dict)
    optimize: bool = Field(default=False, description="Run Optuna optimization")


class TrainingResponse(BaseModel):
    job_id: str
    status: str
    message: str


class HealthResponse(BaseModel):
    status: str
    models_loaded: list[str]
    version: str



# --- Endpoints ---


@app.get("/health", response_model=HealthResponse)
async def health_check():
    return HealthResponse(status="healthy", models_loaded=list(models.keys()), version="0.1.0")


@app.get("/metrics", response_class=PlainTextResponse)
async def metrics():
    return generate_latest().decode("utf-8")


@app.post("/predict", response_model=PredictionResponse)
async def predict(request: PredictionRequest):
    """Run anomaly detection prediction."""
    if request.model_type not in models:
        raise HTTPException(404, f"Model '{request.model_type}' not loaded.")

    model = models[request.model_type]
    with PREDICTION_LATENCY.labels(model_type=request.model_type).time():
        try:
            engineer = FeatureEngineer()
            features = engineer.transform(request.features, request.model_type)
            result = model.predict(features)
            label = "anomaly" if result["is_anomaly"] else "normal"
            PREDICTION_COUNTER.labels(model_type=request.model_type, result=label).inc()
            return PredictionResponse(
                anomaly_score=result["anomaly_score"],
                is_anomaly=result["is_anomaly"],
                confidence=result["confidence"],
                details=result.get("details", {}),
            )
        except Exception as e:
            PREDICTION_COUNTER.labels(model_type=request.model_type, result="error").inc()
            logger.error("prediction_failed", error=str(e))
            raise HTTPException(500, f"Prediction failed: {e}")


@app.post("/train", response_model=TrainingResponse)
async def train_model(request: TrainingRequest, background_tasks: BackgroundTasks):
    """Trigger model training in background."""
    pipeline = TrainingPipeline()
    job_id = pipeline.create_job(
        model_type=request.model_type,
        data_source=request.data_source,
        hyperparameters=request.hyperparameters,
        optimize=request.optimize,
    )
    background_tasks.add_task(_run_training, pipeline, job_id, request.model_type)
    return TrainingResponse(job_id=job_id, status="queued", message=f"Training '{job_id}' queued")


async def _run_training(pipeline: TrainingPipeline, job_id: str, model_type: str):
    try:
        result = await pipeline.run(job_id)
        if result.get("success"):
            models[model_type] = result["model"]
            TRAINING_COUNTER.labels(model_type=model_type, status="success").inc()
        else:
            TRAINING_COUNTER.labels(model_type=model_type, status="failed").inc()
    except Exception as e:
        TRAINING_COUNTER.labels(model_type=model_type, status="error").inc()
        logger.error("training_error", job_id=job_id, error=str(e))


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "src.main:app",
        host="0.0.0.0",
        port=int(os.getenv("ML_ENGINE_PORT", "8000")),
        reload=os.getenv("ENV", "production") == "development",
    )
