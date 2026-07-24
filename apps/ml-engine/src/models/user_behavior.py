"""User Behavior Analytics (UBA) model for detecting insider threats."""

import os
from typing import Any

import numpy as np
import joblib
import structlog
from sklearn.ensemble import GradientBoostingClassifier
from sklearn.preprocessing import StandardScaler
from sklearn.cluster import DBSCAN

logger = structlog.get_logger(__name__)


class UserBehaviorAnalyzer:
    """Detects anomalous user behavior patterns indicating insider threats."""

    FEATURE_NAMES = [
        "login_hour", "login_day_of_week", "session_duration_minutes",
        "failed_login_count", "unique_resources_accessed", "data_download_mb",
        "data_upload_mb", "privilege_escalation_count", "off_hours_activity_ratio",
        "new_device_login", "geo_distance_km", "concurrent_sessions",
        "admin_actions_count", "sensitive_file_access_count",
        "avg_request_interval_seconds", "unusual_port_access_count",
    ]

    def __init__(self, contamination: float = 0.03):
        self.contamination = contamination
        self.scaler = StandardScaler()
        self.cluster_model = DBSCAN(eps=0.5, min_samples=5)
        self.classifier = GradientBoostingClassifier(
            n_estimators=150, max_depth=5, learning_rate=0.1, random_state=42,
        )
        self.user_baselines: dict[str, np.ndarray] = {}
        self.is_fitted = False

    def fit(self, X: np.ndarray, user_ids: list[str] | None = None):
        """Train on historical normal user behavior."""
        logger.info("training_uba_model", n_samples=X.shape[0])
        X_scaled = self.scaler.fit_transform(X)

        if user_ids:
            for uid in set(user_ids):
                mask = [i for i, u in enumerate(user_ids) if u == uid]
                self.user_baselines[uid] = np.mean(X_scaled[mask], axis=0)

        self.cluster_model.fit(X_scaled)

        # Synthetic anomaly generation for supervised training
        n = X_scaled.shape[0]
        n_anom = max(int(n * self.contamination), 10)
        rng = np.random.default_rng(42)
        idx = rng.choice(n, n_anom, replace=False)
        X_anom = X_scaled[idx] + rng.normal(0, 2, (n_anom, X_scaled.shape[1]))

        X_train = np.vstack([X_scaled, X_anom])
        y_train = np.concatenate([np.zeros(n), np.ones(n_anom)])
        self.classifier.fit(X_train, y_train)
        self.is_fitted = True
        logger.info("uba_model_trained", n_users=len(self.user_baselines))


    def predict(self, features: np.ndarray, user_id: str | None = None) -> dict[str, Any]:
        """Predict if user behavior is anomalous."""
        if not self.is_fitted:
            raise RuntimeError("Model not fitted.")
        if features.ndim == 1:
            features = features.reshape(1, -1)

        X_scaled = self.scaler.transform(features)
        proba = self.classifier.predict_proba(X_scaled)[0]
        anomaly_prob = float(proba[1]) if len(proba) > 1 else 0.0

        baseline_deviation = 0.0
        if user_id and user_id in self.user_baselines:
            baseline = self.user_baselines[user_id]
            baseline_deviation = float(np.linalg.norm(X_scaled[0] - baseline))

        score = 0.7 * anomaly_prob + 0.3 * min(baseline_deviation / 5.0, 1.0)
        risk_factors = self._identify_risk_factors(features[0])

        return {
            "anomaly_score": round(score, 4),
            "is_anomaly": score > 0.5,
            "confidence": round(0.85 if baseline_deviation > 0 else 0.7, 2),
            "details": {
                "classifier_score": round(anomaly_prob, 4),
                "baseline_deviation": round(baseline_deviation, 4),
                "risk_factors": risk_factors,
            },
        }

    def _identify_risk_factors(self, features: np.ndarray) -> list[str]:
        """Identify which behavioral factors are risky."""
        factors = []
        if len(features) >= len(self.FEATURE_NAMES):
            if features[3] > 5:
                factors.append("excessive_failed_logins")
            if features[5] > 100:
                factors.append("large_data_download")
            if features[8] > 0.7:
                factors.append("unusual_hours_activity")
            if features[9] > 0:
                factors.append("new_device_detected")
            if features[10] > 500:
                factors.append("impossible_travel")
            if features[12] > 10:
                factors.append("excessive_admin_actions")
        return factors

    def save(self, model_dir: str):
        os.makedirs(model_dir, exist_ok=True)
        joblib.dump({
            "scaler": self.scaler, "classifier": self.classifier,
            "baselines": self.user_baselines, "contamination": self.contamination,
        }, os.path.join(model_dir, "user_behavior.joblib"))

    @classmethod
    def load(cls, model_dir: str) -> "UserBehaviorAnalyzer":
        data = joblib.load(os.path.join(model_dir, "user_behavior.joblib"))
        analyzer = cls(contamination=data["contamination"])
        analyzer.scaler = data["scaler"]
        analyzer.classifier = data["classifier"]
        analyzer.user_baselines = data["baselines"]
        analyzer.is_fitted = True
        return analyzer
