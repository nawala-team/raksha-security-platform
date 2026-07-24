"""
Raksha ML Engine - FastAPI Application

Provides REST endpoints for:
- Anomaly detection inference
- Model training triggers
- Health and status checks
"""

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from contextlib import asynccontextmanager
import structlog

from .models import AnomalyDetector, AnomalyPrediction

logger = structlog.get_logger()

detector = AnomalyDetector(model_dir="./models")


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Load model on startup if available."""
    loaded = detector.load_model()
    if loaded:
        logger.info("ml_engine_ready", model=detector.model_version)
    else:
        logger.info("ml_engine_started", status="no_model_loaded")
    yield
    logger.info("ml_engine_shutdown")


app = FastAPI(
    title="Raksha ML Engine",
    version="0.1.0",
    description="Machine Learning API for security anomaly detection",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)


class PredictRequest(BaseModel):
    event_id: str
    features: dict[str, float]


class PredictBatchRequest(BaseModel):
    events: list[PredictRequest]


class HealthResponse(BaseModel):
    status: str
    model_loaded: bool
    model_version: str | None


@app.get("/health", response_model=HealthResponse)
async def health():
    return HealthResponse(
        status="healthy",
        model_loaded=detector.is_trained,
        model_version=detector.model_version if detector.is_trained else None,
    )


@app.post("/api/v1/predict", response_model=AnomalyPrediction)
async def predict(req: PredictRequest):
    if not detector.is_trained:
        raise HTTPException(status_code=503, detail="Model not trained yet")
    try:
        result = detector.predict(req.event_id, req.features)
        return result
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/v1/predict/batch", response_model=list[AnomalyPrediction])
async def predict_batch(req: PredictBatchRequest):
    if not detector.is_trained:
        raise HTTPException(status_code=503, detail="Model not trained yet")
    results = []
    for event in req.events:
        results.append(detector.predict(event.event_id, event.features))
    return results


@app.get("/api/v1/model/status")
async def model_status():
    return {
        "is_trained": detector.is_trained,
        "model_version": detector.model_version,
        "features": detector.feature_names,
    }
