use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;

use raksha_alert::AlertEngine;
use raksha_audit::AuditStore;
use raksha_auth::{SessionManager, TokenService};
use raksha_compliance::ComplianceChecker;
use raksha_core::AppConfig;
use raksha_ml::{AnomalyDetector, RiskScorer};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub token_service: TokenService,
    pub session_manager: SessionManager,
    pub alert_engine: AlertEngine,
    pub audit_store: AuditStore,
    pub compliance_checker: ComplianceChecker,
    pub anomaly_detector: Arc<AnomalyDetector>,
    pub risk_scorer: Arc<RiskScorer>,
    pub realtime_hub: crate::handlers::websocket::RealtimeHub,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> anyhow::Result<Self> {
        // Initialize database pool
        let db = raksha_core::db::create_pool(&config.database).await?;
        tracing::info!("Database pool established");

        // Run migrations (in production, do this separately)
        // sqlx::migrate!("./migrations").run(&db).await?;

        // Initialize Redis pool
        let redis = raksha_core::redis::create_pool(&config.redis)?;
        tracing::info!("Redis pool established");

        // Initialize services
        let token_service = TokenService::new(&config.jwt);
        let session_manager = SessionManager::new(redis.clone(), config.jwt.refresh_token_ttl_secs);
        let alert_engine = AlertEngine::new(db.clone());
        let audit_store = AuditStore::new(db.clone());
        let compliance_checker = ComplianceChecker::new(db.clone());
        let anomaly_detector = Arc::new(AnomalyDetector::new(0.7));
        let risk_scorer = Arc::new(RiskScorer::new());

        Ok(Self {
            db,
            redis,
            token_service,
            session_manager,
            alert_engine,
            audit_store,
            compliance_checker,
            anomaly_detector,
            risk_scorer,
            realtime_hub: crate::handlers::websocket::RealtimeHub::new(1024),
            config: Arc::new(config.clone()),
        })
    }
}
