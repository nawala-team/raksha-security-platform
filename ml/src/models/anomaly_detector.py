"""
Anomaly Detection Engine using Isolation Forest

Detects anomalies in:
- Network traffic patterns (bytes, connections, protocols)
- Server metrics (CPU, memory, disk, load)
- Authentication events (login frequency, geo-anomalies)
- Database queries (query rate, slow queries, unusual patterns)
"""

from __future__ import annotations

import numpy as np
import pandas as pd
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler
from joblib import dump, load
from pathlib import Path
from datetime import datetime
from pydantic import BaseModel
import structlog

logger = structlog.get_logger()


class AnomalyPrediction(BaseModel):
    """Result of anomaly detection inference."""
    event_id: str
    score: float
    is_anomaly: bool
    confidence: float
    top_features: list[dict[str, float]]
    model_version: str
    detected_at: str


class AnomalyDetector:
    """Isolation Forest based anomaly detector for security events."""

    def __init__(self, model_dir: str = "./models", contamination: float = 0.05):
        self.model_dir = Path(model_dir)
        self.model_dir.mkdir(parents=True, exist_ok=True)
        self.contamination = contamination
        self.model: IsolationForest | None = None
        self.scaler: StandardScaler | None = None
        self.feature_names: list[str] = []
        self.model_version = "v0.1.0"
        self._is_trained = False

    @property
    def is_trained(self) -> bool:
        return self._is_trained

    def train(self, data: pd.DataFrame, feature_columns: list[str] | None = None) -> dict:
        """Train the anomaly detection model on historical data."""
        if feature_columns:
            self.feature_names = feature_columns
            X = data[feature_columns].values
        else:
            self.feature_names = list(data.select_dtypes(include=[np.number]).columns)
            X = data[self.feature_names].values

        if len(X) < 50:
            raise ValueError(f"Need at least 50 samples to train. Got: {len(X)}")

        self.scaler = StandardScaler()
        X_scaled = self.scaler.fit_transform(X)

        self.model = IsolationForest(
            contamination=self.contamination,
            n_estimators=200,
            max_samples="auto",
            random_state=42,
            n_jobs=-1,
        )
        self.model.fit(X_scaled)
        self._is_trained = True
        self.model_version = f"v{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}"

        scores = self.model.decision_function(X_scaled)
        predictions = self.model.predict(X_scaled)
        anomaly_count = int((predictions == -1).sum())

        logger.info("model_trained", samples=len(X), features=len(self.feature_names),
                    anomalies_found=anomaly_count, version=self.model_version)

        return {
            "model_version": self.model_version,
            "training_samples": len(X),
            "features": self.feature_names,
            "anomalies_in_training": anomaly_count,
            "mean_score": float(scores.mean()),
            "std_score": float(scores.std()),
        }

    def predict(self, event_id: str, features: dict[str, float]) -> AnomalyPrediction:
        """Run anomaly detection on a single event."""
        if not self._is_trained or self.model is None or self.scaler is None:
            raise RuntimeError("Model not trained. Call train() first.")

        X = np.array([[features.get(f, 0.0) for f in self.feature_names]])
        X_scaled = self.scaler.transform(X)

        raw_score = self.model.decision_function(X_scaled)[0]
        prediction = self.model.predict(X_scaled)[0]

        anomaly_score = max(0.0, min(1.0, 0.5 - raw_score))
        is_anomaly = prediction == -1
        confidence = abs(raw_score) / (abs(raw_score) + 0.5)

        top_features = self._compute_feature_importance(X_scaled[0])

        return AnomalyPrediction(
            event_id=event_id,
            score=round(anomaly_score, 4),
            is_anomaly=is_anomaly,
            confidence=round(confidence, 4),
            top_features=top_features[:5],
            model_version=self.model_version,
            detected_at=datetime.utcnow().isoformat() + "Z",
        )

    def _compute_feature_importance(self, x_scaled: np.ndarray) -> list[dict[str, float]]:
        """Approximate feature importance based on deviation from mean."""
        contributions = []
        for i, name in enumerate(self.feature_names):
            contributions.append({"feature": name, "deviation": round(abs(float(x_scaled[i])), 4)})
        contributions.sort(key=lambda c: c["deviation"], reverse=True)
        return contributions

    def save(self, name: str = "anomaly_model") -> str:
        """Save model to disk."""
        if not self._is_trained:
            raise RuntimeError("No trained model to save.")
        model_path = self.model_dir / f"{name}.joblib"
        scaler_path = self.model_dir / f"{name}_scaler.joblib"
        meta_path = self.model_dir / f"{name}_meta.joblib"
        dump(self.model, model_path)
        dump(self.scaler, scaler_path)
        dump({"feature_names": self.feature_names, "model_version": self.model_version}, meta_path)
        logger.info("model_saved", path=str(model_path), version=self.model_version)
        return str(model_path)

    def load_model(self, name: str = "anomaly_model") -> bool:
        """Load model from disk."""
        model_path = self.model_dir / f"{name}.joblib"
        scaler_path = self.model_dir / f"{name}_scaler.joblib"
        meta_path = self.model_dir / f"{name}_meta.joblib"
        if not model_path.exists():
            logger.warning("model_not_found", path=str(model_path))
            return False
        self.model = load(model_path)
        self.scaler = load(scaler_path)
        meta = load(meta_path)
        self.feature_names = meta["feature_names"]
        self.model_version = meta["model_version"]
        self._is_trained = True
        logger.info("model_loaded", version=self.model_version)
        return True

