.PHONY: build up down logs shell restart ps build-prod up-prod down-prod

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

# ============================================
# 本番用
# ============================================

build-prod:
	docker compose -f docker-compose.prod.yml build

up-prod:
	docker compose -f docker-compose.prod.yml up -d

down-prod:
	docker compose -f docker-compose.prod.yml down