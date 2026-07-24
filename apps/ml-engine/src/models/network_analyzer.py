"""Network traffic anomaly detection model."""

import os
from typing import Any

import numpy as np
import joblib
import structlog
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler, LabelEncoder

logger = structlog.get_logger(__name__)


class NetworkAnalyzer:
    """Detects anomalous network traffic patterns.

    Monitors: packet rates, connection patterns, protocol distributions,
    DNS queries, port usage, and lateral movement indicators.
    """

    FEATURE_NAMES = [
        "packets_per_second", "bytes_per_second", "unique_dst_ips",
        "unique_dst_ports", "avg_packet_size", "syn_ratio",
        "dns_query_count", "failed_connections", "connection_duration_avg",
        "protocol_entropy", "port_entropy", "inbound_outbound_ratio",
        "new_external_connections", "internal_scan_indicators",
        "payload_entropy", "beacon_score",
    ]

    def __init__(self, contamination: float = 0.05):
        self.contamination = contamination
        self.scaler = StandardScaler()
        self.isolation_forest = IsolationForest(
            n_estimators=200,
            contamination=contamination,
            random_state=42,
            n_jobs=-1,
        )
        self.baseline_stats: dict[str, float] = {}
        self.is_fitted = False

    def fit(self, X: np.ndarray):
        """Train on normal network traffic patterns."""
        logger.info("training_network_analyzer", n_samples=X.shape[0])
        X_scaled = self.scaler.fit_transform(X)
        self.isolation_forest.fit(X_scaled)

        # Store baseline statistics for deviation scoring
        self.baseline_stats = {
            "mean": X_scaled.mean(axis=0).tolist(),
            "std": X_scaled.std(axis=0).tolist(),
            "p95": np.percentile(X_scaled, 95, axis=0).tolist(),
            "p99": np.percentile(X_scaled, 99, axis=0).tolist(),
        }
        self.is_fitted = True

    def predict(self, features: np.ndarray) -> dict[str, Any]:
        """Predict if network traffic is anomalous."""
        if not self.is_fitted:
            raise RuntimeError("Model not fitted.")
        if features.ndim == 1:
            features = features.reshape(1, -1)

        X_scaled = self.scaler.transform(features)
        if_score = self.isolation_forest.decision_function(X_scaled)[0]
        if_pred = self.isolation_forest.predict(X_scaled)[0]

        mean = np.array(self.baseline_stats["mean"])
        std = np.array(self.baseline_stats["std"]) + 1e-10
        z_scores = np.abs((X_scaled[0] - mean) / std)
        max_deviation = float(z_scores.max())
        avg_deviation = float(z_scores.mean())

        normalized_if = float(np.clip(1.0 - (if_score + 0.5), 0, 1))
        deviation_score = min(avg_deviation / 3.0, 1.0)
        final_score = 0.6 * normalized_if + 0.4 * deviation_score
        is_anomaly = final_score > 0.5 or if_pred == -1
        indicators = self._detect_attack_patterns(features[0], z_scores)

        return {
            "anomaly_score": round(final_score, 4),
            "is_anomaly": bool(is_anomaly),
            "confidence": round(0.85 if max_deviation > 3 else 0.7, 2),
            "details": {
                "isolation_forest_score": round(float(if_score), 4),
                "max_z_score": round(max_deviation, 4),
                "attack_indicators": indicators,
            },
        }

    def _detect_attack_patterns(self, features: np.ndarray, z_scores: np.ndarray) -> list[str]:
        """Identify potential attack patterns from network features."""
        indicators = []
        if len(features) >= len(self.FEATURE_NAMES):
            if features[2] > 50 and features[3] > 20:
                indicators.append("potential_port_scan")
            if features[5] > 0.8:
                indicators.append("syn_flood_indicator")
            if features[6] > 100:
                indicators.append("dns_tunneling_suspect")
            if features[13] > 5:
                indicators.append("lateral_movement_detected")
            if features[14] > 7.5:
                indicators.append("encrypted_exfiltration")
            if features[15] > 0.8:
                indicators.append("c2_beacon_pattern")
        return indicators

    def save(self, model_dir: str):
        os.makedirs(model_dir, exist_ok=True)
        joblib.dump({
            "isolation_forest": self.isolation_forest,
            "scaler": self.scaler,
            "baseline_stats": self.baseline_stats,
            "contamination": self.contamination,
        }, os.path.join(model_dir, "network_analyzer.joblib"))

    @classmethod
    def load(cls, model_dir: str) -> "NetworkAnalyzer":
        data = joblib.load(os.path.join(model_dir, "network_analyzer.joblib"))
        analyzer = cls(contamination=data["contamination"])
        analyzer.isolation_forest = data["isolation_forest"]
        analyzer.scaler = data["scaler"]
        analyzer.baseline_stats = data["baseline_stats"]
        analyzer.is_fitted = True
        return analyzer

        logger.info("network_analyzer_trained")
