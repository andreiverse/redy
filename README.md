# redy

A high-performance, event-driven RSS engine and content processing platform. 

`redy` is more than just an RSS aggregator. It is a full-pipeline system designed to fetch, scrape, clean, and analyze web content at scale. It leverages a distributed architecture with a Rust-based core, Python-powered ML workers, and a modern React frontend.

## 🚀 Key Features

*   **Intelligent Aggregation**: Scheduled RSS/Atom feed synchronization with stateful tracking.
*   **Deep Content Extraction**: Advanced scraping engine supporting multiple strategies:
    *   **Normal**: Direct HTTP requests for standard sites.
    *   **Googlebot Spoofing**: Bypasses basic bot detection.
    *   **AMP Support**: Prioritizes accelerated mobile pages for faster, cleaner parsing.
*   **Content Sanitization**: Integrated readability algorithms to extract core article text, stripping ads, trackers, and navigation clutter.
*   **Asynchronous ML Pipeline**: Distributed sentiment analysis using Python and NLTK, coordinated via NATS Jetstream.
*   **Enterprise-Grade Auth**: OIDC-compliant authentication (Keycloak, Google, Authelia) with Redis-backed session management.
*   **Full Observability**: Exported Prometheus metrics and pre-configured Grafana dashboards for monitoring system health and worker performance.

## 🛠 Tech Stack

### Backend (Rust)
- **Framework**: [Axum](https://github.com/tokio-rs/axum) for a high-performance, type-safe API.
- **ORM**: [SeaORM](https://www.sea-ql.org/SeaORM/) with PostgreSQL for robust data modeling.
- **Messaging**: [NATS Jetstream](https://nats.io/) for reliable, distributed task distribution.
- **Documentation**: Automatic OpenAPI (Swagger) generation via [Utoipa](https://github.com/juhakivekas/utoipa).

### ML Worker (Python)
- **Engine**: [NLTK](https://www.nltk.org/) (Vader) for sentiment scoring.
- **Integration**: Asynchronous NATS consumers for processing content streams.

### Frontend (TypeScript/React)
- **Framework**: [React 19](https://react.dev/) with [TanStack Start](https://tanstack.com/start) (SSR-ready).
- **Routing & State**: [TanStack Router](https://tanstack.com/router) & [TanStack Query](https://tanstack.com/query).
- **Styling**: [Tailwind CSS 4](https://tailwindcss.com/) and [Shadcn UI](https://ui.shadcn.com/).

## 🏗 Architecture

`redy` follows an event-driven microservices pattern:

1.  **Server (Rust)**: Manages the API, user state, and feed scheduling. It publishes "scrape tasks" to NATS.
2.  **Scrape Worker (Rust)**: Consumes tasks, executes the appropriate scraping strategy, extracts the clean content, and publishes an "analysis task".
3.  **ML Worker (Python)**: Consumes analysis tasks, calculates sentiment scores, and updates the database.
4.  **Client (React)**: Provides a responsive, real-time interface for reading and managing feeds.

## 🚦 Getting Started

### Prerequisites
- Docker & Docker Compose
- Rust (1.80+)
- Node.js (v20+) & pnpm

### Quick Start (Docker)
```bash
docker-compose up -d
```
The system will be available at:
- Frontend: `http://localhost:3000`
- API / Swagger: `http://localhost:8080/swagger-ui`
- Metrics: `http://localhost:9091/metrics`

### Development

**Backend:**
```bash
cd server
cargo run -- run-server
```

**Frontend:**
```bash
cd client
pnpm install
pnpm dev
```

**ML Worker:**
```bash
cd ml-worker
pip install -r requirements.txt
python main.py
```

## 📊 Monitoring
The project includes a `monitoring/` directory with Grafana dashboards. It tracks:
- Feed fetch success/failure rates.
- Worker latency and throughput.
- Database connection pool health.
- ML analysis distribution.

## ⚖️ License
This project is licensed under the [LICENSE](LICENSE) provided in the repository.
