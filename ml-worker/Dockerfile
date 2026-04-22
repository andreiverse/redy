FROM python:3.12-slim

WORKDIR /app

# Install system dependencies if needed (psycopg2 might need some)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements from root
COPY requirements.txt .

# Install python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Download NLTK data
RUN python -c "import nltk; nltk.download('vader_lexicon')"

# Copy ml-worker source code
COPY ml-worker/ .

# Set environment variables (can be overridden)
ENV NATS_SERVER="nats://localhost:4222"
ENV POSTGRES_URL="postgres://user:password@localhost:5432/my_app_db"

# Run the application
CMD ["python", "main.py"]
