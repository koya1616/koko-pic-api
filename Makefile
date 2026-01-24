.PHONY: build up down logs shell

# Build the Docker image
build:
	docker-compose build

# Start services with Docker Compose
up:
	docker-compose up -d

# Stop services
down:
	docker-compose down

# View logs
logs:
	docker-compose logs -f

# Shell into the app container
shell:
	docker-compose exec app /bin/sh

# Rebuild and restart
restart: down up

# Show status of containers
ps:
	docker-compose ps