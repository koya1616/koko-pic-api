.PHONY: build up down logs shell restart ps build-prod up-prod down-prod push-prod

# ============================================
# 開発用
# ============================================

build:
	docker compose build

up:
	docker compose up

down:
	docker compose down

logs:
	docker compose logs -f

shell:
	docker compose exec app /bin/bash

restart:
	docker compose up --build

ps:
	docker compose ps

check:
	docker exec koko-pic-api-app-1 sh -c "cargo fmt && cargo clippy -- -D warnings && cargo test"

# ============================================
# 本番用
# ============================================

build-prod:
	docker buildx build --platform linux/amd64 -t koko-pic-api-app:latest -f Dockerfile .

up-prod:
	docker compose -f docker-compose.prod.yml up -d

down-prod:
	docker compose -f docker-compose.prod.yml down

push-prod: build-prod
	docker tag koko-pic-api-app:latest kuuuuya/koko-pic-api:latest
	docker push kuuuuya/koko-pic-api:latest