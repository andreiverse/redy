import os

NATS_SERVER = os.getenv("NATS_SERVER", "nats://localhost:4222") 
POSTGRES_URL = os.getenv("POSTGRES_URL", "postgres://user:password@localhost:5432/my_app_db") 
TIMEOUT_ON_FAIL_SECONDS = 30