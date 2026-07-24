"""Tests for threat intelligence feed integrations."""

import pytest
from unittest.mock import AsyncMock, patch, MagicMock

import httpx

from src.feeds.otx import OTXFeed


class TestOTXFeed:
    """Tests for the AlienVault OTX feed client."""

    def test_initialization(self):
        """OTXFeed initializes with API key and correct headers."""
        feed = OTXFeed(api_key="test-api-key-123")
        assert feed.api_key == "test-api-key-123"
        assert feed.headers["X-OTX-API-KEY"] == "test-api-key-123"

    @pytest.mark.asyncio
    async def test_fetch_subscribed_pulses_success(self, mock_otx_pulse_response):
        """Successfully fetches and parses subscribed pulses."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_otx_pulse_response
        mock_response.raise_for_status = MagicMock()

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_client = AsyncMock()
            mock_client.get = AsyncMock(return_value=mock_response)
            mock_client.__aenter__ = AsyncMock(return_value=mock_client)
            mock_client.__aexit__ = AsyncMock(return_value=False)
            mock_client_cls.return_value = mock_client

            feed = OTXFeed(api_key="test-key")
            pulses = await feed.fetch_subscribed_pulses(days=7, limit=50)

        assert len(pulses) == 2
        assert pulses[0]["id"] == "pulse-001"
        assert pulses[0]["name"] == "APT28 Campaign Indicators"
        assert "apt28" in pulses[0]["tags"]
        assert pulses[0]["indicator_count"] == 47

    @pytest.mark.asyncio
    async def test_fetch_subscribed_pulses_api_error(self):
        """Returns empty list on API error."""
        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_client = AsyncMock()
            mock_client.get = AsyncMock(
                side_effect=httpx.HTTPStatusError(
                    "Server Error",
                    request=MagicMock(),
                    response=MagicMock(status_code=500),
                )
            )
            mock_client.__aenter__ = AsyncMock(return_value=mock_client)
            mock_client.__aexit__ = AsyncMock(return_value=False)
            mock_client_cls.return_value = mock_client

            feed = OTXFeed(api_key="test-key")
            pulses = await feed.fetch_subscribed_pulses()

        assert pulses == []

    @pytest.mark.asyncio
    async def test_check_ip_reputation(self, mock_ip_reputation_response):
        """Checks IP reputation and parses response correctly."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_ip_reputation_response
        mock_response.raise_for_status = MagicMock()

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_client = AsyncMock()
            mock_client.get = AsyncMock(return_value=mock_response)
            mock_client.__aenter__ = AsyncMock(return_value=mock_client)
            mock_client.__aexit__ = AsyncMock(return_value=False)
            mock_client_cls.return_value = mock_client

            feed = OTXFeed(api_key="test-key")
            result = await feed.check_ip("198.51.100.23")

        assert result["ip"] == "198.51.100.23"
        assert result["reputation"] == 42
        assert result["pulse_count"] == 5
        assert result["country"] == "Russia"

    def test_parse_pulses_empty(self):
        """Handles empty pulse response gracefully."""
        feed = OTXFeed(api_key="test-key")
        result = feed._parse_pulses({"results": []})
        assert result == []

    def test_parse_indicators_empty(self):
        """Handles empty indicator list."""
        feed = OTXFeed(api_key="test-key")
        result = feed._parse_indicators([])
        assert result == []

    def test_parse_pulses_missing_fields(self):
        """Handles pulses with missing optional fields."""
        feed = OTXFeed(api_key="test-key")
        result = feed._parse_pulses({
            "results": [{"id": "pulse-minimal"}]
        })
        assert len(result) == 1
        assert result[0]["id"] == "pulse-minimal"
        assert result[0]["name"] == ""
        assert result[0]["tags"] == []
        assert result[0]["indicator_count"] == 0
