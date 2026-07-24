cargo build --release -p backend -p migration
docker compose -f docker/docker-compose.python_test.yml up -d --build