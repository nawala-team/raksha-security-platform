"""Anomaly detection using Isolation Forest + Autoencoder ensemble."""

import os
from typing import Any

import numpy as np
import joblib
import structlog
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler

logger = structlog.get_logger(__name__)


class Autoencoder:
    """PyTorch-based autoencoder for anomaly detection."""

    def __init__(self, input_dim: int = 64, encoding_dim: int = 16):
        self.input_dim = input_dim
        self.encoding_dim = encoding_dim
        self.model = None
        self.threshold = 0.5
        self._build_model()

    def _build_model(self):
        """Build autoencoder architecture."""
        import torch
        import torch.nn as nn

        class AutoencoderNet(nn.Module):
            def __init__(self, input_dim: int, encoding_dim: int):
                super().__init__()
                self.encoder = nn.Sequential(
                    nn.Linear(input_dim, input_dim // 2),
                    nn.ReLU(),
                    nn.BatchNorm1d(input_dim // 2),
                    nn.Dropout(0.2),
                    nn.Linear(input_dim // 2, input_dim // 4),
                    nn.ReLU(),
                    nn.BatchNorm1d(input_dim // 4),
                    nn.Linear(input_dim // 4, encoding_dim),
                    nn.ReLU(),
                )
                self.decoder = nn.Sequential(
                    nn.Linear(encoding_dim, input_dim // 4),
                    nn.ReLU(),
                    nn.BatchNorm1d(input_dim // 4),
                    nn.Linear(input_dim // 4, input_dim // 2),
                    nn.ReLU(),
                    nn.BatchNorm1d(input_dim // 2),
                    nn.Dropout(0.2),
                    nn.Linear(input_dim // 2, input_dim),
                    nn.Sigmoid(),
                )

            def forward(self, x):
                encoded = self.encoder(x)
                decoded = self.decoder(encoded)
                return decoded

        self.model = AutoencoderNet(self.input_dim, self.encoding_dim)

    def fit(self, X: np.ndarray, epochs: int = 100, lr: float = 1e-3):
        """Train autoencoder on normal traffic data."""
        import torch
        import torch.nn as nn
        from torch.utils.data import DataLoader, TensorDataset

        self.model.train()
        tensor_x = torch.FloatTensor(X)
        dataset = TensorDataset(tensor_x, tensor_x)
        loader = DataLoader(dataset, batch_size=256, shuffle=True)

        optimizer = torch.optim.Adam(self.model.parameters(), lr=lr)
        criterion = nn.MSELoss()

        for epoch in range(epochs):
            total_loss = 0.0
            for batch_x, batch_target in loader:
                optimizer.zero_grad()
                output = self.model(batch_x)
                loss = criterion(output, batch_target)
                loss.backward()
                optimizer.step()
                total_loss += loss.item()

            if (epoch + 1) % 20 == 0:
                logger.info("autoencoder_training", epoch=epoch + 1, loss=total_loss / len(loader))

        # Set threshold as 95th percentile of reconstruction error on training data
        self.model.eval()
        with torch.no_grad():
            reconstructed = self.model(tensor_x)
            errors = torch.mean((tensor_x - reconstructed) ** 2, dim=1).numpy()
            self.threshold = float(np.percentile(errors, 95))

    def predict_scores(self, X: np.ndarray) -> np.ndarray:
        """Return reconstruction error scores."""
        import torch

        self.model.eval()


class AnomalyDetector:
    """Ensemble anomaly detector combining Isolation Forest and Autoencoder."""

    def __init__(
        self,
        contamination: float = 0.05,
        n_estimators: int = 200,
        autoencoder_dim: int = 64,
        encoding_dim: int = 16,
    ):
        self.contamination = contamination
        self.n_estimators = n_estimators
        self.scaler = StandardScaler()
        self.isolation_forest = IsolationForest(
            n_estimators=n_estimators,
            contamination=contamination,
            random_state=42,
            n_jobs=-1,
        )
        self.autoencoder = Autoencoder(input_dim=autoencoder_dim, encoding_dim=encoding_dim)
        self.is_fitted = False
        self.feature_names: list[str] = []

    def fit(self, X: np.ndarray, feature_names: list[str] | None = None):
        """Train both models on normal behavior data."""
        self.feature_names = feature_names or [f"feature_{i}" for i in range(X.shape[1])]

        # Scale features
        X_scaled = self.scaler.fit_transform(X)

        # Train Isolation Forest
        logger.info("training_isolation_forest", n_samples=X.shape[0], n_features=X.shape[1])
        self.isolation_forest.fit(X_scaled)

        # Train Autoencoder
        logger.info("training_autoencoder", input_dim=X_scaled.shape[1])
        self.autoencoder.input_dim = X_scaled.shape[1]
        self.autoencoder._build_model()
        self.autoencoder.fit(X_scaled)

        self.is_fitted = True
        logger.info("anomaly_detector_trained")

    def predict(self, features: np.ndarray) -> dict[str, Any]:
        """Predict anomaly for input features."""
        if not self.is_fitted:
            raise RuntimeError("Model not fitted. Call fit() first.")

        if features.ndim == 1:
            features = features.reshape(1, -1)

        X_scaled = self.scaler.transform(features)

        # Isolation Forest score (-1 for anomaly, 1 for normal)
        if_scores = self.isolation_forest.decision_function(X_scaled)
        if_predictions = self.isolation_forest.predict(X_scaled)

        # Autoencoder reconstruction error
        ae_scores = self.autoencoder.predict_scores(X_scaled)

        # Normalize IF scores to [0, 1] (lower decision_function = more anomalous)
        if_normalized = 1.0 - (if_scores - if_scores.min()) / (if_scores.max() - if_scores.min() + 1e-10)

        # Normalize AE scores to [0, 1]
        ae_normalized = ae_scores / (self.autoencoder.threshold * 2 + 1e-10)
        ae_normalized = np.clip(ae_normalized, 0.0, 1.0)

        # Ensemble: weighted average (IF: 0.4, AE: 0.6)
        ensemble_score = 0.4 * if_normalized + 0.6 * ae_normalized
        final_score = float(ensemble_score[0])
        is_anomaly = final_score > 0.5 or if_predictions[0] == -1

        # Confidence based on agreement between models
        if_anomaly = if_predictions[0] == -1
        ae_anomaly = ae_scores[0] > self.autoencoder.threshold
        confidence = 0.9 if if_anomaly == ae_anomaly else 0.6

        return {
            "anomaly_score": round(final_score, 4),
            "is_anomaly": bool(is_anomaly),
            "confidence": confidence,
            "details": {
                "isolation_forest_score": round(float(if_scores[0]), 4),
                "autoencoder_error": round(float(ae_scores[0]), 4),
                "ae_threshold": round(self.autoencoder.threshold, 4),
            },
        }

    def save(self, model_dir: str):
        """Persist model to disk."""
        import torch

        os.makedirs(model_dir, exist_ok=True)
        joblib.dump(self.isolation_forest, os.path.join(model_dir, "isolation_forest.joblib"))
        joblib.dump(self.scaler, os.path.join(model_dir, "scaler.joblib"))
        torch.save(self.autoencoder.model.state_dict(), os.path.join(model_dir, "autoencoder.pt"))
        joblib.dump(
            {"threshold": self.autoencoder.threshold, "input_dim": self.autoencoder.input_dim,
             "encoding_dim": self.autoencoder.encoding_dim, "feature_names": self.feature_names},
            os.path.join(model_dir, "anomaly_meta.joblib"),
        )
        logger.info("anomaly_detector_saved", path=model_dir)

    @classmethod
    def load(cls, model_dir: str) -> "AnomalyDetector":
        """Load model from disk."""
        import torch

        meta = joblib.load(os.path.join(model_dir, "anomaly_meta.joblib"))
        detector = cls(autoencoder_dim=meta["input_dim"], encoding_dim=meta["encoding_dim"])
        detector.isolation_forest = joblib.load(os.path.join(model_dir, "isolation_forest.joblib"))
        detector.scaler = joblib.load(os.path.join(model_dir, "scaler.joblib"))
        detector.autoencoder.model.load_state_dict(
            torch.load(os.path.join(model_dir, "autoencoder.pt"), weights_only=True)
        )
        detector.autoencoder.threshold = meta["threshold"]
        detector.feature_names = meta["feature_names"]
        detector.is_fitted = True
        logger.info("anomaly_detector_loaded", path=model_dir)
        return detector

        with torch.no_grad():
            tensor_x = torch.FloatTensor(X)
            reconstructed = self.model(tensor_x)
            errors = torch.mean((tensor_x - reconstructed) ** 2, dim=1).numpy()
        return errors
